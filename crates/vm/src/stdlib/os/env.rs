use crate::{Program, Value, ValueData, Vm, VmError};

const WASM_PAGE_SIZE: i64 = 64 * 1024;

pub(super) fn os_getenv(vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let key = match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s.as_str(),
        _ => "",
    };
    let value = vm.env.get(key).cloned().unwrap_or_default();
    Ok(Value::string(value))
}

pub(super) fn os_setenv(vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let key = match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s.clone(),
        _ => String::new(),
    };
    let value = match args.get(1) {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s.clone(),
        _ => String::new(),
    };
    vm.env.insert(key, value);
    Ok(Value::nil())
}

pub(super) fn os_unsetenv(
    vm: &mut Vm,
    _program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let key = match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s.as_str(),
        _ => "",
    };
    vm.env.remove(key);
    Ok(Value::nil())
}

pub(super) fn os_clearenv(
    vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    vm.env.clear();
    Ok(Value::nil())
}

pub(super) fn os_environ(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }

    let mut values: Vec<String> = vm
        .env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    values.sort();
    Ok(Value::slice(
        values.into_iter().map(Value::string).collect(),
    ))
}

pub(super) fn os_lookup_env(
    vm: &mut Vm,
    _program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let key = match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s.as_str(),
        _ => "",
    };
    match vm.env.get(key) {
        Some(value) => Ok(vec![Value::string(value.clone()), Value::bool(true)]),
        None => Ok(vec![Value::string(String::new()), Value::bool(false)]),
    }
}

pub(super) fn os_expand_env(
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
    let ValueData::String(input) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.ExpandEnv".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(Value::string(expand_with_mapping(input, |name| {
        vm.env.get(name).cloned().unwrap_or_default()
    })))
}

pub(super) fn os_expand(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::String(input) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.Expand".into(),
            expected: "a string argument".into(),
        });
    };
    let ValueData::Function(mapping) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[1]),
        });
    };
    let mapping = mapping.clone();
    Ok(Value::string(try_expand_with_mapping(input, |name| {
        let mut callback_args = mapping.captures.clone();
        callback_args.push(Value::string(name.to_string()));
        let result = vm.invoke_callback(program, mapping.function, callback_args)?;
        match result.data {
            ValueData::String(value) => Ok(value),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "os.Expand callback".into(),
                expected: "string return value".into(),
            }),
        }
    })?))
}

pub(super) fn os_temp_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(Value::string(
        vm.env
            .get("TMPDIR")
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "/tmp".to_string()),
    ))
}

pub(super) fn os_user_home_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    ok_or_error(
        vm.env
            .get("HOME")
            .filter(|value| !value.is_empty())
            .cloned(),
        "$HOME is not defined",
    )
}

pub(super) fn os_user_cache_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    if let Some(dir) = vm
        .env
        .get("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .cloned()
    {
        return Ok(vec![Value::string(dir), Value::nil()]);
    }

    ok_or_error(
        vm.env
            .get("HOME")
            .filter(|value| !value.is_empty())
            .map(|home| format!("{home}/.cache")),
        "neither $XDG_CACHE_HOME nor $HOME are defined",
    )
}

pub(super) fn os_user_config_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    if let Some(dir) = vm
        .env
        .get("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .cloned()
    {
        return Ok(vec![Value::string(dir), Value::nil()]);
    }

    ok_or_error(
        vm.env
            .get("HOME")
            .filter(|value| !value.is_empty())
            .map(|home| format!("{home}/.config")),
        "neither $XDG_CONFIG_HOME nor $HOME are defined",
    )
}

pub(super) fn os_hostname(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(vec![Value::string("js"), Value::nil()])
}

pub(super) fn os_executable(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(vec![
        Value::string(String::new()),
        Value::error("Executable not implemented for js"),
    ])
}

pub(super) fn os_getuid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_geteuid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_getgid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_getegid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_getpid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_getppid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    browser_identity_sentinel(vm, program, args)
}

pub(super) fn os_getpagesize(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(Value::int(WASM_PAGE_SIZE))
}

pub(super) fn os_getgroups(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(vec![
        Value::slice(Vec::new()),
        Value::error("Getgroups not implemented for js"),
    ])
}

fn expand_with_mapping<F>(input: &str, mut mapping: F) -> String
where
    F: FnMut(&str) -> String,
{
    try_expand_with_mapping(input, |name| Ok(mapping(name)))
        .expect("env expansion should not fail with infallible mapping")
}

fn expect_no_args(vm: &Vm, program: &Program, args: &[Value]) -> Result<(), VmError> {
    if args.is_empty() {
        return Ok(());
    }
    Err(VmError::WrongArgumentCount {
        function: vm.current_function_name(program)?,
        expected: 0,
        actual: args.len(),
    })
}

fn ok_or_error(value: Option<String>, message: &str) -> Result<Vec<Value>, VmError> {
    Ok(match value {
        Some(value) => vec![Value::string(value), Value::nil()],
        None => vec![Value::string(String::new()), Value::error(message)],
    })
}

fn browser_identity_sentinel(vm: &Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    expect_no_args(vm, program, args)?;
    Ok(Value::int(-1))
}

fn try_expand_with_mapping<F>(input: &str, mut mapping: F) -> Result<String, VmError>
where
    F: FnMut(&str) -> Result<String, VmError>,
{
    let bytes = input.as_bytes();
    let mut buffer: Option<Vec<u8>> = None;
    let mut i = 0usize;
    let mut j = 0usize;

    while j < bytes.len() {
        if bytes[j] == b'$' && j + 1 < bytes.len() {
            let output = buffer.get_or_insert_with(|| Vec::with_capacity(2 * bytes.len()));
            output.extend_from_slice(&bytes[i..j]);
            let (name, width) = get_shell_name(&input[j + 1..]);
            if name.is_empty() && width == 0 {
                output.push(bytes[j]);
            } else if !name.is_empty() {
                output.extend_from_slice(mapping(name)?.as_bytes());
            }
            j += width;
            i = j + 1;
        }
        j += 1;
    }

    match buffer {
        None => Ok(input.to_string()),
        Some(mut output) => {
            output.extend_from_slice(&bytes[i..]);
            Ok(String::from_utf8(output).expect("expanded env strings should stay utf-8"))
        }
    }
}

fn is_shell_special_var(byte: u8) -> bool {
    matches!(
        byte,
        b'*' | b'#'
            | b'$'
            | b'@'
            | b'!'
            | b'?'
            | b'-'
            | b'0'
            | b'1'
            | b'2'
            | b'3'
            | b'4'
            | b'5'
            | b'6'
            | b'7'
            | b'8'
            | b'9'
    )
}

fn is_alpha_num(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_digit() || byte.is_ascii_lowercase() || byte.is_ascii_uppercase()
}

fn get_shell_name(input: &str) -> (&str, usize) {
    let bytes = input.as_bytes();
    match () {
        _ if bytes[0] == b'{' => {
            if bytes.len() > 2 && is_shell_special_var(bytes[1]) && bytes[2] == b'}' {
                return (&input[1..2], 3);
            }
            for index in 1..bytes.len() {
                if bytes[index] == b'}' {
                    if index == 1 {
                        return ("", 2);
                    }
                    return (&input[1..index], index + 1);
                }
            }
            ("", 1)
        }
        _ if is_shell_special_var(bytes[0]) => (&input[..1], 1),
        _ => {
            let mut index = 0usize;
            while index < bytes.len() && is_alpha_num(bytes[index]) {
                index += 1;
            }
            (&input[..index], index)
        }
    }
}
