use crate::{
    Program, Value, ValueData, Vm, VmError, TYPE_FS_DIR_ENTRY, TYPE_FS_FILE_INFO, TYPE_FS_FILE_MODE,
};

const FILE_INFO_NAME_FIELD: &str = "__fs_file_info_name";
const FILE_INFO_IS_DIR_FIELD: &str = "__fs_file_info_is_dir";
const FILE_INFO_SIZE_FIELD: &str = "__fs_file_info_size";
const FILE_INFO_MOD_TIME_FIELD: &str = "__fs_file_info_mod_time";
const FILE_INFO_PATH_FIELD: &str = "__fs_file_info_path";
const FILE_INFO_SYS_FIELD: &str = "__fs_file_info_sys";
const FILE_INFO_SOURCE_FIELD: &str = "__fs_file_info_source";
const DIR_ENTRY_INFO_FIELD: &str = "__fs_dir_entry_info";
const DIR_ENTRY_SOURCE_FIELD: &str = "__fs_dir_entry_source_os";

pub(super) const FILE_INFO_SOURCE_FS: &str = "fs";
pub(super) const FILE_INFO_SOURCE_OS: &str = "os";

pub(super) fn file_info_name(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<String, VmError> {
    let (name, _, _, _) = extract_file_info_fields(vm, program, builtin, value)?;
    Ok(name)
}

pub(super) fn file_info_is_dir(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<bool, VmError> {
    let (_, is_dir, _, _) = extract_file_info_fields(vm, program, builtin, value)?;
    Ok(is_dir)
}

pub(super) fn file_info_size(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<i64, VmError> {
    let (_, _, size, _) = extract_file_info_fields(vm, program, builtin, value)?;
    Ok(size)
}

pub(super) fn file_info_mode(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let (_, is_dir, _, _) = extract_file_info_fields(vm, program, builtin, value)?;
    Ok(file_mode_value(file_mode_bits(is_dir)))
}

pub(super) fn file_info_mod_time(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let (_, _, _, mod_time_unix_nanos) = extract_file_info_fields(vm, program, builtin, value)?;
    Ok(super::time_impl::time_value(mod_time_unix_nanos))
}

pub(super) fn file_info_sys(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    if value.typ == TYPE_FS_FILE_INFO {
        let ValueData::Struct(fields) = &value.data else {
            return Err(invalid_file_info_argument(vm, program, builtin));
        };
        let _ = extract_file_info_fields(vm, program, builtin, value)?;
        return Ok(fields
            .iter()
            .find_map(|(field, value)| {
                if field == FILE_INFO_SYS_FIELD {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(Value::nil));
    }

    let sys = vm.invoke_method(program, value.clone(), "Sys", Vec::new())?;
    let _ = extract_dynamic_file_info_fields(vm, program, builtin, value)?;
    Ok(sys)
}

pub(super) fn file_mode_is_dir(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<bool, VmError> {
    let mode = extract_file_mode_bits(vm, program, builtin, value)?;
    Ok(mode & super::io_fs_registry_impl::MODE_DIR != 0)
}

pub(super) fn file_mode_is_regular(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<bool, VmError> {
    let mode = extract_file_mode_bits(vm, program, builtin, value)?;
    Ok(mode & super::io_fs_registry_impl::MODE_TYPE == 0)
}

pub(super) fn file_mode_type(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let mode = extract_file_mode_bits(vm, program, builtin, value)?;
    Ok(file_mode_value(
        mode & super::io_fs_registry_impl::MODE_TYPE,
    ))
}

pub(super) fn file_mode_string(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<String, VmError> {
    const TYPE_BITS: &str = "dalTLDpSugct?";
    const RWX_BITS: &str = "rwxrwxrwx";

    let mode = extract_file_mode_bits(vm, program, builtin, value)? as u32;
    let mut output = String::with_capacity(10);

    for (index, ch) in TYPE_BITS.chars().enumerate() {
        if mode & (1_u32 << (31 - index)) != 0 {
            output.push(ch);
        }
    }
    if output.is_empty() {
        output.push('-');
    }

    for (index, ch) in RWX_BITS.chars().enumerate() {
        if mode & (1_u32 << (8 - index)) != 0 {
            output.push(ch);
        } else {
            output.push('-');
        }
    }

    Ok(output)
}

pub(super) fn file_mode_perm(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let mode = extract_file_mode_bits(vm, program, builtin, value)?;
    Ok(file_mode_value(
        mode & super::io_fs_registry_impl::MODE_PERM,
    ))
}

pub(super) fn dir_entry_name(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<String, VmError> {
    let (name, _, _, _) = extract_dir_entry_fields(vm, program, builtin, value)?;
    Ok(name)
}

pub(super) fn dir_entry_is_dir(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<bool, VmError> {
    let (_, is_dir, _, _) = extract_dir_entry_fields(vm, program, builtin, value)?;
    Ok(is_dir)
}

pub(super) fn dir_entry_type(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    if let Some(info) = dir_entry_original_info(value) {
        let mode = vm.invoke_method(program, info, "Mode", Vec::new())?;
        return vm.invoke_method(program, mode, "Type", Vec::new());
    }
    if value.typ != TYPE_FS_DIR_ENTRY {
        let mode = vm.invoke_method(program, value.clone(), "Type", Vec::new())?;
        let _ = extract_dynamic_dir_entry_fields(vm, program, builtin, value)?;
        return Ok(mode);
    }
    let (_, is_dir, _, _) = extract_dir_entry_fields(vm, program, builtin, value)?;
    Ok(file_mode_value(file_mode_bits(is_dir)))
}

pub(super) fn dir_entry_info(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<Value>, VmError> {
    if let Some(info) = dir_entry_original_info(value) {
        return Ok(vec![info, Value::nil()]);
    }
    if value.typ != TYPE_FS_DIR_ENTRY {
        let info = vm.invoke_method_results(program, value.clone(), "Info", Vec::new())?;
        if info.len() != 2 {
            return Err(invalid_dir_entry_argument(vm, program, builtin));
        }
        match &info[1].data {
            ValueData::Nil | ValueData::Error(_) => return Ok(info),
            _ => return Err(invalid_dir_entry_argument(vm, program, builtin)),
        }
    }
    let (name, is_dir, size, path) = extract_dir_entry_fields(vm, program, builtin, value)?;
    let source = if dir_entry_source_is_os(value) {
        FILE_INFO_SOURCE_OS
    } else {
        FILE_INFO_SOURCE_FS
    };
    Ok(vec![
        file_info_value(&name, is_dir, size, 0, &path, source),
        Value::nil(),
    ])
}

pub(super) fn same_file_workspace_path(value: &Value) -> Option<String> {
    if value.typ != TYPE_FS_FILE_INFO {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };

    let source = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_SOURCE_FIELD {
            return None;
        }
        match &value.data {
            ValueData::String(source) => Some(source.as_str()),
            _ => None,
        }
    })?;
    if source != FILE_INFO_SOURCE_OS {
        return None;
    }

    fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_PATH_FIELD {
            return None;
        }
        match &value.data {
            ValueData::String(path) => Some(path.clone()),
            _ => None,
        }
    })
}

pub(super) fn file_info_value(
    name: &str,
    is_dir: bool,
    size: i64,
    mod_time_unix_nanos: i64,
    path: &str,
    source: &str,
) -> Value {
    Value::struct_value(
        TYPE_FS_FILE_INFO,
        vec![
            (FILE_INFO_NAME_FIELD.into(), Value::string(name)),
            (FILE_INFO_IS_DIR_FIELD.into(), Value::bool(is_dir)),
            (FILE_INFO_SIZE_FIELD.into(), Value::int(size)),
            (
                FILE_INFO_MOD_TIME_FIELD.into(),
                Value::int(mod_time_unix_nanos),
            ),
            (FILE_INFO_PATH_FIELD.into(), Value::string(path)),
            (FILE_INFO_SYS_FIELD.into(), Value::nil()),
            (FILE_INFO_SOURCE_FIELD.into(), Value::string(source)),
        ],
    )
}

pub(super) fn dir_entry_value(
    name: &str,
    is_dir: bool,
    size: i64,
    path: &str,
    source_is_os: bool,
) -> Value {
    Value::struct_value(
        TYPE_FS_DIR_ENTRY,
        vec![
            (FILE_INFO_NAME_FIELD.into(), Value::string(name)),
            (FILE_INFO_IS_DIR_FIELD.into(), Value::bool(is_dir)),
            (FILE_INFO_SIZE_FIELD.into(), Value::int(size)),
            (FILE_INFO_PATH_FIELD.into(), Value::string(path)),
            (DIR_ENTRY_SOURCE_FIELD.into(), Value::bool(source_is_os)),
        ],
    )
}

pub(super) fn dir_entry_value_from_file_info(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    info: &Value,
    path: &str,
) -> Result<Value, VmError> {
    let name = match vm
        .invoke_method(program, info.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => name,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let is_dir = match vm
        .invoke_method(program, info.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => is_dir,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let size = match vm
        .invoke_method(program, info.clone(), "Size", Vec::new())?
        .data
    {
        ValueData::Int(size) if size >= 0 => size,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let mod_time = vm.invoke_method(program, info.clone(), "ModTime", Vec::new())?;
    let _mod_time_unix_nanos =
        super::time_impl::extract_time_unix_nanos(vm, program, builtin, &mod_time)
            .map_err(|_| invalid_file_info_argument(vm, program, builtin))?;
    let stored_path = if path.is_empty() {
        name.clone()
    } else {
        path.to_string()
    };
    Ok(Value::struct_value(
        TYPE_FS_DIR_ENTRY,
        vec![
            (FILE_INFO_NAME_FIELD.into(), Value::string(name)),
            (FILE_INFO_IS_DIR_FIELD.into(), Value::bool(is_dir)),
            (FILE_INFO_SIZE_FIELD.into(), Value::int(size)),
            (FILE_INFO_PATH_FIELD.into(), Value::string(stored_path)),
            (DIR_ENTRY_INFO_FIELD.into(), info.clone()),
        ],
    ))
}

fn invalid_file_info_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a fs.FileInfo value".into(),
    }
}

fn invalid_file_mode_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a fs.FileMode value".into(),
    }
}

fn invalid_dir_entry_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a fs.DirEntry value".into(),
    }
}

fn file_mode_value(bits: i64) -> Value {
    Value {
        typ: TYPE_FS_FILE_MODE,
        data: ValueData::Int(bits),
    }
}

fn extract_file_mode_bits(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<i64, VmError> {
    if value.typ != TYPE_FS_FILE_MODE {
        return Err(invalid_file_mode_argument(vm, program, builtin));
    }
    let ValueData::Int(bits) = value.data else {
        return Err(invalid_file_mode_argument(vm, program, builtin));
    };
    Ok(bits)
}

fn file_mode_bits(is_dir: bool) -> i64 {
    if is_dir {
        super::io_fs_registry_impl::MODE_DIR
    } else {
        0
    }
}

fn dir_entry_original_info(value: &Value) -> Option<Value> {
    if value.typ != TYPE_FS_DIR_ENTRY {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    fields.iter().find_map(|(field, value)| {
        if field == DIR_ENTRY_INFO_FIELD {
            Some(value.clone())
        } else {
            None
        }
    })
}

fn dir_entry_source_is_os(value: &Value) -> bool {
    if value.typ != TYPE_FS_DIR_ENTRY {
        return false;
    }
    let ValueData::Struct(fields) = &value.data else {
        return false;
    };
    fields
        .iter()
        .find_map(|(field, value)| {
            if field != DIR_ENTRY_SOURCE_FIELD {
                return None;
            }
            match &value.data {
                ValueData::Bool(source_is_os) => Some(*source_is_os),
                _ => None,
            }
        })
        .unwrap_or(false)
}

fn extract_file_info_fields(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<(String, bool, i64, i64), VmError> {
    if value.typ != TYPE_FS_FILE_INFO {
        return extract_dynamic_file_info_fields(vm, program, builtin, value);
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(invalid_file_info_argument(vm, program, builtin));
    };

    let Some(name) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_NAME_FIELD {
            return None;
        }
        match &value.data {
            ValueData::String(name) => Some(name.clone()),
            _ => None,
        }
    }) else {
        return Err(invalid_file_info_argument(vm, program, builtin));
    };

    let Some(is_dir) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_IS_DIR_FIELD {
            return None;
        }
        match &value.data {
            ValueData::Bool(is_dir) => Some(*is_dir),
            _ => None,
        }
    }) else {
        return Err(invalid_file_info_argument(vm, program, builtin));
    };

    let Some(size) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_SIZE_FIELD {
            return None;
        }
        match &value.data {
            ValueData::Int(size) if *size >= 0 => Some(*size),
            _ => None,
        }
    }) else {
        return Err(invalid_file_info_argument(vm, program, builtin));
    };

    let Some(mod_time_unix_nanos) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_MOD_TIME_FIELD {
            return None;
        }
        match &value.data {
            ValueData::Int(mod_time_unix_nanos) => Some(*mod_time_unix_nanos),
            _ => None,
        }
    }) else {
        return Err(invalid_file_info_argument(vm, program, builtin));
    };

    Ok((name, is_dir, size, mod_time_unix_nanos))
}

