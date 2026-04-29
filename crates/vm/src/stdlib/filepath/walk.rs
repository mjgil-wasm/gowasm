use crate::{describe_value, FunctionValue, Program, Value, ValueData, Vm, VmError};

pub(crate) fn filepath_walk_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(root) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.WalkDir".into(),
            expected: "string and callback arguments".into(),
        });
    };
    let ValueData::Function(callback) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[1]),
        });
    };
    let (root, absolute_output) = normalize_filepath_walk_root(root);
    let filesystem = super::super::workspace_fs_impl::dirfs_value(".");
    let stat_results = super::super::io_fs_impl::io_fs_stat(
        vm,
        program,
        &[filesystem.clone(), Value::string(&root)],
    )?;
    match &stat_results[1].data {
        ValueData::Nil => {
            let root_entry = super::super::workspace_fs_impl::dir_entry_value_from_file_info(
                vm,
                program,
                "filepath.WalkDir",
                &stat_results[0],
                &root,
            )?;
            match filepath_walk_dir_path(
                vm,
                program,
                &filesystem,
                &root,
                root_entry,
                callback,
                absolute_output,
            )? {
                FilepathWalkResult::Continue
                | FilepathWalkResult::SkipDir
                | FilepathWalkResult::SkipAll => Ok(Value::nil()),
                FilepathWalkResult::Error(err) => Ok(err),
            }
        }
        ValueData::Error(_) => match invoke_filepath_walk_dir_callback(
            vm,
            program,
            callback,
            &root,
            Value::nil(),
            stat_results[1].clone(),
            absolute_output,
        )? {
            FilepathWalkResult::Continue
            | FilepathWalkResult::SkipDir
            | FilepathWalkResult::SkipAll => Ok(Value::nil()),
            FilepathWalkResult::Error(err) => Ok(err),
        },
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.WalkDir".into(),
            expected: "a (fs.FileInfo, error) result".into(),
        }),
    }
}

pub(crate) fn filepath_walk(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(root) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "string and callback arguments".into(),
        });
    };
    let ValueData::Function(callback) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[1]),
        });
    };

    let (root, absolute_output) = normalize_filepath_walk_root(root);
    let filesystem = super::super::workspace_fs_impl::dirfs_value(".");
    let stat_results = super::super::io_fs_impl::io_fs_stat(
        vm,
        program,
        &[filesystem.clone(), Value::string(&root)],
    )?;
    match &stat_results[1].data {
        ValueData::Nil => match filepath_walk_path(
            vm,
            program,
            &filesystem,
            &root,
            stat_results[0].clone(),
            callback,
            absolute_output,
        )? {
            FilepathWalkResult::Continue
            | FilepathWalkResult::SkipDir
            | FilepathWalkResult::SkipAll => Ok(Value::nil()),
            FilepathWalkResult::Error(err) => Ok(err),
        },
        ValueData::Error(_) => match invoke_filepath_walk_callback(
            vm,
            program,
            callback,
            &root,
            Value::nil(),
            stat_results[1].clone(),
            absolute_output,
        )? {
            FilepathWalkResult::Continue
            | FilepathWalkResult::SkipDir
            | FilepathWalkResult::SkipAll => Ok(Value::nil()),
            FilepathWalkResult::Error(err) => Ok(err),
        },
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "a (fs.FileInfo, error) result".into(),
        }),
    }
}

enum FilepathWalkResult {
    Continue,
    SkipDir,
    SkipAll,
    Error(Value),
}

