use super::workspace_fs_metadata_impl::{
    dir_entry_value, file_info_value, FILE_INFO_SOURCE_FS, FILE_INFO_SOURCE_OS,
};
use crate::{Program, Value, ValueData, Vm, VmError, TYPE_FS_FILE, TYPE_INT, TYPE_OS_DIR_FS};

pub(super) use super::workspace_fs_metadata_impl::{
    dir_entry_info, dir_entry_is_dir, dir_entry_name, dir_entry_type,
    dir_entry_value_from_file_info, file_info_is_dir, file_info_mod_time, file_info_mode,
    file_info_name, file_info_size, file_mode_is_dir, file_mode_is_regular, file_mode_perm,
    file_mode_string, file_mode_type, same_file_workspace_path,
};

const DIR_FS_ROOT_FIELD: &str = "__dirfs_root";
const FILE_ID_FIELD: &str = "__fs_file_id";

pub(super) fn dirfs_value(root: &str) -> Value {
    Value::struct_value(
        TYPE_OS_DIR_FS,
        vec![(DIR_FS_ROOT_FIELD.into(), Value::string(root))],
    )
}

pub(super) fn os_dirfs_value(root: &str) -> Option<Value> {
    normalize_os_path(root).map(|path| dirfs_value(&path))
}

pub(super) fn absolute_workspace_path(path: &str) -> String {
    if path == "." {
        "/".into()
    } else {
        format!("/{path}")
    }
}

