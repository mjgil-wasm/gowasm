use super::io_fs_impl::{io_fs_read_dir, io_fs_stat};
use crate::{describe_value, FunctionValue, Program, Value, ValueData, Vm, VmError};

pub(super) fn io_fs_walk_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let ValueData::String(root) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.WalkDir".into(),
            expected: "fs.FS, string, and callback arguments".into(),
        });
    };
    let ValueData::Function(callback) = &args[2].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[2]),
        });
    };
    walk_dir_function(vm, program, "fs.WalkDir", &args[0], root, callback)
}

pub(super) fn walk_dir_function(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    filesystem: &Value,
    root: &str,
    callback: &FunctionValue,
) -> Result<Value, VmError> {
    let stat_args = vec![filesystem.clone(), Value::string(root)];
    let stat_results = io_fs_stat(vm, program, &stat_args)?;
    match &stat_results[1].data {
        ValueData::Nil => {
            let root_entry = super::workspace_fs_impl::dir_entry_value_from_file_info(
                vm,
                program,
                builtin,
                &stat_results[0],
                root,
            )?;
            match walk_dir(vm, program, builtin, filesystem, root, root_entry, callback)? {
                WalkDirResult::Continue | WalkDirResult::SkipDir | WalkDirResult::SkipAll => {
                    Ok(Value::nil())
                }
                WalkDirResult::Error(err) => Ok(err),
            }
        }
        ValueData::Error(_) => match invoke_walk_dir_callback(
            vm,
            program,
            builtin,
            callback,
            root,
            Value::nil(),
            stat_results[1].clone(),
        )? {
            WalkDirResult::Continue | WalkDirResult::SkipDir | WalkDirResult::SkipAll => {
                Ok(Value::nil())
            }
            WalkDirResult::Error(err) => Ok(err),
        },
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a (fs.FileInfo, error) result".into(),
        }),
    }
}

pub(super) fn io_fs_file_info_to_dir_entry(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    if matches!(args[0].data, ValueData::Nil) {
        return Ok(Value::nil());
    }
    super::workspace_fs_impl::dir_entry_value_from_file_info(
        vm,
        program,
        "fs.FileInfoToDirEntry",
        &args[0],
        "",
    )
}

pub(super) fn io_fs_format_dir_entry(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let name = walk_dir_entry_name(vm, program, &args[0])?;
    let is_dir = walk_dir_entry_is_dir(vm, program, &args[0])?;
    let mode = walk_dir_entry_mode_string(vm, program, &args[0])?;
    let mut rendered = String::with_capacity(name.len() + 5);
    rendered.push_str(&mode[..mode.len().saturating_sub(9)]);
    rendered.push(' ');
    rendered.push_str(&name);
    if is_dir {
        rendered.push('/');
    }
    Ok(Value::string(rendered))
}

pub(super) fn io_fs_format_file_info(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let name = walk_file_info_name(vm, program, &args[0])?;
    let is_dir = walk_file_info_is_dir(vm, program, &args[0])?;
    let size = walk_file_info_size(vm, program, &args[0])?;
    let mode = walk_file_info_mode_string(vm, program, &args[0])?;
    let modified = walk_file_info_mod_time_string(vm, program, &args[0])?;
    let mut rendered = String::with_capacity(name.len() + mode.len() + modified.len() + 24);
    rendered.push_str(&mode);
    rendered.push(' ');
    rendered.push_str(&size.to_string());
    rendered.push(' ');
    rendered.push_str(&modified);
    rendered.push(' ');
    rendered.push_str(&name);
    if is_dir {
        rendered.push('/');
    }
    Ok(Value::string(rendered))
}

enum WalkDirResult {
    Continue,
    SkipDir,
    SkipAll,
    Error(Value),
}

fn walk_dir(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    filesystem: &Value,
    path: &str,
    entry: Value,
    callback: &FunctionValue,
) -> Result<WalkDirResult, VmError> {
    let is_dir = walk_dir_entry_is_dir(vm, program, &entry)?;
    match invoke_walk_dir_callback(
        vm,
        program,
        builtin,
        callback,
        path,
        entry.clone(),
        Value::nil(),
    )? {
        WalkDirResult::Continue => {}
        WalkDirResult::SkipDir if is_dir => return Ok(WalkDirResult::Continue),
        WalkDirResult::SkipDir => return Ok(WalkDirResult::SkipDir),
        WalkDirResult::SkipAll => return Ok(WalkDirResult::SkipAll),
        WalkDirResult::Error(err) => return Ok(WalkDirResult::Error(err)),
    }

    if !is_dir {
        return Ok(WalkDirResult::Continue);
    }

    let read_dir_args = vec![filesystem.clone(), Value::string(path)];
    let read_dir_results = io_fs_read_dir(vm, program, &read_dir_args)?;
    let (entries, read_dir_err) = normalize_walk_dir_entries(vm, program, &read_dir_results)?;
    if let Some(read_dir_err) = read_dir_err {
        match invoke_walk_dir_callback(vm, program, builtin, callback, path, entry, read_dir_err)? {
            WalkDirResult::Continue => {}
            WalkDirResult::SkipDir => return Ok(WalkDirResult::Continue),
            WalkDirResult::SkipAll => return Ok(WalkDirResult::SkipAll),
            WalkDirResult::Error(err) => return Ok(WalkDirResult::Error(err)),
        }
    }

    for child in entries {
        let child_name = walk_dir_entry_name(vm, program, &child)?;
        let child_path = join_walk_dir_path(path, &child_name);
        match walk_dir(
            vm,
            program,
            builtin,
            filesystem,
            &child_path,
            child,
            callback,
        )? {
            WalkDirResult::Continue => {}
            WalkDirResult::SkipDir => break,
            WalkDirResult::SkipAll => return Ok(WalkDirResult::SkipAll),
            WalkDirResult::Error(err) => return Ok(WalkDirResult::Error(err)),
        }
    }
    Ok(WalkDirResult::Continue)
}