fn extract_dynamic_file_info_fields(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<(String, bool, i64, i64), VmError> {
    let name = match vm
        .invoke_method(program, value.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => name,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let is_dir = match vm
        .invoke_method(program, value.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => is_dir,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let size = match vm
        .invoke_method(program, value.clone(), "Size", Vec::new())?
        .data
    {
        ValueData::Int(size) if size >= 0 => size,
        _ => return Err(invalid_file_info_argument(vm, program, builtin)),
    };
    let mod_time = vm.invoke_method(program, value.clone(), "ModTime", Vec::new())?;
    let mod_time_unix_nanos =
        super::time_impl::extract_time_unix_nanos(vm, program, builtin, &mod_time)
            .map_err(|_| invalid_file_info_argument(vm, program, builtin))?;
    Ok((name, is_dir, size, mod_time_unix_nanos))
}

fn extract_dir_entry_fields(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<(String, bool, i64, String), VmError> {
    if value.typ != TYPE_FS_DIR_ENTRY {
        return extract_dynamic_dir_entry_fields(vm, program, builtin, value);
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(invalid_dir_entry_argument(vm, program, builtin));
    };

    let Some(name) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_NAME_FIELD {
            return None;
        }
        match &value.data {
            ValueData::String(name) => Some(name.clone()),
            _ => None,
        }
    }) else {
        return Err(invalid_dir_entry_argument(vm, program, builtin));
    };

    let Some(is_dir) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_IS_DIR_FIELD {
            return None;
        }
        match &value.data {
            ValueData::Bool(is_dir) => Some(*is_dir),
            _ => None,
        }
    }) else {
        return Err(invalid_dir_entry_argument(vm, program, builtin));
    };

    let Some(size) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_SIZE_FIELD {
            return None;
        }
        match &value.data {
            ValueData::Int(size) if *size >= 0 => Some(*size),
            _ => None,
        }
    }) else {
        return Err(invalid_dir_entry_argument(vm, program, builtin));
    };

    let Some(path) = fields.iter().find_map(|(field, value)| {
        if field != FILE_INFO_PATH_FIELD {
            return None;
        }
        match &value.data {
            ValueData::String(path) => Some(path.clone()),
            _ => None,
        }
    }) else {
        return Err(invalid_dir_entry_argument(vm, program, builtin));
    };

    Ok((name, is_dir, size, path))
}

fn extract_dynamic_dir_entry_fields(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<(String, bool, i64, String), VmError> {
    let name = match vm
        .invoke_method(program, value.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => name,
        _ => return Err(invalid_dir_entry_argument(vm, program, builtin)),
    };
    let is_dir = match vm
        .invoke_method(program, value.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => is_dir,
        _ => return Err(invalid_dir_entry_argument(vm, program, builtin)),
    };
    let size = match vm.invoke_method_results(program, value.clone(), "Info", Vec::new())? {
        results if results.len() == 2 => match (&results[0], &results[1].data) {
            (info, ValueData::Nil) => extract_file_info_fields(vm, program, builtin, info)?.2,
            (_, ValueData::Error(_)) => 0,
            _ => return Err(invalid_dir_entry_argument(vm, program, builtin)),
        },
        _ => return Err(invalid_dir_entry_argument(vm, program, builtin)),
    };
    Ok((name.clone(), is_dir, size, name))
}
