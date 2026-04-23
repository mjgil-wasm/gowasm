use crate::{Program, Value, ValueData, Vm, VmError, TYPE_FS_SUB_FS};

const SUB_FS_BASE_FIELD: &str = "__subfs_base";
const SUB_FS_PREFIX_FIELD: &str = "__subfs_prefix";
const READ_BUFFER_SIZE: usize = 8 * 1024;

pub(super) fn subfs_path_call(
    filesystem: &Value,
    name: &str,
) -> Result<Option<(Value, String)>, String> {
    let Some((base, prefix)) = read_subfs(filesystem) else {
        return Ok(None);
    };
    let path = super::workspace_fs_impl::subdir_root(&prefix, name)
        .ok_or_else(|| format!("open {name}: invalid path"))?;
    Ok(Some((base, path)))
}

pub(super) fn subfs_glob_call(filesystem: &Value, pattern: &str) -> Option<(Value, String)> {
    let (base, prefix) = read_subfs(filesystem)?;
    Some((base, join_subfs_pattern(&prefix, pattern)))
}

pub(super) fn subfs_value(filesystem: &Value, dir: &str) -> Option<Value> {
    let (base, root, original_wrapper) = match read_subfs(filesystem) {
        Some((base, root)) => (base, root, true),
        None => (filesystem.clone(), ".".to_string(), false),
    };
    let prefix = super::workspace_fs_impl::subdir_root(&root, dir)?;
    if prefix == root && original_wrapper {
        return Some(filesystem.clone());
    }
    if prefix == "." {
        return Some(base);
    }
    Some(Value::struct_value(
        TYPE_FS_SUB_FS,
        vec![
            (SUB_FS_BASE_FIELD.into(), base),
            (SUB_FS_PREFIX_FIELD.into(), Value::string(prefix)),
        ],
    ))
}

pub(super) fn read_file_via_open(
    vm: &mut Vm,
    program: &Program,
    filesystem: &Value,
    name: &str,
) -> Result<Vec<Value>, VmError> {
    let open_results = match vm.invoke_method_results(
        program,
        filesystem.clone(),
        "Open",
        vec![Value::string(name.to_string())],
    ) {
        Ok(results) => results,
        Err(VmError::UnknownMethod { .. }) => {
            return Ok(vec![
                Value::nil_slice(),
                Value::error("fs.ReadFile: unsupported fs.FS implementation"),
            ]);
        }
        Err(error) => return Err(error),
    };
    let Some(file) = normalize_open_results(vm, program, "fs.ReadFile", &open_results)? else {
        return Ok(vec![Value::nil_slice(), open_results[1].clone()]);
    };

    let read_result = read_all_file_bytes(vm, program, &file);
    let close_result = vm.invoke_method(program, file, "Close", Vec::new());
    match (read_result, close_result) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(result), Ok(_)) => Ok(result),
    }
}

pub(super) fn glob_via_read_dir(
    vm: &mut Vm,
    program: &Program,
    filesystem: &Value,
    pattern: &str,
) -> Result<Vec<Value>, VmError> {
    let mut candidates = Vec::new();
    if let Some(error) = collect_glob_candidates(vm, program, filesystem, ".", &mut candidates)? {
        return Ok(vec![Value::nil_slice(), error]);
    }

    let mut matches = Vec::new();
    for candidate in candidates {
        match super::path_impl::match_pattern(pattern, &candidate) {
            Ok(true) => matches.push(candidate),
            Ok(false) => {}
            Err(detail) => return Ok(vec![Value::nil_slice(), Value::error(detail)]),
        }
    }
    matches.sort();
    Ok(vec![string_slice_value(&matches), Value::nil()])
}

fn read_subfs(value: &Value) -> Option<(Value, String)> {
    if value.typ != TYPE_FS_SUB_FS {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    let base = fields
        .iter()
        .find(|(name, _)| name == SUB_FS_BASE_FIELD)
        .map(|(_, value)| value.clone())?;
    let prefix = fields
        .iter()
        .find(|(name, _)| name == SUB_FS_PREFIX_FIELD)
        .and_then(|(_, value)| match &value.data {
            ValueData::String(prefix) => Some(prefix.clone()),
            _ => None,
        })?;
    Some((base, prefix))
}

fn join_subfs_pattern(prefix: &str, pattern: &str) -> String {
    if prefix.is_empty() || prefix == "." {
        pattern.to_string()
    } else if pattern.is_empty() || pattern == "." {
        prefix.to_string()
    } else {
        format!("{prefix}/{pattern}")
    }
}

fn normalize_open_results(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    results: &[Value],
) -> Result<Option<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a (fs.File, error) result".into(),
        });
    }
    match &results[1].data {
        ValueData::Nil if matches!(results[0].data, ValueData::Nil) => {
            Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a (fs.File, error) result".into(),
            })
        }
        ValueData::Nil => Ok(Some(results[0].clone())),
        ValueData::Error(_) => Ok(None),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a (fs.File, error) result".into(),
        }),
    }
}