pub(super) fn read_dirfs_root(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<String, VmError> {
    if value.typ != TYPE_OS_DIR_FS {
        return Err(invalid_fs_argument(vm, program, builtin));
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(invalid_fs_argument(vm, program, builtin));
    };
    fields
        .iter()
        .find(|(name, _)| name == DIR_FS_ROOT_FIELD)
        .and_then(|(_, value)| match &value.data {
            ValueData::String(root) => Some(root.clone()),
            _ => None,
        })
        .ok_or_else(|| invalid_fs_argument(vm, program, builtin))
}

pub(super) fn valid_path(name: &str) -> bool {
    if name == "." {
        return true;
    }
    if name.is_empty() || name.starts_with('/') || name.ends_with('/') {
        return false;
    }
    !name
        .split('/')
        .any(|element| element.is_empty() || element == "." || element == "..")
}

pub(super) fn read_workspace_file(
    vm: &Vm,
    root: Option<&str>,
    name: &str,
) -> Result<Vec<u8>, String> {
    let path =
        resolve_workspace_path(root, name).ok_or_else(|| format!("open {name}: invalid path"))?;
    read_normalized_workspace_file(vm, &path, name)
}

pub(super) fn read_os_workspace_file(vm: &Vm, name: &str) -> Result<Vec<u8>, String> {
    let path = normalize_os_path(name).ok_or_else(|| format!("open {name}: invalid path"))?;
    read_normalized_workspace_file(vm, &path, name)
}

pub(super) fn write_os_workspace_file(vm: &mut Vm, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = normalize_os_path(name).ok_or_else(|| format!("open {name}: invalid path"))?;
    write_normalized_workspace_file(vm, &path, name, bytes)
}

pub(super) fn open_workspace_file(
    vm: &mut Vm,
    root: Option<&str>,
    name: &str,
    source_is_os: bool,
) -> Result<Value, String> {
    let path = resolve_workspace_directory(root, name)
        .ok_or_else(|| format!("open {name}: invalid path"))?;
    open_normalized_workspace_file(vm, &path, name, source_is_os)
}

pub(super) fn open_os_workspace_file(
    vm: &mut Vm,
    name: &str,
    source_is_os: bool,
) -> Result<Value, String> {
    let path = normalize_os_path(name).ok_or_else(|| format!("open {name}: invalid path"))?;
    open_normalized_workspace_file(vm, &path, name, source_is_os)
}

fn open_normalized_workspace_file(
    vm: &mut Vm,
    path: &str,
    display_name: &str,
    source_is_os: bool,
) -> Result<Value, String> {
    let is_dir = if path != "." && vm.workspace_files.contains_key(path) {
        false
    } else if path == "." || workspace_has_directory(vm, path) {
        true
    } else {
        return Err(format!("open {display_name}: file does not exist"));
    };
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.workspace_fs_files.insert(
        id,
        crate::WorkspaceFsFileState {
            path: path.to_string(),
            closed: false,
            is_dir,
            source_is_os,
            read_offset: 0,
            read_dir_offset: 0,
        },
    );
    Ok(workspace_file_value(id))
}

pub(super) fn close_workspace_file(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let id = extract_workspace_file_id(vm, program, builtin, value)?;
    let Some(state) = vm.workspace_fs_files.get_mut(&id) else {
        return Err(invalid_file_argument(vm, program, builtin));
    };
    if state.closed {
        return Ok(error_value(format!(
            "close {}: file already closed",
            state.path
        )));
    }
    state.closed = true;
    Ok(Value::nil())
}

pub(super) fn stat_workspace_file(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<Value>, VmError> {
    let id = extract_workspace_file_id(vm, program, builtin, value)?;
    let Some(state) = vm.workspace_fs_files.get(&id) else {
        return Err(invalid_file_argument(vm, program, builtin));
    };
    if state.closed {
        return Ok(vec![
            Value::nil(),
            error_value(format!("stat {}: file already closed", state.path)),
        ]);
    }
    let exists = if state.is_dir {
        state.path == "." || workspace_has_directory(vm, &state.path)
    } else {
        vm.workspace_files.contains_key(&state.path)
    };
    if !exists {
        return Ok(vec![
            Value::nil(),
            error_value(format!("stat {}: file does not exist", state.path)),
        ]);
    }
    let name = state.path.rsplit('/').next().unwrap_or(&state.path);
    let size = if state.is_dir {
        0
    } else {
        workspace_file_size(vm, &state.path)
    };
    let source = if state.source_is_os {
        FILE_INFO_SOURCE_OS
    } else {
        FILE_INFO_SOURCE_FS
    };
    Ok(vec![
        file_info_value(name, state.is_dir, size as i64, 0, &state.path, source),
        Value::nil(),
    ])
}

pub(super) fn subdir_root(root: &str, dir: &str) -> Option<String> {
    if dir == "." {
        return Some(root.to_string());
    }
    if root.is_empty() || root == "." {
        return resolve_workspace_path(None, dir);
    }
    resolve_workspace_path(Some(root), dir)
}

pub(super) fn mkdir_all_os_workspace_path(vm: &mut Vm, name: &str) -> Result<(), String> {
    let path = normalize_os_path(name).ok_or_else(|| format!("mkdir {name}: invalid path"))?;
    mkdir_all_normalized_workspace_path(vm, &path, name)
}

fn mkdir_all_normalized_workspace_path(
    vm: &mut Vm,
    path: &str,
    display_name: &str,
) -> Result<(), String> {
    if path == "." {
        return Ok(());
    }

    let mut current = String::new();
    for segment in path.split('/') {
        if current.is_empty() {
            current.push_str(segment);
        } else {
            current.push('/');
            current.push_str(segment);
        }
        if vm.workspace_files.contains_key(&current) {
            return Err(format!("mkdir {display_name}: not a directory"));
        }
        vm.workspace_dirs.insert(current.clone());
    }

    Ok(())
}

pub(super) fn remove_all_os_workspace_path(vm: &mut Vm, name: &str) -> Result<(), String> {
    let path = normalize_os_path(name).ok_or_else(|| format!("removeall {name}: invalid path"))?;
    remove_all_normalized_workspace_path(vm, &path)
}

fn remove_all_normalized_workspace_path(vm: &mut Vm, path: &str) -> Result<(), String> {
    if path == "." {
        vm.workspace_files.clear();
        vm.workspace_dirs.clear();
        return Ok(());
    }

    let prefix = format!("{path}/");
    vm.workspace_files
        .retain(|candidate, _| candidate != path && !candidate.starts_with(&prefix));
    vm.workspace_dirs
        .retain(|candidate| candidate != path && !candidate.starts_with(&prefix));
    Ok(())
}

pub(super) fn glob_workspace_files(
    vm: &Vm,
    root: Option<&str>,
    pattern: &str,
) -> Result<Vec<String>, String> {
    let Some(candidates) = workspace_file_candidates(vm, root) else {
        return Err(format!("glob {pattern}: invalid path"));
    };

    let mut matches = Vec::new();
    for candidate in candidates {
        match super::path_impl::match_pattern(pattern, &candidate) {
            Ok(true) => matches.push(candidate),
            Ok(false) => {}
            Err(detail) => return Err(detail.to_string()),
        }
    }
    matches.sort();
    Ok(matches)
}

pub(super) fn read_workspace_dir_entries(
    vm: &Vm,
    root: Option<&str>,
    name: &str,
    source_is_os: bool,
) -> Result<Vec<Value>, String> {
    let directory = resolve_workspace_directory(root, name)
        .ok_or_else(|| format!("open {name}: invalid path"))?;

    let mut entries = std::collections::BTreeMap::new();
    let prefix = match directory.as_str() {
        "." => None,
        _ => Some(format!("{directory}/")),
    };

    for (path, contents) in &vm.workspace_files {
        let relative = match prefix.as_deref() {
            None => path.as_str(),
            Some(prefix) => match path.strip_prefix(prefix) {
                Some(stripped) => stripped,
                None => continue,
            },
        };
        if relative.is_empty() {
            continue;
        }
        let mut parts = relative.split('/');
        let entry_name = parts.next().unwrap_or_default();
        if entry_name.is_empty() {
            continue;
        }
        let is_dir = parts.next().is_some();
        entries
            .entry(entry_name.to_string())
            .and_modify(|(current_is_dir, current_size)| {
                if is_dir {
                    *current_is_dir = true;
                    *current_size = 0;
                } else if !*current_is_dir {
                    *current_size = contents.len();
                }
            })
            .or_insert((is_dir, if is_dir { 0 } else { contents.len() }));
    }

    for path in &vm.workspace_dirs {
        let relative = match prefix.as_deref() {
            None => path.as_str(),
            Some(prefix) => match path.strip_prefix(prefix) {
                Some(stripped) => stripped,
                None => continue,
            },
        };
        if relative.is_empty() {
            continue;
        }
        let mut parts = relative.split('/');
        let entry_name = parts.next().unwrap_or_default();
        if entry_name.is_empty() {
            continue;
        }
        entries
            .entry(entry_name.to_string())
            .and_modify(|(current_is_dir, current_size)| {
                *current_is_dir = true;
                *current_size = 0;
            })
            .or_insert((true, 0));
    }

    if entries.is_empty() {
        if directory != "." && vm.workspace_files.contains_key(&directory) {
            return Err(format!("readdir {name}: not implemented"));
        }
        if directory == "." || workspace_has_directory(vm, &directory) {
            return Ok(Vec::new());
        }
        return Err(format!("open {name}: file does not exist"));
    }

    Ok(entries
        .into_iter()
        .map(|(entry_name, (is_dir, size))| {
            let path = if directory == "." {
                entry_name.clone()
            } else {
                format!("{directory}/{entry_name}")
            };
            dir_entry_value(&entry_name, is_dir, size as i64, &path, source_is_os)
        })
        .collect())
}

pub(super) fn read_workspace_file_dir_entries(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
    n: i64,
) -> Result<Vec<Value>, VmError> {
    let id = extract_workspace_file_id(vm, program, builtin, value)?;
    let Some(state) = vm.workspace_fs_files.get(&id) else {
        return Err(invalid_file_argument(vm, program, builtin));
    };
    if state.closed {
        return Ok(vec![
            Value::nil_slice(),
            error_value(format!("readdir {}: file already closed", state.path)),
        ]);
    }
    if !state.is_dir {
        return Ok(vec![
            Value::nil_slice(),
            Value::error(format!("readdir {}: not implemented", state.path)),
        ]);
    }
    let path = state.path.clone();
    let source_is_os = state.source_is_os;
    let start = state.read_dir_offset;

    let values = match read_workspace_dir_entries(vm, None, &path, source_is_os) {
        Ok(entries) => entries,
        Err(error) => return Ok(vec![Value::nil_slice(), error_value(error)]),
    };
    let start = start.min(values.len());
    if n <= 0 {
        if let Some(state) = vm.workspace_fs_files.get_mut(&id) {
            state.read_dir_offset = values.len();
        }
        return Ok(vec![Value::slice(values[start..].to_vec()), Value::nil()]);
    }
    if start >= values.len() {
        return Ok(vec![Value::slice(Vec::new()), Value::error("EOF")]);
    }

    let end = (start + n as usize).min(values.len());
    if let Some(state) = vm.workspace_fs_files.get_mut(&id) {
        state.read_dir_offset = end;
    }
    Ok(vec![
        Value::slice(values[start..end].to_vec()),
        Value::nil(),
    ])
}

pub(super) fn bytes_to_value(bytes: &[u8]) -> Value {
    Value::slice(
        bytes
            .iter()
            .map(|byte| Value {
                typ: TYPE_INT,
                data: ValueData::Int(i64::from(*byte)),
            })
            .collect(),
    )
}

pub(super) fn extract_byte_slice(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a []byte result".into(),
        });
    };
    slice
        .values_snapshot()
        .iter()
        .map(|value| match value.data {
            ValueData::Int(number) if (0..=255).contains(&number) => Ok(number as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a []byte result".into(),
            }),
        })
        .collect()
}

