use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) fn io_fs_valid_path(
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
            builtin: "fs.ValidPath".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(Value::bool(super::workspace_fs_impl::valid_path(name)))
}

pub(super) fn io_fs_read_file(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadFile".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };

    match super::io_fs_fallback_impl::subfs_path_call(&args[0], name) {
        Ok(Some((filesystem, path))) => {
            return io_fs_read_file(vm, program, &[filesystem, Value::string(path)]);
        }
        Ok(None) => {}
        Err(error) => {
            return Ok(vec![
                Value::nil_slice(),
                super::workspace_fs_impl::error_value(error),
            ]);
        }
    }

    if let Ok(root) =
        super::workspace_fs_impl::read_dirfs_root(vm, program, "fs.ReadFile", &args[0])
    {
        return Ok(
            match super::workspace_fs_impl::read_workspace_file(vm, Some(&root), name) {
                Ok(bytes) => vec![
                    super::workspace_fs_impl::bytes_to_value(&bytes),
                    Value::nil(),
                ],
                Err(error) => {
                    vec![
                        Value::nil_slice(),
                        super::workspace_fs_impl::error_value(error),
                    ]
                }
            },
        );
    }

    match vm.invoke_method_results(
        program,
        args[0].clone(),
        "ReadFile",
        vec![Value::string(name.clone())],
    ) {
        Ok(results) => normalize_read_file_results(vm, program, &results),
        Err(VmError::UnknownMethod { .. }) => {
            super::io_fs_fallback_impl::read_file_via_open(vm, program, &args[0], name)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn io_fs_stat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Stat".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };

    match super::io_fs_fallback_impl::subfs_path_call(&args[0], name) {
        Ok(Some((filesystem, path))) => {
            return io_fs_stat(vm, program, &[filesystem, Value::string(path)]);
        }
        Ok(None) => {}
        Err(error) => {
            return Ok(vec![
                Value::nil(),
                super::workspace_fs_impl::error_value(error),
            ]);
        }
    }

    match vm.invoke_method_results(
        program,
        args[0].clone(),
        "Stat",
        vec![Value::string(name.clone())],
    ) {
        Ok(results) => normalize_stat_results(vm, program, "fs.Stat", &results),
        Err(VmError::UnknownMethod { .. }) => {
            let open_results = match vm.invoke_method_results(
                program,
                args[0].clone(),
                "Open",
                vec![Value::string(name.clone())],
            ) {
                Ok(results) => results,
                Err(VmError::UnknownMethod { .. }) => {
                    return Ok(vec![
                        Value::nil(),
                        Value::error("fs.Stat: unsupported fs.FS implementation"),
                    ]);
                }
                Err(error) => return Err(error),
            };
            let Some(file) = normalize_open_results(vm, program, "fs.Stat", &open_results)? else {
                return Ok(vec![Value::nil(), open_results[1].clone()]);
            };
            let stat_results =
                vm.invoke_method_results(program, file.clone(), "Stat", Vec::new())?;
            let normalized = normalize_stat_results(vm, program, "fs.Stat", &stat_results)?;
            let _ = vm.invoke_method(program, file, "Close", Vec::new())?;
            Ok(normalized)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn io_fs_read_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadDir".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };

    match super::io_fs_fallback_impl::subfs_path_call(&args[0], name) {
        Ok(Some((filesystem, path))) => {
            return io_fs_read_dir(vm, program, &[filesystem, Value::string(path)]);
        }
        Ok(None) => {}
        Err(error) => {
            return Ok(vec![
                Value::nil_slice(),
                super::workspace_fs_impl::error_value(error),
            ]);
        }
    }

    match vm.invoke_method_results(
        program,
        args[0].clone(),
        "ReadDir",
        vec![Value::string(name.clone())],
    ) {
        Ok(results) => normalize_read_dir_results(vm, program, &results),
        Err(VmError::UnknownMethod { .. }) => {
            let open_results = match vm.invoke_method_results(
                program,
                args[0].clone(),
                "Open",
                vec![Value::string(name.clone())],
            ) {
                Ok(results) => results,
                Err(VmError::UnknownMethod { .. }) => {
                    return Ok(vec![
                        Value::nil_slice(),
                        Value::error("fs.ReadDir: unsupported fs.FS implementation"),
                    ]);
                }
                Err(error) => return Err(error),
            };
            let Some(file) = normalize_open_results(vm, program, "fs.ReadDir", &open_results)?
            else {
                return Ok(vec![Value::nil_slice(), open_results[1].clone()]);
            };
            let read_dir_results =
                vm.invoke_method_results(program, file.clone(), "ReadDir", vec![Value::int(-1)])?;
            let normalized = normalize_read_dir_results(vm, program, &read_dir_results)?;
            let _ = vm.invoke_method(program, file, "Close", Vec::new())?;
            Ok(normalized)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn io_fs_sub(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(dir) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Sub".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };

    if let Ok(root) = super::workspace_fs_impl::read_dirfs_root(vm, program, "fs.Sub", &args[0]) {
        return Ok(match super::workspace_fs_impl::subdir_root(&root, dir) {
            Some(subdir) => vec![super::workspace_fs_impl::dirfs_value(&subdir), Value::nil()],
            None => vec![
                Value::nil(),
                Value::error(format!("sub {dir}: invalid path")),
            ],
        });
    }

    match vm.invoke_method_results(
        program,
        args[0].clone(),
        "Sub",
        vec![Value::string(dir.clone())],
    ) {
        Ok(results) => normalize_sub_results(vm, program, &results),
        Err(VmError::UnknownMethod { .. }) => {
            Ok(super::io_fs_fallback_impl::subfs_value(&args[0], dir)
                .map(|filesystem| vec![filesystem, Value::nil()])
                .unwrap_or_else(|| {
                    vec![
                        Value::nil(),
                        Value::error(format!("sub {dir}: invalid path")),
                    ]
                }))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn io_fs_glob(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(pattern) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Glob".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };

    if let Some((filesystem, rebased_pattern)) =
        super::io_fs_fallback_impl::subfs_glob_call(&args[0], pattern)
    {
        return io_fs_glob(vm, program, &[filesystem, Value::string(rebased_pattern)]);
    }

    if let Ok(root) = super::workspace_fs_impl::read_dirfs_root(vm, program, "fs.Glob", &args[0]) {
        return Ok(
            match super::workspace_fs_impl::glob_workspace_files(vm, Some(&root), pattern) {
                Ok(matches) => vec![string_slice_value(&matches), Value::nil()],
                Err(error) => {
                    vec![
                        Value::nil_slice(),
                        super::workspace_fs_impl::error_value(error),
                    ]
                }
            },
        );
    }

    match vm.invoke_method_results(
        program,
        args[0].clone(),
        "Glob",
        vec![Value::string(pattern.clone())],
    ) {
        Ok(results) => normalize_glob_results(vm, program, &results),
        Err(VmError::UnknownMethod { .. }) => {
            super::io_fs_fallback_impl::glob_via_read_dir(vm, program, &args[0], pattern)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn io_fs_walk_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    super::io_fs_walk_impl::io_fs_walk_dir(vm, program, args)
}

pub(super) fn io_fs_fs_open(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(name) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.FS.Open".into(),
            expected: "fs.FS and string arguments".into(),
        });
    };
    match super::io_fs_fallback_impl::subfs_path_call(&args[0], name) {
        Ok(Some((filesystem, path))) => {
            return vm.invoke_method_results(
                program,
                filesystem,
                "Open",
                vec![Value::string(path)],
            );
        }
        Ok(None) => {}
        Err(error) => {
            return Ok(vec![
                Value::nil(),
                super::workspace_fs_impl::error_value(error),
            ]);
        }
    }
    let root = super::workspace_fs_impl::read_dirfs_root(vm, program, "fs.FS.Open", &args[0])?;
    Ok(
        match super::workspace_fs_impl::open_workspace_file(vm, Some(&root), name, true) {
            Ok(file) => vec![file, Value::nil()],
            Err(error) => vec![Value::nil(), super::workspace_fs_impl::error_value(error)],
        },
    )
}

pub(super) fn io_fs_file_close(
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
    super::workspace_fs_impl::close_workspace_file(vm, program, "fs.File.Close", &args[0])
}

pub(super) fn io_fs_file_stat(
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
    super::workspace_fs_impl::stat_workspace_file(vm, program, "fs.File.Stat", &args[0])
}

pub(super) fn io_fs_file_read(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    vm.workspace_fs_read(program, &args[0], &args[1])
}

pub(super) fn io_fs_read_dir_file_read_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Int(n) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadDirFile.ReadDir".into(),
            expected: "fs.ReadDirFile and int arguments".into(),
        });
    };
    super::workspace_fs_impl::read_workspace_file_dir_entries(
        vm,
        program,
        "fs.ReadDirFile.ReadDir",
        &args[0],
        *n,
    )
}

pub(super) fn io_fs_dir_entry_info(
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
    super::workspace_fs_impl::dir_entry_info(vm, program, "fs.DirEntry.Info", &args[0])
}

fn normalize_read_file_results(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<Vec<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadFile".into(),
            expected: "a ([]byte, error) result".into(),
        });
    }
    let bytes =
        super::workspace_fs_impl::extract_byte_slice(vm, program, "fs.ReadFile", &results[0])?;
    let err = match &results[1].data {
        ValueData::Nil | ValueData::Error(_) => results[1].clone(),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "fs.ReadFile".into(),
                expected: "a ([]byte, error) result".into(),
            });
        }
    };
    Ok(vec![super::workspace_fs_impl::bytes_to_value(&bytes), err])
}

