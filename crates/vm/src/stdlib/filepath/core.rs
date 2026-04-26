use crate::{Program, Value, ValueData, Vm, VmError};

pub(crate) fn filepath_abs(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let path = filepath_arg(vm, program, "filepath.Abs", args)?;
    let rooted = if super::super::path_impl::is_abs(path) {
        path.to_string()
    } else {
        format!("/{path}")
    };
    Ok(vec![
        Value::string(super::super::path_impl::clean(&rooted)),
        Value::nil(),
    ])
}

pub(crate) fn filepath_base(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.Base", args)?;
    Ok(Value::string(super::super::path_impl::base(path)))
}

pub(crate) fn filepath_clean(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.Clean", args)?;
    Ok(Value::string(super::super::path_impl::clean(path)))
}

pub(crate) fn filepath_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.Dir", args)?;
    Ok(Value::string(super::super::path_impl::dir(path)))
}

pub(crate) fn filepath_ext(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.Ext", args)?;
    Ok(Value::string(super::super::path_impl::ext(path)))
}

pub(crate) fn filepath_is_abs(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.IsAbs", args)?;
    Ok(Value::bool(super::super::path_impl::is_abs(path)))
}

pub(crate) fn filepath_split(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let path = filepath_arg(vm, program, "filepath.Split", args)?;
    let (dir, file) = super::super::path_impl::split(path);
    Ok(vec![Value::string(dir), Value::string(file)])
}

pub(crate) fn filepath_join(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let paths = filepath_string_args(vm, program, "filepath.Join", args)?;
    Ok(Value::string(super::super::path_impl::join(&paths)))
}

pub(crate) fn filepath_match(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (pattern, name) = filepath_pair_args(vm, program, "filepath.Match", args)?;
    match super::super::path_impl::match_pattern(pattern, name) {
        Ok(matched) => Ok(vec![Value::bool(matched), Value::nil()]),
        Err(detail) => Ok(vec![Value::bool(false), Value::error(detail)]),
    }
}

pub(crate) fn filepath_to_slash(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.ToSlash", args)?;
    Ok(Value::string(path))
}

pub(crate) fn filepath_from_slash(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.FromSlash", args)?;
    Ok(Value::string(path))
}

pub(crate) fn filepath_split_list(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.SplitList", args)?;
    if path.is_empty() {
        return Ok(Value::slice(Vec::new()));
    }
    Ok(Value::slice(
        path.split(':').map(Value::string).collect::<Vec<_>>(),
    ))
}

pub(crate) fn filepath_volume_name(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let _path = filepath_arg(vm, program, "filepath.VolumeName", args)?;
    Ok(Value::string(String::new()))
}

pub(crate) fn filepath_rel(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (basepath, targpath) = filepath_pair_args(vm, program, "filepath.Rel", args)?;
    let base = super::super::path_impl::clean(basepath);
    let targ = super::super::path_impl::clean(targpath);
    if targ == base {
        return Ok(vec![Value::string("."), Value::nil()]);
    }

    let base = if base == "." { "" } else { base.as_str() };
    let targ = if targ == "." { "" } else { targ.as_str() };

    if base.starts_with('/') != targ.starts_with('/') {
        return Ok(vec![
            Value::string(String::new()),
            Value::error(format!("Rel: can't make {targpath} relative to {basepath}")),
        ]);
    }

    let (mut b0, mut bi, mut t0, mut ti) = (0usize, 0usize, 0usize, 0usize);
    while bi <= base.len() && ti <= targ.len() {
        while bi < base.len() && base.as_bytes()[bi] != b'/' {
            bi += 1;
        }
        while ti < targ.len() && targ.as_bytes()[ti] != b'/' {
            ti += 1;
        }
        if targ[t0..ti] != base[b0..bi] {
            break;
        }
        if bi < base.len() {
            bi += 1;
        }
        if ti < targ.len() {
            ti += 1;
        }
        b0 = bi;
        t0 = ti;
    }

    if base[b0..bi] == *".." {
        return Ok(vec![
            Value::string(String::new()),
            Value::error(format!("Rel: can't make {targpath} relative to {basepath}")),
        ]);
    }

    if b0 != base.len() {
        let seps = base[b0..].bytes().filter(|byte| *byte == b'/').count();
        let mut rel = String::from("..");
        for _ in 0..seps {
            rel.push_str("/..");
        }
        if t0 != targ.len() {
            rel.push('/');
            rel.push_str(&targ[t0..]);
        }
        return Ok(vec![Value::string(rel), Value::nil()]);
    }

    Ok(vec![Value::string(&targ[t0..]), Value::nil()])
}

pub(crate) fn filepath_is_local(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = filepath_arg(vm, program, "filepath.IsLocal", args)?;
    Ok(Value::bool(is_local_path(path)))
}

pub(crate) fn filepath_localize(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let path = filepath_arg(vm, program, "filepath.Localize", args)?;
    if !super::super::workspace_fs_impl::valid_path(path) || path.bytes().any(|byte| byte == 0) {
        return Ok(vec![
            Value::string(String::new()),
            Value::error("invalid path"),
        ]);
    }
    Ok(vec![Value::string(path), Value::nil()])
}

pub(crate) fn filepath_glob(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let pattern = filepath_arg(vm, program, "filepath.Glob", args)?;
    let (normalized_pattern, absolute_output) = normalize_filepath_glob_pattern(pattern);
    Ok(
        match super::super::workspace_fs_impl::glob_workspace_files(vm, None, &normalized_pattern) {
            Ok(matches) => vec![
                string_slice_value(&format_glob_matches(matches, absolute_output)),
                Value::nil(),
            ],
            Err(error) => vec![
                Value::nil_slice(),
                super::super::workspace_fs_impl::error_value(error),
            ],
        },
    )
}

fn filepath_arg<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<&'a str, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(path) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };
    Ok(path)
}

fn filepath_string_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<Vec<&'a str>, VmError> {
    let mut paths = Vec::with_capacity(args.len());
    for arg in args {
        let ValueData::String(path) = &arg.data else {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "string arguments".into(),
            });
        };
        paths.push(path.as_str());
    }
    Ok(paths)
}

fn filepath_pair_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::String(left) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(right) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok((left, right))
}

fn is_local_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') {
        return false;
    }

    let has_dot_elements = path
        .split('/')
        .any(|element| element == "." || element == "..");
    let cleaned = if has_dot_elements {
        super::super::path_impl::clean(path)
    } else {
        path.to_string()
    };

    cleaned != ".." && !cleaned.starts_with("../")
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

fn normalize_filepath_glob_pattern(pattern: &str) -> (String, bool) {
    let absolute = pattern.starts_with('/');
    let mut segments = Vec::new();
    for (index, segment) in pattern.split('/').enumerate() {
        if absolute && index == 0 && segment.is_empty() {
            continue;
        }
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            let _ = segments.pop();
            continue;
        }
        segments.push(segment);
    }

    let normalized = if segments.is_empty() {
        ".".to_string()
    } else {
        segments.join("/")
    };
    (normalized, absolute)
}

fn format_glob_matches(matches: Vec<String>, absolute: bool) -> Vec<String> {
    if !absolute {
        return matches;
    }
    matches
        .into_iter()
        .map(|path| super::super::workspace_fs_impl::absolute_workspace_path(&path))
        .collect()
}
