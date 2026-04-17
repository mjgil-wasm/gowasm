use super::{
    StdlibFunction, StdlibMethod, REGEXP_COMPILE, REGEXP_MATCH_STRING, REGEXP_MUST_COMPILE,
    REGEXP_QUOTE_META, REGEXP_REGEXP_FIND_STRING, REGEXP_REGEXP_FIND_STRING_SUBMATCH,
    REGEXP_REGEXP_MATCH_STRING, REGEXP_REGEXP_REPLACE_ALL_STRING, REGEXP_REGEXP_SPLIT,
};
use crate::{ConcreteType, Program, Value, ValueData, Vm, VmError, TYPE_REGEXP, TYPE_STRING};

pub(super) const REGEXP_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: REGEXP_QUOTE_META,
        symbol: "QuoteMeta",
        returns_value: true,
        handler: regexp_quote_meta,
    },
    StdlibFunction {
        id: REGEXP_MATCH_STRING,
        symbol: "MatchString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: REGEXP_COMPILE,
        symbol: "Compile",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: REGEXP_MUST_COMPILE,
        symbol: "MustCompile",
        returns_value: true,
        handler: regexp_must_compile,
    },
];

pub(super) const REGEXP_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "*regexp.Regexp",
        method: "MatchString",
        function: REGEXP_REGEXP_MATCH_STRING,
    },
    StdlibMethod {
        receiver_type: "*regexp.Regexp",
        method: "FindString",
        function: REGEXP_REGEXP_FIND_STRING,
    },
    StdlibMethod {
        receiver_type: "*regexp.Regexp",
        method: "ReplaceAllString",
        function: REGEXP_REGEXP_REPLACE_ALL_STRING,
    },
    StdlibMethod {
        receiver_type: "*regexp.Regexp",
        method: "FindStringSubmatch",
        function: REGEXP_REGEXP_FIND_STRING_SUBMATCH,
    },
    StdlibMethod {
        receiver_type: "*regexp.Regexp",
        method: "Split",
        function: REGEXP_REGEXP_SPLIT,
    },
];

pub(super) const REGEXP_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: REGEXP_REGEXP_MATCH_STRING,
        symbol: "MatchString",
        returns_value: true,
        handler: regexp_regexp_match_string,
    },
    StdlibFunction {
        id: REGEXP_REGEXP_FIND_STRING,
        symbol: "FindString",
        returns_value: true,
        handler: regexp_regexp_find_string,
    },
    StdlibFunction {
        id: REGEXP_REGEXP_REPLACE_ALL_STRING,
        symbol: "ReplaceAllString",
        returns_value: true,
        handler: regexp_regexp_replace_all_string,
    },
    StdlibFunction {
        id: REGEXP_REGEXP_FIND_STRING_SUBMATCH,
        symbol: "FindStringSubmatch",
        returns_value: true,
        handler: regexp_regexp_find_string_submatch,
    },
    StdlibFunction {
        id: REGEXP_REGEXP_SPLIT,
        symbol: "Split",
        returns_value: true,
        handler: regexp_regexp_split,
    },
];

fn regexp_quote_meta(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(s) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.QuoteMeta".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(Value::string(regex::escape(s)))
}

pub(super) fn regexp_match_string(
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
    let ValueData::String(pattern) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.MatchString".into(),
            expected: "two string arguments".into(),
        });
    };
    let ValueData::String(s) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.MatchString".into(),
            expected: "two string arguments".into(),
        });
    };
    match regex::Regex::new(pattern) {
        Ok(re) => Ok(vec![Value::bool(re.is_match(s)), Value::nil()]),
        Err(e) => Ok(vec![
            Value::bool(false),
            Value::error(compile_error_text(pattern, &e)),
        ]),
    }
}

pub(super) fn regexp_compile(
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
    let ValueData::String(pattern) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.Compile".into(),
            expected: "a string argument".into(),
        });
    };
    match regex::Regex::new(pattern) {
        Ok(compiled) => Ok(vec![store_compiled_regexp(vm, compiled), Value::nil()]),
        Err(e) => Ok(vec![
            Value::nil(),
            Value::error(compile_error_text(pattern, &e)),
        ]),
    }
}

fn regexp_must_compile(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(pattern) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.MustCompile".into(),
            expected: "a string argument".into(),
        });
    };
    match regex::Regex::new(pattern) {
        Ok(compiled) => Ok(store_compiled_regexp(vm, compiled)),
        Err(e) => Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: compile_error_text(pattern, &e),
        }),
    }
}

fn regexp_regexp_match_string(
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
    let regexp_id = extract_compiled_regexp_id(vm, program, &args[0], "MatchString")?;
    let ValueData::String(s) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).MatchString".into(),
            expected: "a string argument".into(),
        });
    };
    let regexp = compiled_regexp(vm, program, regexp_id, "MatchString")?;
    Ok(Value::bool(regexp.is_match(s)))
}