fn normalize_read_dir_results(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<Vec<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadDir".into(),
            expected: "a ([]fs.DirEntry, error) result".into(),
        });
    }
    match &results[1].data {
        ValueData::Nil => {
            validate_dir_entry_slice(vm, program, "fs.ReadDir", &results[0], false)?;
            Ok(results.to_vec())
        }
        ValueData::Error(_) => {
            validate_dir_entry_slice(vm, program, "fs.ReadDir", &results[0], true)?;
            Ok(results.to_vec())
        }
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.ReadDir".into(),
            expected: "a ([]fs.DirEntry, error) result".into(),
        }),
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

fn normalize_stat_results(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    results: &[Value],
) -> Result<Vec<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a (fs.FileInfo, error) result".into(),
        });
    }
    match &results[1].data {
        ValueData::Nil if matches!(results[0].data, ValueData::Nil) => {
            Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a (fs.FileInfo, error) result".into(),
            })
        }
        ValueData::Nil | ValueData::Error(_) => Ok(results.to_vec()),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a (fs.FileInfo, error) result".into(),
        }),
    }
}

fn normalize_sub_results(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<Vec<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Sub".into(),
            expected: "a (fs.FS, error) result".into(),
        });
    }
    match &results[1].data {
        ValueData::Nil | ValueData::Error(_) => Ok(results.to_vec()),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Sub".into(),
            expected: "a (fs.FS, error) result".into(),
        }),
    }
}

fn normalize_glob_results(
    vm: &Vm,
    program: &Program,
    results: &[Value],
) -> Result<Vec<Value>, VmError> {
    if results.len() != 2 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Glob".into(),
            expected: "a ([]string, error) result".into(),
        });
    }
    validate_string_slice(vm, program, "fs.Glob", &results[0])?;
    match &results[1].data {
        ValueData::Nil | ValueData::Error(_) => Ok(results.to_vec()),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "fs.Glob".into(),
            expected: "a ([]string, error) result".into(),
        }),
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

fn validate_string_slice(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<(), VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a []string result".into(),
        });
    };
    let values = slice.values_snapshot();
    for entry in &values {
        if !matches!(entry.data, ValueData::String(_)) {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a []string result".into(),
            });
        }
    }
    Ok(())
}

fn validate_dir_entry_slice(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
    allow_nil: bool,
) -> Result<(), VmError> {
    match &value.data {
        ValueData::Slice(_) => Ok(()),
        ValueData::Nil if allow_nil => Ok(()),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a []fs.DirEntry result".into(),
        }),
    }
}