fn filepath_walk_path(
    vm: &mut Vm,
    program: &Program,
    filesystem: &Value,
    path: &str,
    info: Value,
    callback: &FunctionValue,
    absolute_output: bool,
) -> Result<FilepathWalkResult, VmError> {
    let is_dir = filepath_walk_file_info_is_dir(vm, program, &info)?;
    match invoke_filepath_walk_callback(
        vm,
        program,
        callback,
        path,
        info.clone(),
        Value::nil(),
        absolute_output,
    )? {
        FilepathWalkResult::Continue => {}
        FilepathWalkResult::SkipDir if is_dir => return Ok(FilepathWalkResult::Continue),
        FilepathWalkResult::SkipDir => return Ok(FilepathWalkResult::SkipDir),
        FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
        FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
    }

    if !is_dir {
        return Ok(FilepathWalkResult::Continue);
    }

    let read_dir_results = super::super::io_fs_impl::io_fs_read_dir(
        vm,
        program,
        &[filesystem.clone(), Value::string(path)],
    )?;
    let (entries, read_dir_err) = normalize_filepath_walk_entries(vm, program, &read_dir_results)?;
    if let Some(read_dir_err) = read_dir_err {
        match invoke_filepath_walk_callback(
            vm,
            program,
            callback,
            path,
            info,
            read_dir_err,
            absolute_output,
        )? {
            FilepathWalkResult::Continue => {}
            FilepathWalkResult::SkipDir => return Ok(FilepathWalkResult::Continue),
            FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
            FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
        }
    }

    for entry in entries {
        let name = filepath_walk_dir_entry_name(vm, program, &entry)?;
        let child_path = filepath_walk_join_path(path, &name);
        let info_results = vm.invoke_method_results(program, entry, "Info", Vec::new())?;
        let (child_info, child_err) = normalize_filepath_walk_info(vm, program, &info_results)?;
        if let Some(child_err) = child_err {
            match invoke_filepath_walk_callback(
                vm,
                program,
                callback,
                &child_path,
                Value::nil(),
                child_err,
                absolute_output,
            )? {
                FilepathWalkResult::Continue => continue,
                FilepathWalkResult::SkipDir => break,
                FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
                FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
            }
        }
        match filepath_walk_path(
            vm,
            program,
            filesystem,
            &child_path,
            child_info,
            callback,
            absolute_output,
        )? {
            FilepathWalkResult::Continue => {}
            FilepathWalkResult::SkipDir => break,
            FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
            FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
        }
    }

    Ok(FilepathWalkResult::Continue)
}

fn invoke_filepath_walk_callback(
    vm: &mut Vm,
    program: &Program,
    callback: &FunctionValue,
    path: &str,
    info: Value,
    err: Value,
    absolute_output: bool,
) -> Result<FilepathWalkResult, VmError> {
    let mut callback_args = callback.captures.clone();
    callback_args.push(Value::string(display_filepath_walk_path(
        path,
        absolute_output,
    )));
    callback_args.push(info);
    callback_args.push(err);
    let result = vm.invoke_callback(program, callback.function, callback_args)?;
    match &result.data {
        ValueData::Nil => Ok(FilepathWalkResult::Continue),
        ValueData::Error(error) => {
            if error.message == super::super::io_fs_registry_impl::SKIP_DIR {
                Ok(FilepathWalkResult::SkipDir)
            } else if error.message == super::super::io_fs_registry_impl::SKIP_ALL {
                Ok(FilepathWalkResult::SkipAll)
            } else {
                Ok(FilepathWalkResult::Error(result))
            }
        }
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk callback".into(),
            expected: "error return value".into(),
        }),
    }
}

fn normalize_filepath_walk_entries(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<(Vec<Value>, Option<Value>), VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "a ([]fs.DirEntry, error) result".into(),
        });
    }
    let entries = match &results[0].data {
        ValueData::Slice(slice) => slice.values_snapshot(),
        ValueData::Nil => Vec::new(),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "filepath.Walk".into(),
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
                builtin: "filepath.Walk".into(),
                expected: "a ([]fs.DirEntry, error) result".into(),
            });
        }
    };
    Ok((entries, err))
}

fn normalize_filepath_walk_info(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<(Value, Option<Value>), VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "a (fs.FileInfo, error) result".into(),
        });
    }
    match &results[1].data {
        ValueData::Nil => Ok((results[0].clone(), None)),
        ValueData::Error(_) => Ok((Value::nil(), Some(results[1].clone()))),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "a (fs.FileInfo, error) result".into(),
        }),
    }
}

fn filepath_walk_file_info_is_dir(
    vm: &mut Vm,
    program: &Program,
    info: &Value,
) -> Result<bool, VmError> {
    match vm
        .invoke_method(program, info.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => Ok(is_dir),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "fs.FileInfo.IsDir to return bool".into(),
        }),
    }
}

fn filepath_walk_dir_entry_is_dir(
    vm: &mut Vm,
    program: &Program,
    entry: &Value,
) -> Result<bool, VmError> {
    match vm
        .invoke_method(program, entry.clone(), "IsDir", Vec::new())?
        .data
    {
        ValueData::Bool(is_dir) => Ok(is_dir),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.WalkDir".into(),
            expected: "fs.DirEntry.IsDir to return bool".into(),
        }),
    }
}