fn regexp_regexp_find_string(
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
    let regexp_id = extract_compiled_regexp_id(vm, program, &args[0], "FindString")?;
    let ValueData::String(s) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).FindString".into(),
            expected: "a string argument".into(),
        });
    };
    let regexp = compiled_regexp(vm, program, regexp_id, "FindString")?;
    Ok(Value::string(
        regexp.find(s).map(|m| m.as_str()).unwrap_or_default(),
    ))
}

fn regexp_regexp_replace_all_string(
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
    let regexp_id = extract_compiled_regexp_id(vm, program, &args[0], "ReplaceAllString")?;
    let ValueData::String(src) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).ReplaceAllString".into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(repl) = &args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).ReplaceAllString".into(),
            expected: "string arguments".into(),
        });
    };
    let regexp = compiled_regexp(vm, program, regexp_id, "ReplaceAllString")?;
    Ok(Value::string(
        regexp.replace_all(src, repl.as_str()).into_owned(),
    ))
}

fn regexp_regexp_find_string_submatch(
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
    let regexp_id = extract_compiled_regexp_id(vm, program, &args[0], "FindStringSubmatch")?;
    let ValueData::String(s) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).FindStringSubmatch".into(),
            expected: "a string argument".into(),
        });
    };
    let regexp = compiled_regexp(vm, program, regexp_id, "FindStringSubmatch")?;
    let Some(captures) = regexp.captures(s) else {
        return Ok(string_slice_nil_value());
    };
    let values = captures
        .iter()
        .map(|capture| Value::string(capture.map(|m| m.as_str()).unwrap_or_default()))
        .collect::<Vec<_>>();
    Ok(string_slice_value(values))
}

fn regexp_regexp_split(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let regexp_id = extract_compiled_regexp_id(vm, program, &args[0], "Split")?;
    let ValueData::String(s) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).Split".into(),
            expected: "a string argument plus an int count".into(),
        });
    };
    let ValueData::Int(count) = args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "regexp.(*Regexp).Split".into(),
            expected: "a string argument plus an int count".into(),
        });
    };
    if count == 0 {
        return Ok(string_slice_nil_value());
    }
    let regexp = compiled_regexp(vm, program, regexp_id, "Split")?;
    let parts = if regexp.as_str().is_empty() {
        split_empty_regexp(s, count)
            .into_iter()
            .map(Value::string)
            .collect::<Vec<_>>()
    } else if count < 0 {
        regexp.split(s).map(Value::string).collect::<Vec<_>>()
    } else {
        regexp
            .splitn(s, count as usize)
            .map(Value::string)
            .collect::<Vec<_>>()
    };
    Ok(string_slice_value(parts))
}

fn store_compiled_regexp(vm: &mut Vm, compiled: regex::Regex) -> Value {
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.compiled_regexps.insert(id, compiled);
    compiled_regexp_value(id)
}

fn compile_error_text(pattern: &str, error: &regex::Error) -> String {
    format!("regexp: Compile(`{pattern}`): {error}")
}

fn compiled_regexp_value(id: u64) -> Value {
    Value {
        typ: TYPE_REGEXP,
        data: ValueData::Struct(vec![("__regexp_id".into(), Value::int(id as i64))]),
    }
}

fn string_slice_value(values: Vec<Value>) -> Value {
    Value::slice_typed(
        values,
        ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        },
    )
}

fn string_slice_nil_value() -> Value {
    Value::nil_slice_typed(ConcreteType::Slice {
        element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
    })
}

fn split_empty_regexp(text: &str, count: i64) -> Vec<String> {
    let chars = text.char_indices().collect::<Vec<_>>();
    if count == 1 {
        return vec![text.to_string()];
    }
    if count < 0 || count as usize >= chars.len() {
        return text.chars().map(|ch| ch.to_string()).collect();
    }
    let split_at = chars[(count - 1) as usize].0;
    let mut parts = chars
        .iter()
        .take((count - 1) as usize)
        .map(|(_, ch)| ch.to_string())
        .collect::<Vec<_>>();
    parts.push(text[split_at..].to_string());
    parts
}

fn compiled_regexp<'a>(
    vm: &'a Vm,
    program: &Program,
    regexp_id: u64,
    method: &str,
) -> Result<&'a regex::Regex, VmError> {
    vm.compiled_regexps
        .get(&regexp_id)
        .ok_or(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: format!("regexp.(*Regexp).{method}"),
            expected: "a compiled regexp receiver".into(),
        })
}

fn extract_compiled_regexp_id(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    method: &str,
) -> Result<u64, VmError> {
    if value.typ != TYPE_REGEXP {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: format!("regexp.(*Regexp).{method}"),
            expected: "a compiled regexp receiver".into(),
        });
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: format!("regexp.(*Regexp).{method}"),
            expected: "a compiled regexp receiver".into(),
        });
    };
    fields
        .iter()
        .find(|(name, _)| name == "__regexp_id")
        .and_then(|(_, value)| match value.data {
            ValueData::Int(id) => Some(id as u64),
            _ => None,
        })
        .ok_or(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: format!("regexp.(*Regexp).{method}"),
            expected: "a compiled regexp receiver".into(),
        })
}