pub(super) fn invalid_fs_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a filesystem value created by os.DirFS".into(),
    }
}

pub(super) fn invalid_file_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a file value created by fs.FS.Open".into(),
    }
}

pub(super) fn error_value(message: impl Into<String>) -> Value {
    let message = message.into();
    match known_error_contract(&message) {
        Some(contract) => {
            let wrapped = if contract.kind_message == "file does not exist" {
                Value::error(message.clone())
            } else {
                Value::error(contract.wrapped_message)
            };
            Value::wrapped_error_with_kind(message, contract.kind_message, wrapped)
        }
        None => Value::error(message),
    }
}

fn workspace_file_candidates(vm: &Vm, root: Option<&str>) -> Option<Vec<String>> {
    let root = root.unwrap_or(".");
    if !root.is_empty() && root != "." && !valid_path(root) {
        return None;
    }

    let prefix = if root.is_empty() || root == "." {
        None
    } else {
        Some(format!("{root}/"))
    };

    let mut candidates = std::collections::BTreeSet::new();
    for path in vm.workspace_files.keys() {
        match prefix.as_deref() {
            None => {
                candidates.insert(path.clone());
            }
            Some(prefix) => {
                if let Some(stripped) = path.strip_prefix(prefix) {
                    candidates.insert(stripped.to_string());
                }
            }
        }
    }
    for path in workspace_directory_paths(vm) {
        match prefix.as_deref() {
            None => {
                candidates.insert(path);
            }
            Some(prefix) => {
                if let Some(stripped) = path.strip_prefix(prefix) {
                    candidates.insert(stripped.to_string());
                }
            }
        }
    }
    Some(candidates.into_iter().collect())
}

