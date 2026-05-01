use super::bytes_impl::{bytes_to_value, extract_bytes};
use super::bytes_utf8_impl::{byte_runes, map_runes_to_bytes, ByteRune};
use crate::{describe_value, FunctionValue, Program, Value, ValueData, Vm, VmError};

pub(super) fn bytes_map(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Function(mapping) = &args[0].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[0]),
        });
    };
    let mapping = mapping.clone();
    let data = extract_bytes(vm, program, "bytes.Map", &args[1])?;
    let mut callback_error = None;
    let result = map_runes_to_bytes(&data, |ch| {
        let mut callback_args = mapping.captures.clone();
        callback_args.push(Value::int(ch as i64));
        let mapped = match vm.invoke_callback(program, mapping.function, callback_args) {
            Ok(value) => value,
            Err(err) => {
                callback_error = Some(err);
                return None;
            }
        };
        let ValueData::Int(v) = mapped.data else {
            callback_error = Some(VmError::InvalidStringFunctionArgument {
                function: vm
                    .current_function_name(program)
                    .expect("bytes.Map callback error should describe the current function"),
                builtin: "bytes.Map callback".into(),
                expected: "int return value".into(),
            });
            return None;
        };
        if v < 0 {
            return None;
        }
        char::from_u32(v as u32)
    });
    if let Some(err) = callback_error {
        return Err(err);
    }
    Ok(bytes_to_value(&result))
}

pub(super) fn bytes_index_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.IndexFunc", args)?;
    for rune in byte_runes(&data) {
        if invoke_byte_rune_predicate(vm, program, &predicate, rune.ch)? {
            return Ok(Value::int(rune.byte_index as i64));
        }
    }
    Ok(Value::int(-1))
}

pub(super) fn bytes_last_index_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.LastIndexFunc", args)?;
    let mut last: i64 = -1;
    for rune in byte_runes(&data) {
        if invoke_byte_rune_predicate(vm, program, &predicate, rune.ch)? {
            last = rune.byte_index as i64;
        }
    }
    Ok(Value::int(last))
}

pub(super) fn bytes_trim_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.TrimFunc", args)?;
    let runes = byte_runes(&data);
    let start = trim_left_boundary(vm, program, &predicate, &runes, data.len())?;
    let end = trim_right_boundary(vm, program, &predicate, &runes, start)?;
    Ok(bytes_to_value(&data[start..end]))
}

pub(super) fn bytes_trim_left_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.TrimLeftFunc", args)?;
    let runes = byte_runes(&data);
    let start = trim_left_boundary(vm, program, &predicate, &runes, data.len())?;
    Ok(bytes_to_value(&data[start..]))
}

pub(super) fn bytes_trim_right_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.TrimRightFunc", args)?;
    let runes = byte_runes(&data);
    let end = trim_right_boundary(vm, program, &predicate, &runes, 0)?;
    Ok(bytes_to_value(&data[..end]))
}

pub(super) fn bytes_fields_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, predicate) = bytes_and_func_args(vm, program, "bytes.FieldsFunc", args)?;
    let mut fields: Vec<Value> = Vec::new();
    let mut start: Option<usize> = None;
    for rune in byte_runes(&data) {
        let is_separator = invoke_byte_rune_predicate(vm, program, &predicate, rune.ch)?;
        match (is_separator, start) {
            (true, Some(s)) => {
                fields.push(bytes_to_value(&data[s..rune.byte_index]));
                start = None;
            }
            (false, None) => {
                start = Some(rune.byte_index);
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        fields.push(bytes_to_value(&data[s..]));
    }
    Ok(Value::slice(fields))
}

fn bytes_and_func_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<u8>, FunctionValue), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, builtin, &args[0])?;
    let ValueData::Function(f) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[1]),
        });
    };
    Ok((data, f.clone()))
}

fn invoke_byte_rune_predicate(
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

fn trim_left_boundary(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    runes: &[ByteRune],
    default: usize,
) -> Result<usize, VmError> {
    for rune in runes {
        if !invoke_byte_rune_predicate(vm, program, predicate, rune.ch)? {
            return Ok(rune.byte_index);
        }
    }
    Ok(default)
}

fn trim_right_boundary(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    runes: &[ByteRune],
    default: usize,
) -> Result<usize, VmError> {
    for rune in runes.iter().rev() {
        if !invoke_byte_rune_predicate(vm, program, predicate, rune.ch)? {
            return Ok(rune.end());
        }
    }
    Ok(default)
}
