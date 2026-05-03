use crate::{FunctionValue, Program, Value, ValueData, Vm, VmError};

#[path = "strings_split_n.rs"]
mod split_n_impl;

#[path = "strings_helpers.rs"]
mod helpers_impl;

use helpers_impl::{
    equal_fold, index_rune, replace_n, string_arg, string_byte_args, string_int_args,
    string_pair_args, string_slice_and_string_args, string_triple_args, string_triple_int_args,
};

pub(super) use split_n_impl::{strings_split_after, strings_split_after_n, strings_split_n};

pub(super) fn strings_contains(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_pair_args(vm, program, "strings.Contains", args)?;
    Ok(Value::bool(text.contains(needle)))
}

pub(super) fn strings_has_prefix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, prefix) = string_pair_args(vm, program, "strings.HasPrefix", args)?;
    Ok(Value::bool(text.starts_with(prefix)))
}

pub(super) fn strings_has_suffix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, suffix) = string_pair_args(vm, program, "strings.HasSuffix", args)?;
    Ok(Value::bool(text.ends_with(suffix)))
}

pub(super) fn strings_trim_space(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.TrimSpace", args)?;
    Ok(Value::string(text.trim()))
}

pub(super) fn strings_to_upper(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.ToUpper", args)?;
    Ok(Value::string(text.to_uppercase()))
}

pub(super) fn strings_to_title(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.ToTitle", args)?;
    let result: String = text.chars().flat_map(|c| c.to_uppercase()).collect();
    Ok(Value::string(result))
}

pub(super) fn strings_to_lower(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.ToLower", args)?;
    Ok(Value::string(text.to_lowercase()))
}

pub(super) fn strings_count(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_pair_args(vm, program, "strings.Count", args)?;
    Ok(Value::int(text.matches(needle).count() as i64))
}

pub(super) fn strings_repeat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, count) = string_int_args(vm, program, "strings.Repeat", args)?;
    if count < 0 {
        return Err(VmError::NegativeRepeatCount {
            function: vm.current_function_name(program)?,
            count,
        });
    }
    Ok(Value::string(text.repeat(count as usize)))
}

pub(super) fn strings_split(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, sep) = string_pair_args(vm, program, "strings.Split", args)?;
    let parts = if sep.is_empty() {
        text.chars()
            .map(|ch| Value::string(ch.to_string()))
            .collect::<Vec<_>>()
    } else {
        text.split(sep).map(Value::string).collect::<Vec<_>>()
    };
    Ok(Value::slice(parts))
}

pub(super) fn strings_join(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (parts, sep) = string_slice_and_string_args(vm, program, args)?;
    Ok(Value::string(parts.join(sep)))
}

pub(super) fn strings_replace_all(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, from, to) = string_triple_args(vm, program, "strings.ReplaceAll", args)?;
    Ok(Value::string(text.replace(from, to)))
}

pub(super) fn strings_cut_prefix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (text, prefix) = string_pair_args(vm, program, "strings.CutPrefix", args)?;
    if let Some(after) = text.strip_prefix(prefix) {
        Ok(vec![Value::string(after), Value::bool(true)])
    } else {
        Ok(vec![Value::string(text), Value::bool(false)])
    }
}

pub(super) fn strings_cut(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (text, sep) = string_pair_args(vm, program, "strings.Cut", args)?;
    if let Some((before, after)) = text.split_once(sep) {
        Ok(vec![
            Value::string(before),
            Value::string(after),
            Value::bool(true),
        ])
    } else {
        Ok(vec![
            Value::string(text),
            Value::string(""),
            Value::bool(false),
        ])
    }
}

pub(super) fn strings_cut_suffix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (text, suffix) = string_pair_args(vm, program, "strings.CutSuffix", args)?;
    if let Some(before) = text.strip_suffix(suffix) {
        Ok(vec![Value::string(before), Value::bool(true)])
    } else {
        Ok(vec![Value::string(text), Value::bool(false)])
    }
}

pub(super) fn strings_fields(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.Fields", args)?;
    Ok(Value::slice(
        text.split_whitespace()
            .map(Value::string)
            .collect::<Vec<_>>(),
    ))
}

pub(super) fn strings_index(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_pair_args(vm, program, "strings.Index", args)?;
    Ok(Value::int(
        text.find(needle).map(|index| index as i64).unwrap_or(-1),
    ))
}

