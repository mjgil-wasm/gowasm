use super::{StdlibFunction, StdlibMethod, STRINGS_REPLACER_REPLACE};
use crate::{Program, StringsReplacerState, Value, ValueData, Vm, VmError, TYPE_STRINGS_REPLACER};

const REPLACER_ID_FIELD: &str = "__strings_replacer_id";

pub(super) const STRINGS_REPLACER_METHODS: &[StdlibMethod] = &[StdlibMethod {
    receiver_type: "*strings.Replacer",
    method: "Replace",
    function: STRINGS_REPLACER_REPLACE,
}];

pub(super) const STRINGS_REPLACER_METHOD_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    id: STRINGS_REPLACER_REPLACE,
    symbol: "Replace",
    returns_value: true,
    handler: strings_replacer_replace,
}];

pub(super) fn strings_new_replacer(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() % 2 == 1 {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "strings.NewReplacer: odd argument count".into(),
        });
    }
    let pairs = collect_replacer_pairs(vm, program, args)?;
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.string_replacers
        .insert(id, StringsReplacerState { pairs });
    Ok(strings_replacer_value(id))
}

fn strings_replacer_replace(
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
    let replacer_id = extract_replacer_id(vm, program, &args[0])?;
    let text = string_value(vm, program, "strings.(*Replacer).Replace", &args[1])?;
    let replacer =
        vm.string_replacers
            .get(&replacer_id)
            .ok_or(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "strings.(*Replacer).Replace".into(),
                expected: "a valid replacer receiver".into(),
            })?;
    Ok(Value::string(apply_replacer(text, &replacer.pairs)))
}

fn collect_replacer_pairs(
    vm: &Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<(String, String)>, VmError> {
    let mut pairs = Vec::with_capacity(args.len() / 2);
    for pair in args.chunks_exact(2) {
        let old = string_value(vm, program, "strings.NewReplacer", &pair[0])?;
        let new = string_value(vm, program, "strings.NewReplacer", &pair[1])?;
        pairs.push((old.to_string(), new.to_string()));
    }
    Ok(pairs)
}

fn strings_replacer_value(id: u64) -> Value {
    Value {
        typ: TYPE_STRINGS_REPLACER,
        data: ValueData::Struct(vec![(REPLACER_ID_FIELD.into(), Value::int(id as i64))]),
    }
}

fn extract_replacer_id(vm: &Vm, program: &Program, value: &Value) -> Result<u64, VmError> {
    if value.typ != TYPE_STRINGS_REPLACER {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "<unknown>".into()),
            builtin: "strings.(*Replacer).Replace".into(),
            expected: "a valid replacer receiver".into(),
        });
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "<unknown>".into()),
            builtin: "strings.(*Replacer).Replace".into(),
            expected: "a valid replacer receiver".into(),
        });
    };
    fields
        .iter()
        .find(|(name, _)| name == REPLACER_ID_FIELD)
        .and_then(|(_, value)| match value.data {
            ValueData::Int(id) if id > 0 => Some(id as u64),
            _ => None,
        })
        .ok_or(VmError::InvalidStringFunctionArgument {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "<unknown>".into()),
            builtin: "strings.(*Replacer).Replace".into(),
            expected: "a valid replacer receiver".into(),
        })
}

fn string_value<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &'a Value,
) -> Result<&'a str, VmError> {
    let ValueData::String(text) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "<unknown>".into()),
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok(text)
}

fn apply_replacer(text: &str, pairs: &[(String, String)]) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;
    let mut empty_used_at_boundary = false;
    loop {
        if let Some(matched) = replacement_at(text, index, pairs, empty_used_at_boundary) {
            output.push_str(matched.new);
            if matched.old_len == 0 {
                empty_used_at_boundary = true;
            } else {
                index += matched.old_len;
                empty_used_at_boundary = false;
            }
            continue;
        }
        if index == text.len() {
            break;
        }
        let ch = text[index..]
            .chars()
            .next()
            .expect("index should stay on a char boundary");
        output.push(ch);
        index += ch.len_utf8();
        empty_used_at_boundary = false;
    }
    output
}

struct ReplacerMatch<'a> {
    old_len: usize,
    new: &'a str,
}

fn replacement_at<'a>(
    text: &str,
    index: usize,
    pairs: &'a [(String, String)],
    empty_used_at_boundary: bool,
) -> Option<ReplacerMatch<'a>> {
    let tail = &text[index..];
    for (old, new) in pairs {
        if old.is_empty() {
            if !empty_used_at_boundary {
                return Some(ReplacerMatch {
                    old_len: 0,
                    new: new.as_str(),
                });
            }
            continue;
        }
        if tail.starts_with(old) {
            return Some(ReplacerMatch {
                old_len: old.len(),
                new: new.as_str(),
            });
        }
    }
    None
}
