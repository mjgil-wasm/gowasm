use super::bytes_impl::{
    byte_slice_arg, byte_slice_pair_args, bytes_to_value, extract_bytes, find_subslice,
    invalid_bytes_argument,
};
use super::bytes_utf8_impl::{
    byte_runes, equal_fold_bytes, map_runes_to_bytes, titlecase_rune, unicode_space,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) fn bytes_fields(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.Fields", args)?;
    let runes = byte_runes(&data);
    let mut fields = Vec::new();
    let mut start: Option<usize> = None;
    for rune in &runes {
        if unicode_space(rune.ch) {
            if let Some(start_index) = start.take() {
                fields.push(bytes_to_value(&data[start_index..rune.byte_index]));
            }
        } else if start.is_none() {
            start = Some(rune.byte_index);
        }
    }
    if let Some(start_index) = start {
        fields.push(bytes_to_value(&data[start_index..]));
    }
    Ok(Value::slice(fields))
}

pub(super) fn bytes_trim(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (data, cutset) = byte_slice_and_string_args(vm, program, "bytes.Trim", args)?;
    Ok(bytes_to_value(trim_bytes(&data, cutset)))
}

pub(super) fn bytes_trim_left(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, cutset) = byte_slice_and_string_args(vm, program, "bytes.TrimLeft", args)?;
    Ok(bytes_to_value(trim_left_bytes(&data, cutset)))
}

pub(super) fn bytes_trim_right(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, cutset) = byte_slice_and_string_args(vm, program, "bytes.TrimRight", args)?;
    Ok(bytes_to_value(trim_right_bytes(&data, cutset)))
}

pub(super) fn bytes_index_any(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, chars) = byte_slice_and_string_args(vm, program, "bytes.IndexAny", args)?;
    for rune in byte_runes(&data) {
        if chars.contains(rune.ch) {
            return Ok(Value::int(rune.byte_index as i64));
        }
    }
    Ok(Value::int(-1))
}

pub(super) fn bytes_last_index_any(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, chars) = byte_slice_and_string_args(vm, program, "bytes.LastIndexAny", args)?;
    let mut last = -1i64;
    for rune in byte_runes(&data) {
        if chars.contains(rune.ch) {
            last = rune.byte_index as i64;
        }
    }
    Ok(Value::int(last))
}

pub(super) fn bytes_index_rune(
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
    let data = extract_bytes(vm, program, "bytes.IndexRune", &args[0])?;
    let ValueData::Int(rune) = args[1].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            "bytes.IndexRune",
            "[]byte, int",
        )?);
    };
    let Some(ch) = char::from_u32(rune as u32) else {
        return Ok(Value::int(-1));
    };
    for item in byte_runes(&data) {
        if item.ch == ch {
            return Ok(Value::int(item.byte_index as i64));
        }
    }
    Ok(Value::int(-1))
}

pub(super) fn bytes_clone(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.Clone", args)?;
    Ok(bytes_to_value(&data))
}

pub(super) fn bytes_to_title(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.ToTitle", args)?;
    Ok(bytes_to_value(&map_runes_to_bytes(&data, |ch| {
        Some(titlecase_rune(ch))
    })))
}

pub(super) fn bytes_equal_fold(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (left, right) = byte_slice_pair_args(vm, program, "bytes.EqualFold", args)?;
    Ok(Value::bool(equal_fold_bytes(&left, &right)))
}

pub(super) fn bytes_cut_prefix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (data, prefix) = byte_slice_pair_args(vm, program, "bytes.CutPrefix", args)?;
    if data.starts_with(&prefix) {
        Ok(vec![
            bytes_to_value(&data[prefix.len()..]),
            Value::bool(true),
        ])
    } else {
        Ok(vec![bytes_to_value(&data), Value::bool(false)])
    }
}

pub(super) fn bytes_cut_suffix(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (data, suffix) = byte_slice_pair_args(vm, program, "bytes.CutSuffix", args)?;
    if data.ends_with(&suffix) {
        let end = data.len().saturating_sub(suffix.len());
        Ok(vec![bytes_to_value(&data[..end]), Value::bool(true)])
    } else {
        Ok(vec![bytes_to_value(&data), Value::bool(false)])
    }
}

pub(super) fn bytes_cut(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (data, sep) = byte_slice_pair_args(vm, program, "bytes.Cut", args)?;
    if let Some(index) = find_subslice(&data, &sep) {
        let after = index + sep.len();
        Ok(vec![
            bytes_to_value(&data[..index]),
            bytes_to_value(&data[after..]),
            Value::bool(true),
        ])
    } else {
        Ok(vec![
            bytes_to_value(&data),
            bytes_to_value(&[]),
            Value::bool(false),
        ])
    }
}

pub(super) fn trim_space_bytes(data: &[u8]) -> Vec<u8> {
    let runes = byte_runes(data);
    let mut start = data.len();
    for rune in &runes {
        if !unicode_space(rune.ch) {
            start = rune.byte_index;
            break;
        }
    }
    if start == data.len() {
        return Vec::new();
    }
    let mut end = start;
    for rune in runes.iter().rev() {
        if !unicode_space(rune.ch) {
            end = rune.end();
            break;
        }
    }
    data[start..end].to_vec()
}

fn byte_slice_and_string_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(Vec<u8>, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, builtin, &args[0])?;
    let ValueData::String(cutset) = &args[1].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            builtin,
            "[]byte, string",
        )?);
    };
    Ok((data, cutset))
}

fn trim_bytes<'a>(data: &'a [u8], cutset: &str) -> &'a [u8] {
    let trimmed_left = trim_left_bytes(data, cutset);
    trim_right_bytes(trimmed_left, cutset)
}

fn trim_left_bytes<'a>(data: &'a [u8], cutset: &str) -> &'a [u8] {
    let runes = byte_runes(data);
    for rune in &runes {
        if !cutset.contains(rune.ch) {
            return &data[rune.byte_index..];
        }
    }
    &[]
}

fn trim_right_bytes<'a>(data: &'a [u8], cutset: &str) -> &'a [u8] {
    let runes = byte_runes(data);
    for rune in runes.iter().rev() {
        if !cutset.contains(rune.ch) {
            return &data[..rune.end()];
        }
    }
    &[]
}
