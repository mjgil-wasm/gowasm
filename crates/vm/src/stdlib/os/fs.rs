use crate::{Program, Value, ValueData, Vm, VmError, TYPE_FS_FILE_MODE, TYPE_INT};

pub(super) fn os_dir_fs(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(root) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.DirFS".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(super::super::workspace_fs_impl::os_dirfs_value(root)
        .unwrap_or_else(|| super::super::workspace_fs_impl::dirfs_value(root)))
}

pub(super) fn os_read_file(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.ReadFile".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(
        match super::super::workspace_fs_impl::read_os_workspace_file(vm, name) {
            Ok(bytes) => vec![
                super::super::workspace_fs_impl::bytes_to_value(&bytes),
                Value::nil(),
            ],
            Err(error) => vec![
                Value::nil_slice(),
                super::super::workspace_fs_impl::error_value(error),
            ],
        },
    )
}

pub(super) fn os_write_file(
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
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.WriteFile".into(),
            expected: "string, []byte, and fs.FileMode arguments".into(),
        });
    };
    let bytes =
        super::super::workspace_fs_impl::extract_byte_slice(vm, program, "os.WriteFile", &args[1])?;
    match (&args[2].typ, &args[2].data) {
        (typ, ValueData::Int(_)) if *typ == TYPE_FS_FILE_MODE || *typ == TYPE_INT => {}
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "os.WriteFile".into(),
                expected: "string, []byte, and fs.FileMode arguments".into(),
            })
        }
    }

    Ok(
        match super::super::workspace_fs_impl::write_os_workspace_file(vm, name, &bytes) {
            Ok(()) => Value::nil(),
            Err(error) => super::super::workspace_fs_impl::error_value(error),
        },
    )
}

pub(super) fn os_read_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.ReadDir".into(),
            expected: "a string argument".into(),
        });
    };
    let file = match super::super::workspace_fs_impl::open_os_workspace_file(vm, name, true) {
        Ok(file) => file,
        Err(error) => {
            return Ok(vec![
                Value::nil_slice(),
                super::super::workspace_fs_impl::error_value(error),
            ])
        }
    };
    let result = super::super::workspace_fs_impl::read_workspace_file_dir_entries(
        vm,
        program,
        "os.ReadDir",
        &file,
        -1,
    )?;
    let _ =
        super::super::workspace_fs_impl::close_workspace_file(vm, program, "os.ReadDir", &file)?;
    Ok(result)
}

pub(super) fn os_mkdir_all(
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
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.MkdirAll".into(),
            expected: "string and fs.FileMode arguments".into(),
        });
    };
    match (&args[1].typ, &args[1].data) {
        (typ, ValueData::Int(_)) if *typ == TYPE_FS_FILE_MODE || *typ == TYPE_INT => {}
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "os.MkdirAll".into(),
                expected: "string and fs.FileMode arguments".into(),
            })
        }
    }

    Ok(
        match super::super::workspace_fs_impl::mkdir_all_os_workspace_path(vm, name) {
            Ok(()) => Value::nil(),
            Err(error) => super::super::workspace_fs_impl::error_value(error),
        },
    )
}

pub(super) fn os_remove_all(
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
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.RemoveAll".into(),
            expected: "a string argument".into(),
        });
    };

    Ok(
        match super::super::workspace_fs_impl::remove_all_os_workspace_path(vm, name) {
            Ok(()) => Value::nil(),
            Err(error) => super::super::workspace_fs_impl::error_value(error),
        },
    )
}

pub(super) fn os_stat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    stat_workspace_path(vm, program, args, "os.Stat")
}

pub(super) fn os_lstat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    stat_workspace_path(vm, program, args, "os.Lstat")
}

pub(super) fn os_getwd(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(vec![Value::string("/"), Value::nil()])
}

pub(super) fn os_is_path_separator(
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
    let ValueData::Int(value) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.IsPathSeparator".into(),
            expected: "a byte argument".into(),
        });
    };
    Ok(Value::bool(value == i64::from(b'/')))
}

pub(super) fn os_same_file(
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

    let left = super::super::workspace_fs_impl::same_file_workspace_path(&args[0]);
    let right = super::super::workspace_fs_impl::same_file_workspace_path(&args[1]);
    Ok(Value::bool(matches!(
        (left, right),
        (Some(left), Some(right)) if left == right
    )))
}

fn stat_workspace_path(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    builtin: &str,
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };

    let file = match super::super::workspace_fs_impl::open_os_workspace_file(vm, name, true) {
        Ok(file) => file,
        Err(error) => {
            return Ok(vec![
                Value::nil(),
                super::super::workspace_fs_impl::error_value(error),
            ])
        }
    };
    let result = super::super::workspace_fs_impl::stat_workspace_file(vm, program, builtin, &file)?;
    let _ = super::super::workspace_fs_impl::close_workspace_file(vm, program, builtin, &file)?;
    Ok(result)
}