fn read_all_file_bytes(
    vm: &mut Vm,
    program: &Program,
    file: &Value,
) -> Result<Vec<Value>, VmError> {
    let mut bytes = Vec::new();
    loop {
        let (results, buffer) = vm.invoke_method_results_mutating_arg(
            program,
            file.clone(),
            "Read",
            vec![Value::slice(vec![Value::int(0); READ_BUFFER_SIZE])],
            0,
        )?;
        let (read_count, read_error) = normalize_file_read(vm, program, &buffer, &results)?;
        if read_count > 0 {
            bytes.extend(read_buffer_prefix(vm, program, &buffer, read_count)?);
        }
        match read_error {
            Some(error) if is_eof_error(&error) => {
                return Ok(vec![
                    super::workspace_fs_impl::bytes_to_value(&bytes),
                    Value::nil(),
                ]);
            }
            Some(error) => {
                return Ok(vec![
                    super::workspace_fs_impl::bytes_to_value(&bytes),
                    error,
                ]);
            }
            None if read_count == 0 => {
                return Ok(vec![
                    super::workspace_fs_impl::bytes_to_value(&bytes),
                    Value::nil(),
                ]);
            }
            None => {}
        }
    }
}

fn normalize_file_read(
    vm: &Vm,
    program: &Program,
    buffer: &Value,
    results: &[Value],
) -> Result<(usize, Option<Value>), VmError> {
    if results.len() != 2 {
        return Err(invalid_file_read(vm, program));
    }

    let buffer_len = match &buffer.data {
        ValueData::Slice(slice) => slice.len(),
        _ => return Err(invalid_file_read(vm, program)),
    };

    let ValueData::Int(read_count) = &results[0].data else {
        return Err(invalid_file_read(vm, program));
    };
    if *read_count < 0 || *read_count as usize > buffer_len {
        return Err(invalid_file_read(vm, program));
    }

    let read_error = match &results[1].data {
        ValueData::Nil => None,
        ValueData::Error(_) => Some(results[1].clone()),
        _ => return Err(invalid_file_read(vm, program)),
    };

    Ok((*read_count as usize, read_error))
}

fn read_buffer_prefix(
    vm: &Vm,
    program: &Program,
    buffer: &Value,
    read_count: usize,
) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &buffer.data else {
        return Err(invalid_file_read(vm, program));
    };

    slice
        .values_snapshot()
        .iter()
        .take(read_count)
        .map(|value| match value.data {
            ValueData::Int(number) if (0..=255).contains(&number) => Ok(number as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "fs.ReadFile".into(),
                expected: "a file reader writing into a []byte buffer".into(),
            }),
        })
        .collect()
}

fn invalid_file_read(vm: &Vm, program: &Program) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: "fs.ReadFile".into(),
        expected: "a file reader returning (int, error)".into(),
    }
}

fn is_eof_error(value: &Value) -> bool {
    matches!(&value.data, ValueData::Error(error) if error.message == "EOF")
}

fn collect_glob_candidates(
    vm: &mut Vm,
    program: &Program,
    filesystem: &Value,
    path: &str,
    candidates: &mut Vec<String>,
) -> Result<Option<Value>, VmError> {
    let stat_results = super::io_fs_impl::io_fs_stat(
        vm,
        program,
        &[filesystem.clone(), Value::string(path.to_string())],
    )?;
    match &stat_results[1].data {
        ValueData::Nil => {}
        ValueData::Error(_) => return Ok(Some(stat_results[1].clone())),
        _ => return Err(invalid_glob_result(vm, program)),
    }

    let is_dir = file_info_is_dir(vm, program, &stat_results[0])?;
    if path != "." {
        candidates.push(path.to_string());
    }
    if !is_dir {
        return Ok(None);
    }

    let read_dir_results = super::io_fs_impl::io_fs_read_dir(
        vm,
        program,
        &[filesystem.clone(), Value::string(path.to_string())],
    )?;
    let entries = match &read_dir_results[0].data {
        ValueData::Slice(slice) => slice.values_snapshot(),
        ValueData::Nil => Vec::new(),
        _ => return Err(invalid_glob_result(vm, program)),
    };
    match &read_dir_results[1].data {
        ValueData::Nil => {}
        ValueData::Error(_) => return Ok(Some(read_dir_results[1].clone())),
        _ => return Err(invalid_glob_result(vm, program)),
    }

    for entry in entries {
        let name = dir_entry_name(vm, program, &entry)?;
        let child_path = if path == "." {
            name
        } else {
            format!("{path}/{name}")
        };
        if let Some(error) =
            collect_glob_candidates(vm, program, filesystem, &child_path, candidates)?
        {
            return Ok(Some(error));
        }
    }
    Ok(None)
}

fn file_info_is_dir(vm: &mut Vm, program: &Program, info: &Value) -> Result<bool, VmError> {
    match vm
        .invoke_method(program, info.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => Ok(is_dir),
        _ => Err(invalid_glob_result(vm, program)),
    }
}

fn dir_entry_name(vm: &mut Vm, program: &Program, entry: &Value) -> Result<String, VmError> {
    match vm
        .invoke_method(program, entry.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => Ok(name),
        _ => Err(invalid_glob_result(vm, program)),
    }
}

fn invalid_glob_result(vm: &Vm, program: &Program) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: "fs.Glob".into(),
        expected: "filesystem helpers returning io/fs-compatible results".into(),
    }
}

fn string_slice_value(values: &[String]) -> Value {
    if values.is_empty() {
        return Value::nil_slice();
    }
    Value::slice(
        values
            .iter()
            .map(|value| Value::string(value.clone()))
            .collect(),
    )
}