fn invoke_walk_dir_callback(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    callback: &FunctionValue,
    path: &str,
    entry: Value,
    err: Value,
) -> Result<WalkDirResult, VmError> {
    let mut callback_args = callback.captures.clone();
    callback_args.push(Value::string(path));
    callback_args.push(entry);
    callback_args.push(err);
    let result = vm.invoke_callback(program, callback.function, callback_args)?;
    match &result.data {
        ValueData::Nil => Ok(WalkDirResult::Continue),
        ValueData::Error(error) => {
            if error.message == super::io_fs_registry_impl::SKIP_DIR {
                Ok(WalkDirResult::SkipDir)
            } else if error.message == super::io_fs_registry_impl::SKIP_ALL {
                Ok(WalkDirResult::SkipAll)
            } else {
                Ok(WalkDirResult::Error(result))
            }
        }
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: format!("{builtin} callback"),
            expected: "error return value".into(),
        }),
    }
}

fn normalize_walk_dir_entries(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<(Vec<Value>, Option<Value>), VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadDir".into(),
            expected: "a ([]fs.DirEntry, error) result".into(),
        });
    }
    let entries = match &results[0].data {
        ValueData::Slice(slice) => slice.values_snapshot(),
        ValueData::Nil => Vec::new(),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "fs.ReadDir".into(),
                expected: "a ([]fs.DirEntry, error) result".into(),
            });
        }
    };
    let err = match &results[1].data {
        ValueData::Nil => None,
        ValueData::Error(_) => Some(results[1].clone()),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "fs.ReadDir".into(),
                expected: "a ([]fs.DirEntry, error) result".into(),
            });
        }
    };
    Ok((entries, err))
}

fn walk_dir_entry_name(vm: &mut Vm, program: &Program, entry: &Value) -> Result<String, VmError> {
    match vm
        .invoke_method(program, entry.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => Ok(name),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.WalkDir".into(),
            expected: "fs.DirEntry.Name to return string".into(),
        }),
    }
}

fn walk_file_info_name(vm: &mut Vm, program: &Program, info: &Value) -> Result<String, VmError> {
    match vm
        .invoke_method(program, info.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => Ok(name),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatFileInfo".into(),
            expected: "fs.FileInfo.Name to return string".into(),
        }),
    }
}

fn walk_file_info_is_dir(vm: &mut Vm, program: &Program, info: &Value) -> Result<bool, VmError> {
    match vm
        .invoke_method(program, info.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => Ok(is_dir),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatFileInfo".into(),
            expected: "fs.FileInfo.IsDir to return bool".into(),
        }),
    }
}

fn walk_file_info_size(vm: &mut Vm, program: &Program, info: &Value) -> Result<i64, VmError> {
    match vm
        .invoke_method(program, info.clone(), "Size", Vec::new())?
        .data
    {
        ValueData::Int(size) => Ok(size),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatFileInfo".into(),
            expected: "fs.FileInfo.Size to return int".into(),
        }),
    }
}

fn walk_file_info_mode_string(
    vm: &mut Vm,
    program: &Program,
    info: &Value,
) -> Result<String, VmError> {
    let mode = vm.invoke_method(program, info.clone(), "Mode", Vec::new())?;
    match vm.invoke_method(program, mode, "String", Vec::new())?.data {
        ValueData::String(rendered) => Ok(rendered),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatFileInfo".into(),
            expected: "fs.FileMode.String to return string".into(),
        }),
    }
}

fn walk_file_info_mod_time_string(
    vm: &mut Vm,
    program: &Program,
    info: &Value,
) -> Result<String, VmError> {
    let modified = vm.invoke_method(program, info.clone(), "ModTime", Vec::new())?;
    match vm
        .invoke_method(
            program,
            modified,
            "Format",
            vec![Value::string(super::time_format_impl::DATE_TIME_LAYOUT)],
        )?
        .data
    {
        ValueData::String(rendered) => Ok(rendered),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatFileInfo".into(),
            expected: "time.Time.Format to return string".into(),
        }),
    }
}

fn walk_dir_entry_is_dir(vm: &mut Vm, program: &Program, entry: &Value) -> Result<bool, VmError> {
    match vm
        .invoke_method(program, entry.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => Ok(is_dir),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.WalkDir".into(),
            expected: "fs.DirEntry.IsDir to return bool".into(),
        }),
    }
}

fn walk_dir_entry_mode_string(
    vm: &mut Vm,
    program: &Program,
    entry: &Value,
) -> Result<String, VmError> {
    let mode = vm.invoke_method(program, entry.clone(), "Type", Vec::new())?;
    match vm.invoke_method(program, mode, "String", Vec::new())?.data {
        ValueData::String(rendered) => Ok(rendered),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FormatDirEntry".into(),
            expected: "fs.FileMode.String to return string".into(),
        }),
    }
}

fn join_walk_dir_path(parent: &str, name: &str) -> String {
    if parent == "." {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}
