use crate::{Program, Value, Vm, VmError};

pub(super) fn io_fs_file_info_name(
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
    Ok(Value::string(super::workspace_fs_impl::file_info_name(
        vm,
        program,
        "fs.FileInfo.Name",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_info_is_dir(
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
    Ok(Value::bool(super::workspace_fs_impl::file_info_is_dir(
        vm,
        program,
        "fs.FileInfo.IsDir",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_info_size(
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
    Ok(Value::int(super::workspace_fs_impl::file_info_size(
        vm,
        program,
        "fs.FileInfo.Size",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_info_mode(
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
    super::workspace_fs_impl::file_info_mode(vm, program, "fs.FileInfo.Mode", &args[0])
}

pub(super) fn io_fs_file_info_mod_time(
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
    super::workspace_fs_impl::file_info_mod_time(vm, program, "fs.FileInfo.ModTime", &args[0])
}

pub(super) fn io_fs_file_info_sys(
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
    super::workspace_fs_metadata_impl::file_info_sys(vm, program, "fs.FileInfo.Sys", &args[0])
}

pub(super) fn io_fs_file_mode_is_dir(
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
    Ok(Value::bool(super::workspace_fs_impl::file_mode_is_dir(
        vm,
        program,
        "fs.FileMode.IsDir",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_mode_is_regular(
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
    Ok(Value::bool(super::workspace_fs_impl::file_mode_is_regular(
        vm,
        program,
        "fs.FileMode.IsRegular",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_mode_type(
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
    super::workspace_fs_impl::file_mode_type(vm, program, "fs.FileMode.Type", &args[0])
}

pub(super) fn io_fs_file_mode_string(
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
    Ok(Value::string(super::workspace_fs_impl::file_mode_string(
        vm,
        program,
        "fs.FileMode.String",
        &args[0],
    )?))
}

pub(super) fn io_fs_file_mode_perm(
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
    super::workspace_fs_impl::file_mode_perm(vm, program, "fs.FileMode.Perm", &args[0])
}

pub(super) fn io_fs_dir_entry_name(
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
    Ok(Value::string(super::workspace_fs_impl::dir_entry_name(
        vm,
        program,
        "fs.DirEntry.Name",
        &args[0],
    )?))
}

pub(super) fn io_fs_dir_entry_is_dir(
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
    Ok(Value::bool(super::workspace_fs_impl::dir_entry_is_dir(
        vm,
        program,
        "fs.DirEntry.IsDir",
        &args[0],
    )?))
}

pub(super) fn io_fs_dir_entry_type(
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
    super::workspace_fs_impl::dir_entry_type(vm, program, "fs.DirEntry.Type", &args[0])
}
