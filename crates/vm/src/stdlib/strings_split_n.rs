use crate::{Program, Value, ValueData, Vm, VmError};

pub fn strings_split_n(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (text, sep, count) = string_pair_int_count_args(vm, program, "strings.SplitN", args)?;
    if count == 0 {
        return Ok(Value::nil_slice());
    }

    let parts = split_n(text, sep, count, false)
        .into_iter()
        .map(Value::string)
        .collect::<Vec<_>>();
    Ok(Value::slice(parts))
}

pub fn strings_split_after_n(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, sep, count) = string_pair_int_count_args(vm, program, "strings.SplitAfterN", args)?;
    if count == 0 {
        return Ok(Value::nil_slice());
    }

    let parts = split_n(text, sep, count, true)
        .into_iter()
        .map(Value::string)
        .collect::<Vec<_>>();
    Ok(Value::slice(parts))
}

pub fn strings_split_after(
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
    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strings.SplitAfter".into(),
            expected: "two string arguments".into(),
        });
    };
    let ValueData::String(sep) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strings.SplitAfter".into(),
            expected: "two string arguments".into(),
        });
    };

    let parts = split_n(text, sep, -1, true)
        .into_iter()
        .map(Value::string)
        .collect::<Vec<_>>();
    Ok(Value::slice(parts))
}

fn string_pair_int_count_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str, i64), VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    let ValueData::String(sep) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    let ValueData::Int(count) = args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    Ok((text, sep, count))
}

fn split_n(text: &str, sep: &str, count: i64, keep_sep: bool) -> Vec<String> {
    if count == 1 {
        return vec![text.to_string()];
    }
    if sep.is_empty() {
        return split_empty_sep_n(text, count);
    }
    if count < 0 {
        if keep_sep {
            return text.split_inclusive(sep).map(str::to_string).collect();
        }
        return text.split(sep).map(str::to_string).collect();
    }

    if !keep_sep {
        return text
            .splitn(count as usize, sep)
            .map(str::to_string)
            .collect();
    }

    let mut parts = Vec::new();
    let mut start = 0;
    let mut matches = text.match_indices(sep);
    for _ in 0..(count - 1) {
        let Some((index, _)) = matches.next() else {
            break;
        };
        let end = index + sep.len();
        parts.push(text[start..end].to_string());
        start = end;
    }
    parts.push(text[start..].to_string());
    parts
}

fn split_empty_sep_n(text: &str, count: i64) -> Vec<String> {
    let chars = text.char_indices().collect::<Vec<_>>();
    if count < 0 || count as usize >= chars.len() {
        return text.chars().map(|ch| ch.to_string()).collect();
    }
    if count == 1 {
        return vec![text.to_string()];
    }

    let split_at = chars[(count - 1) as usize].0;
    let mut parts = chars
        .into_iter()
        .take((count - 1) as usize)
        .map(|(_, ch)| ch.to_string())
        .collect::<Vec<_>>();
    parts.push(text[split_at..].to_string());
    parts
}