fn filepath_walk_dir_entry_name(
    vm: &mut Vm,
    program: &Program,
    entry: &Value,
) -> Result<String, VmError> {
    match vm
        .invoke_method(program, entry.clone(), "Name", Vec::new())?
        .data
    {
        ValueData::String(name) => Ok(name),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.Walk".into(),
            expected: "fs.DirEntry.Name to return string".into(),
        }),
    }
}

fn filepath_walk_join_path(parent: &str, name: &str) -> String {
    if parent == "." {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

fn filepath_walk_dir_path(
    vm: &mut Vm,
    program: &Program,
    filesystem: &Value,
    path: &str,
    entry: Value,
    callback: &FunctionValue,
    absolute_output: bool,
) -> Result<FilepathWalkResult, VmError> {
    let is_dir = filepath_walk_dir_entry_is_dir(vm, program, &entry)?;
    match invoke_filepath_walk_dir_callback(
        vm,
        program,
        callback,
        path,
        entry.clone(),
        Value::nil(),
        absolute_output,
    )? {
        FilepathWalkResult::Continue => {}
        FilepathWalkResult::SkipDir if is_dir => return Ok(FilepathWalkResult::Continue),
        FilepathWalkResult::SkipDir => return Ok(FilepathWalkResult::SkipDir),
        FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
        FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
    }

    if !is_dir {
        return Ok(FilepathWalkResult::Continue);
    }

    let read_dir_results = super::super::io_fs_impl::io_fs_read_dir(
        vm,
        program,
        &[filesystem.clone(), Value::string(path)],
    )?;
    let (entries, read_dir_err) = normalize_filepath_walk_entries(vm, program, &read_dir_results)?;
    if let Some(read_dir_err) = read_dir_err {
        match invoke_filepath_walk_dir_callback(
            vm,
            program,
            callback,
            path,
            entry,
            read_dir_err,
            absolute_output,
        )? {
            FilepathWalkResult::Continue => {}
            FilepathWalkResult::SkipDir => return Ok(FilepathWalkResult::Continue),
            FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
            FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
        }
    }

    for child in entries {
        let child_name = filepath_walk_dir_entry_name(vm, program, &child)?;
        let child_path = filepath_walk_join_path(path, &child_name);
        match filepath_walk_dir_path(
            vm,
            program,
            filesystem,
            &child_path,
            child,
            callback,
            absolute_output,
        )? {
            FilepathWalkResult::Continue => {}
            FilepathWalkResult::SkipDir => break,
            FilepathWalkResult::SkipAll => return Ok(FilepathWalkResult::SkipAll),
            FilepathWalkResult::Error(err) => return Ok(FilepathWalkResult::Error(err)),
        }
    }

    Ok(FilepathWalkResult::Continue)
}

fn invoke_filepath_walk_dir_callback(
    vm: &mut Vm,
    program: &Program,
    callback: &FunctionValue,
    path: &str,
    entry: Value,
    err: Value,
    absolute_output: bool,
) -> Result<FilepathWalkResult, VmError> {
    let mut callback_args = callback.captures.clone();
    callback_args.push(Value::string(display_filepath_walk_path(
        path,
        absolute_output,
    )));
    callback_args.push(entry);
    callback_args.push(err);
    let result = vm.invoke_callback(program, callback.function, callback_args)?;
    match &result.data {
        ValueData::Nil => Ok(FilepathWalkResult::Continue),
        ValueData::Error(error) => {
            if error.message == super::super::io_fs_registry_impl::SKIP_DIR {
                Ok(FilepathWalkResult::SkipDir)
            } else if error.message == super::super::io_fs_registry_impl::SKIP_ALL {
                Ok(FilepathWalkResult::SkipAll)
            } else {
                Ok(FilepathWalkResult::Error(result))
            }
        }
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "filepath.WalkDir callback".into(),
            expected: "error return value".into(),
        }),
    }
}

fn normalize_filepath_walk_root(root: &str) -> (String, bool) {
    let absolute = root.starts_with('/');
    let normalized = super::super::workspace_fs_impl::normalize_os_path(root)
        .unwrap_or_else(|| root.to_string());
    (normalized, absolute)
}

fn display_filepath_walk_path(path: &str, absolute: bool) -> String {
    if absolute {
        super::super::workspace_fs_impl::absolute_workspace_path(path)
    } else {
        path.to_string()
    }
}