pub(super) fn strings_trim_prefix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, prefix) = string_pair_args(vm, program, "strings.TrimPrefix", args)?;
    Ok(Value::string(text.strip_prefix(prefix).unwrap_or(text)))
}

pub(super) fn strings_trim_suffix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, suffix) = string_pair_args(vm, program, "strings.TrimSuffix", args)?;
    Ok(Value::string(text.strip_suffix(suffix).unwrap_or(text)))
}

pub(super) fn strings_last_index(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_pair_args(vm, program, "strings.LastIndex", args)?;
    Ok(Value::int(
        text.rfind(needle).map(|index| index as i64).unwrap_or(-1),
    ))
}

pub(super) fn strings_trim_left(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, cutset) = string_pair_args(vm, program, "strings.TrimLeft", args)?;
    Ok(Value::string(
        text.trim_start_matches(|ch| cutset.contains(ch)),
    ))
}

pub(super) fn strings_trim_right(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, cutset) = string_pair_args(vm, program, "strings.TrimRight", args)?;
    Ok(Value::string(
        text.trim_end_matches(|ch| cutset.contains(ch)),
    ))
}

pub(super) fn strings_trim(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, cutset) = string_pair_args(vm, program, "strings.Trim", args)?;
    Ok(Value::string(text.trim_matches(|ch| cutset.contains(ch))))
}

pub(super) fn strings_contains_any(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, chars) = string_pair_args(vm, program, "strings.ContainsAny", args)?;
    Ok(Value::bool(text.chars().any(|ch| chars.contains(ch))))
}

pub(super) fn strings_index_any(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, chars) = string_pair_args(vm, program, "strings.IndexAny", args)?;
    Ok(Value::int(
        text.char_indices()
            .find(|(_, ch)| chars.contains(*ch))
            .map(|(index, _)| index as i64)
            .unwrap_or(-1),
    ))
}

pub(super) fn strings_last_index_any(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, chars) = string_pair_args(vm, program, "strings.LastIndexAny", args)?;
    Ok(Value::int(
        text.char_indices()
            .rev()
            .find(|(_, ch)| chars.contains(*ch))
            .map(|(index, _)| index as i64)
            .unwrap_or(-1),
    ))
}

pub(super) fn strings_clone(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let text = string_arg(vm, program, "strings.Clone", args)?;
    Ok(Value::string(text.to_string()))
}

pub(super) fn strings_contains_rune(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, rune) = string_int_args(vm, program, "strings.ContainsRune", args)?;
    Ok(Value::bool(index_rune(text, rune) >= 0))
}

pub(super) fn strings_index_rune(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, rune) = string_int_args(vm, program, "strings.IndexRune", args)?;
    Ok(Value::int(index_rune(text, rune)))
}

pub(super) fn strings_compare(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (left, right) = string_pair_args(vm, program, "strings.Compare", args)?;
    let value = match left.cmp(right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    Ok(Value::int(value))
}

pub(super) fn strings_equal_fold(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (left, right) = string_pair_args(vm, program, "strings.EqualFold", args)?;
    Ok(Value::bool(equal_fold(left, right)))
}

pub(super) fn strings_replace(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, old, new, count) = string_triple_int_args(vm, program, "strings.Replace", args)?;
    Ok(Value::string(replace_n(text, old, new, count)))
}

pub(super) fn strings_index_byte(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_byte_args(vm, program, "strings.IndexByte", args)?;
    Ok(Value::int(
        text.as_bytes()
            .iter()
            .position(|byte| *byte == needle)
            .map(|index| index as i64)
            .unwrap_or(-1),
    ))
}

pub(super) fn strings_last_index_byte(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, needle) = string_byte_args(vm, program, "strings.LastIndexByte", args)?;
    Ok(Value::int(
        text.as_bytes()
            .iter()
            .rposition(|byte| *byte == needle)
            .map(|index| index as i64)
            .unwrap_or(-1),
    ))
}

pub(super) fn strings_map(
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
    let mapping = match &args[0].data {
        ValueData::Function(f) => f.clone(),
        _ => {
            return Err(VmError::InvalidFunctionValue {
                function: vm.current_function_name(program)?,
                target: crate::describe_value(&args[0]),
            });
        }
    };
    let text = match &args[1].data {
        ValueData::String(s) => s.as_str(),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: "strings.Map".into(),
                expected: "string argument".into(),
            });
        }
    };
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        let mapped = invoke_rune_callback(vm, program, &mapping, ch)?;
        if mapped >= 0 {
            if let Some(c) = char::from_u32(mapped as u32) {
                result.push(c);
            }
        }
    }
    Ok(Value::string(result))
}

