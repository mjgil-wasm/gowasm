use super::bytes_impl::{
    byte_slice_pair_args, bytes_to_value, extract_bytes, find_subslice, invalid_bytes_argument,
};
use super::bytes_utf8_impl::{byte_runes, empty_match_boundaries, split_empty_sep_bytes};
use crate::{Program, Value, ValueData, Vm, VmError};

type ByteTripleIntResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>, i64), VmError>;

pub(super) fn bytes_count(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (haystack, needle) = byte_slice_pair_args(vm, program, "bytes.Count", args)?;
    if needle.is_empty() {
        return Ok(Value::int(byte_runes(&haystack).len() as i64 + 1));
    }
    let mut count = 0i64;
    let mut start = 0usize;
    while start + needle.len() <= haystack.len() {
        if haystack[start..start + needle.len()] == needle[..] {
            count += 1;
            start += needle.len();
        } else {
            start += 1;
        }
    }
    Ok(Value::int(count))
}

pub(super) fn bytes_split(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, sep) = byte_slice_pair_args(vm, program, "bytes.Split", args)?;
    Ok(slice_of_byte_slices(split_n_bytes(&data, &sep, -1, false)))
}

pub(super) fn bytes_split_n(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, sep, count) = byte_pair_int_args(vm, program, "bytes.SplitN", args)?;
    if count == 0 {
        return Ok(Value::nil_slice());
    }
    Ok(slice_of_byte_slices(split_n_bytes(
        &data, &sep, count, false,
    )))
}

pub(super) fn bytes_split_after_n(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, sep, count) = byte_pair_int_args(vm, program, "bytes.SplitAfterN", args)?;
    if count == 0 {
        return Ok(Value::nil_slice());
    }
    Ok(slice_of_byte_slices(split_n_bytes(
        &data, &sep, count, true,
    )))
}

pub(super) fn bytes_split_after(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, sep) = byte_slice_pair_args(vm, program, "bytes.SplitAfter", args)?;
    Ok(slice_of_byte_slices(split_n_bytes(&data, &sep, -1, true)))
}

pub(super) fn bytes_replace(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (data, old, new, count) = byte_triple_int_args(vm, program, "bytes.Replace", args)?;
    Ok(bytes_to_value(&replace_n_bytes(&data, &old, &new, count)))
}

pub(super) fn bytes_replace_all(
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
    let data = extract_bytes(vm, program, "bytes.ReplaceAll", &args[0])?;
    let old = extract_bytes(vm, program, "bytes.ReplaceAll", &args[1])?;
    let new = extract_bytes(vm, program, "bytes.ReplaceAll", &args[2])?;
    Ok(bytes_to_value(&replace_n_bytes(&data, &old, &new, -1)))
}

fn byte_pair_int_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<u8>, Vec<u8>, i64), VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, builtin, &args[0])?;
    let sep = extract_bytes(vm, program, builtin, &args[1])?;
    let ValueData::Int(count) = args[2].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            builtin,
            "[]byte, []byte, int",
        )?);
    };
    Ok((data, sep, count))
}

fn byte_triple_int_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> ByteTripleIntResult {
    if args.len() != 4 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 4,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, builtin, &args[0])?;
    let old = extract_bytes(vm, program, builtin, &args[1])?;
    let new = extract_bytes(vm, program, builtin, &args[2])?;
    let ValueData::Int(count) = args[3].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            builtin,
            "[]byte, []byte, []byte, int",
        )?);
    };
    Ok((data, old, new, count))
}

fn split_n_bytes(data: &[u8], sep: &[u8], count: i64, keep_sep: bool) -> Vec<Vec<u8>> {
    if count == 1 {
        return vec![data.to_vec()];
    }
    if sep.is_empty() {
        return split_empty_sep_bytes(data, count);
    }

    let mut parts = Vec::new();
    let mut start = 0usize;
    let unlimited = count < 0;
    let mut remaining = if unlimited {
        usize::MAX
    } else {
        count as usize
    };

    while start <= data.len() {
        if !unlimited && remaining <= 1 {
            break;
        }
        let Some(index) = find_subslice(&data[start..], sep) else {
            break;
        };
        let match_start = start + index;
        let match_end = match_start + sep.len();
        let part_end = if keep_sep { match_end } else { match_start };
        parts.push(data[start..part_end].to_vec());
        start = match_end;
        if !unlimited {
            remaining -= 1;
        }
    }

    parts.push(data[start..].to_vec());
    parts
}

fn replace_n_bytes(data: &[u8], old: &[u8], new: &[u8], count: i64) -> Vec<u8> {
    if count == 0 {
        return data.to_vec();
    }
    if old.is_empty() {
        return replace_empty_matches(data, new, count);
    }

    let mut result = Vec::new();
    let mut start = 0usize;
    let unlimited = count < 0;
    let mut remaining = if unlimited {
        usize::MAX
    } else {
        count as usize
    };

    while start <= data.len() {
        if !unlimited && remaining == 0 {
            break;
        }
        let Some(index) = find_subslice(&data[start..], old) else {
            break;
        };
        let match_start = start + index;
        result.extend_from_slice(&data[start..match_start]);
        result.extend_from_slice(new);
        start = match_start + old.len();
        if !unlimited {
            remaining -= 1;
        }
    }

    result.extend_from_slice(&data[start..]);
    result
}

fn replace_empty_matches(data: &[u8], new: &[u8], count: i64) -> Vec<u8> {
    let boundaries = empty_match_boundaries(data);
    let limit = if count < 0 {
        boundaries.len()
    } else {
        boundaries.len().min(count as usize)
    };
    if limit == 0 {
        return data.to_vec();
    }

    let mut result = Vec::new();
    let mut previous = 0usize;
    for boundary in boundaries.into_iter().take(limit) {
        result.extend_from_slice(&data[previous..boundary]);
        result.extend_from_slice(new);
        previous = boundary;
    }
    result.extend_from_slice(&data[previous..]);
    result
}

fn slice_of_byte_slices(parts: Vec<Vec<u8>>) -> Value {
    Value::slice(
        parts
            .into_iter()
            .map(|part| bytes_to_value(&part))
            .collect(),
    )
}