fn workspace_has_directory(vm: &Vm, path: &str) -> bool {
    if path == "." {
        return true;
    }
    if vm.workspace_dirs.contains(path) {
        return true;
    }
    let prefix = format!("{path}/");
    vm.workspace_files
        .keys()
        .any(|candidate| candidate.starts_with(&prefix))
        || vm
            .workspace_dirs
            .iter()
            .any(|candidate| candidate.starts_with(&prefix))
}

fn workspace_directory_paths(vm: &Vm) -> std::collections::BTreeSet<String> {
    let mut paths = vm.workspace_dirs.clone();
    for path in vm.workspace_files.keys() {
        let mut current = String::new();
        let mut segments = path.split('/').peekable();
        while let Some(segment) = segments.next() {
            if segments.peek().is_none() {
                break;
            }
            if current.is_empty() {
                current.push_str(segment);
            } else {
                current.push('/');
                current.push_str(segment);
            }
            paths.insert(current.clone());
        }
    }
    paths
}

pub(super) fn normalize_os_path(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    let rooted = if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/{name}")
    };
    let cleaned = super::path_impl::clean(&rooted);
    if cleaned == "/" {
        return Some(".".into());
    }
    let path = cleaned.strip_prefix('/')?;
    if valid_path(path) {
        Some(path.to_string())
    } else {
        None
    }
}