fn extract_string_and_func(
    vm: &mut Vm,
    program: &Program,
    builtin_name: &str,
    args: &[Value],
) -> Result<(String, FunctionValue), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let text = match &args[0].data {
        ValueData::String(s) => s.clone(),
        _ => {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin_name.into(),
                expected: "string argument".into(),
            });
        }
    };
    let func = match &args[1].data {
        ValueData::Function(f) => f.clone(),
        _ => {
            return Err(VmError::InvalidFunctionValue {
                function: vm.current_function_name(program)?,
                target: crate::describe_value(&args[1]),
            });
        }
    };
    Ok((text, func))
}

fn invoke_rune_callback(
    vm: &mut Vm,
    program: &Program,
    mapping: &FunctionValue,
    ch: char,
) -> Result<i64, VmError> {
    let mut callback_args = mapping.captures.clone();
    callback_args.push(Value::int(ch as i64));
    let result = vm.invoke_callback(program, mapping.function, callback_args)?;
    match result.data {
        ValueData::Int(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strings.Map callback".into(),
            expected: "int return value".into(),
        }),
    }
}

fn invoke_rune_predicate(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    ch: char,
) -> Result<bool, VmError> {
    let mut callback_args = predicate.captures.clone();
    callback_args.push(Value::int(ch as i64));
    let result = vm.invoke_callback(program, predicate.function, callback_args)?;
    match result.data {
        ValueData::Bool(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "callback".into(),
            expected: "bool return value".into(),
        }),
    }
}

pub(super) fn strings_index_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.IndexFunc", args)?;
    for (byte_idx, ch) in text.char_indices() {
        if invoke_rune_predicate(vm, program, &predicate, ch)? {
            return Ok(Value::int(byte_idx as i64));
        }
    }
    Ok(Value::int(-1))
}

pub(super) fn strings_last_index_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.LastIndexFunc", args)?;
    let mut last: i64 = -1;
    for (byte_idx, ch) in text.char_indices() {
        if invoke_rune_predicate(vm, program, &predicate, ch)? {
            last = byte_idx as i64;
        }
    }
    Ok(Value::int(last))
}

pub(super) fn strings_trim_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.TrimFunc", args)?;
    let left = trim_left_with_predicate(vm, program, &predicate, &text)?;
    let trimmed = trim_right_with_predicate(vm, program, &predicate, left)?;
    Ok(Value::string(trimmed.to_string()))
}

pub(super) fn strings_trim_left_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.TrimLeftFunc", args)?;
    let trimmed = trim_left_with_predicate(vm, program, &predicate, &text)?;
    Ok(Value::string(trimmed.to_string()))
}

pub(super) fn strings_trim_right_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.TrimRightFunc", args)?;
    let trimmed = trim_right_with_predicate(vm, program, &predicate, &text)?;
    Ok(Value::string(trimmed.to_string()))
}

fn trim_left_with_predicate<'a>(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    text: &'a str,
) -> Result<&'a str, VmError> {
    for (byte_idx, ch) in text.char_indices() {
        if !invoke_rune_predicate(vm, program, predicate, ch)? {
            return Ok(&text[byte_idx..]);
        }
    }
    Ok("")
}

fn trim_right_with_predicate<'a>(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    text: &'a str,
) -> Result<&'a str, VmError> {
    for (byte_idx, ch) in text.char_indices().rev() {
        if !invoke_rune_predicate(vm, program, predicate, ch)? {
            return Ok(&text[..byte_idx + ch.len_utf8()]);
        }
    }
    Ok("")
}

pub(super) fn strings_fields_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (text, predicate) = extract_string_and_func(vm, program, "strings.FieldsFunc", args)?;
    let mut fields: Vec<Value> = Vec::new();
    let mut start: Option<usize> = None;
    for (byte_idx, ch) in text.char_indices() {
        let is_separator = invoke_rune_predicate(vm, program, &predicate, ch)?;
        match (is_separator, start) {
            (true, Some(s)) => {
                fields.push(Value::string(text[s..byte_idx].to_string()));
                start = None;
            }
            (false, None) => {
                start = Some(byte_idx);
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        fields.push(Value::string(text[s..].to_string()));
    }
    Ok(Value::slice(fields))
}