fn resolve_workspace_directory(root: Option<&str>, name: &str) -> Option<String> {
    if name == "." {
        let root = root.unwrap_or(".");
        if root.is_empty() || root == "." {
            return Some(".".into());
        }
        if !valid_path(root) {
            return None;
        }
        return Some(root.to_string());
    }
    resolve_workspace_path(root, name)
}

fn resolve_workspace_path(root: Option<&str>, name: &str) -> Option<String> {
    if !valid_path(name) {
        return None;
    }
    let root = root.unwrap_or(".");
    if root.is_empty() || root == "." {
        return Some(name.to_string());
    }
    if !valid_path(root) {
        return None;
    }
    Some(format!("{root}/{name}"))
}

fn workspace_file_value(id: u64) -> Value {
    Value::struct_value(
        TYPE_FS_FILE,
        vec![(FILE_ID_FIELD.into(), Value::int(id as i64))],
    )
}

fn extract_workspace_file_id(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<u64, VmError> {
    if value.typ != TYPE_FS_FILE {
        return Err(invalid_file_argument(vm, program, builtin));
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(invalid_file_argument(vm, program, builtin));
    };
    fields
        .iter()
        .find(|(name, _)| name == FILE_ID_FIELD)
        .and_then(|(_, value)| match &value.data {
            ValueData::Int(id) if *id >= 0 => Some(*id as u64),
            _ => None,
        })
        .ok_or_else(|| invalid_file_argument(vm, program, builtin))
}

fn workspace_file_size(vm: &Vm, path: &str) -> usize {
    vm.workspace_files
        .get(path)
        .map(|contents| contents.len())
        .unwrap_or(0)
}

fn read_normalized_workspace_file(
    vm: &Vm,
    path: &str,
    display_name: &str,
) -> Result<Vec<u8>, String> {
    vm.workspace_files
        .get(path)
        .map(|contents| contents.as_bytes().to_vec())
        .ok_or_else(|| format!("open {display_name}: file does not exist"))
}

fn write_normalized_workspace_file(
    vm: &mut Vm,
    path: &str,
    display_name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    if path == "." || workspace_has_directory(vm, path) {
        return Err(format!("open {display_name}: is a directory"));
    }
    if let Some((parent, _)) = path.rsplit_once('/') {
        if vm.workspace_files.contains_key(parent) {
            return Err(format!("open {display_name}: not a directory"));
        }
        if !workspace_has_directory(vm, parent) {
            return Err(format!("open {display_name}: file does not exist"));
        }
    }
    vm.workspace_files.insert(
        path.to_string(),
        String::from_utf8_lossy(bytes).into_owned(),
    );
    Ok(())
}

struct ErrorContract {
    kind_message: &'static str,
    wrapped_message: &'static str,
}

fn known_error_contract(message: &str) -> Option<ErrorContract> {
    for message_kind in [
        "file does not exist",
        "file already exists",
        "permission denied",
        "file already closed",
    ] {
        if message == message_kind || message.ends_with(&format!(": {message_kind}")) {
            return Some(ErrorContract {
                kind_message: message_kind,
                wrapped_message: message_kind,
            });
        }
    }

    if message == "invalid path" || message.ends_with(": invalid path") {
        return Some(ErrorContract {
            kind_message: "invalid argument",
            wrapped_message: "invalid argument",
        });
    }

    None
}
