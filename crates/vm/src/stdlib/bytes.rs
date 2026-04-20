use super::{
    bytes_callback_impl, bytes_more_impl, bytes_split_impl, StdlibFunction, BYTES_CLONE,
    BYTES_COMPARE, BYTES_CONTAINS, BYTES_CONTAINS_ANY, BYTES_CONTAINS_RUNE, BYTES_COUNT, BYTES_CUT,
    BYTES_CUT_PREFIX, BYTES_CUT_SUFFIX, BYTES_EQUAL, BYTES_EQUAL_FOLD, BYTES_FIELDS,
    BYTES_FIELDS_FUNC, BYTES_HAS_PREFIX, BYTES_HAS_SUFFIX, BYTES_INDEX, BYTES_INDEX_ANY,
    BYTES_INDEX_BYTE, BYTES_INDEX_FUNC, BYTES_INDEX_RUNE, BYTES_JOIN, BYTES_LAST_INDEX,
    BYTES_LAST_INDEX_ANY, BYTES_LAST_INDEX_BYTE, BYTES_LAST_INDEX_FUNC, BYTES_MAP, BYTES_REPEAT,
    BYTES_REPLACE, BYTES_REPLACE_ALL, BYTES_SPLIT, BYTES_SPLIT_AFTER, BYTES_SPLIT_AFTER_N,
    BYTES_SPLIT_N, BYTES_TO_LOWER, BYTES_TO_TITLE, BYTES_TO_UPPER, BYTES_TRIM, BYTES_TRIM_FUNC,
    BYTES_TRIM_LEFT, BYTES_TRIM_LEFT_FUNC, BYTES_TRIM_RIGHT, BYTES_TRIM_RIGHT_FUNC,
    BYTES_TRIM_SPACE,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const BYTES_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: BYTES_CONTAINS,
        symbol: "Contains",
        returns_value: true,
        handler: bytes_contains,
    },
    StdlibFunction {
        id: BYTES_EQUAL,
        symbol: "Equal",
        returns_value: true,
        handler: bytes_equal,
    },
    StdlibFunction {
        id: BYTES_HAS_PREFIX,
        symbol: "HasPrefix",
        returns_value: true,
        handler: bytes_has_prefix,
    },
    StdlibFunction {
        id: BYTES_HAS_SUFFIX,
        symbol: "HasSuffix",
        returns_value: true,
        handler: bytes_has_suffix,
    },
    StdlibFunction {
        id: BYTES_INDEX,
        symbol: "Index",
        returns_value: true,
        handler: bytes_index,
    },
    StdlibFunction {
        id: BYTES_LAST_INDEX,
        symbol: "LastIndex",
        returns_value: true,
        handler: bytes_last_index,
    },
    StdlibFunction {
        id: BYTES_INDEX_BYTE,
        symbol: "IndexByte",
        returns_value: true,
        handler: bytes_index_byte,
    },
    StdlibFunction {
        id: BYTES_LAST_INDEX_BYTE,
        symbol: "LastIndexByte",
        returns_value: true,
        handler: bytes_last_index_byte,
    },
    StdlibFunction {
        id: BYTES_COUNT,
        symbol: "Count",
        returns_value: true,
        handler: bytes_split_impl::bytes_count,
    },
    StdlibFunction {
        id: BYTES_REPEAT,
        symbol: "Repeat",
        returns_value: true,
        handler: bytes_repeat,
    },
    StdlibFunction {
        id: BYTES_JOIN,
        symbol: "Join",
        returns_value: true,
        handler: bytes_join,
    },
    StdlibFunction {
        id: BYTES_SPLIT,
        symbol: "Split",
        returns_value: true,
        handler: bytes_split_impl::bytes_split,
    },
    StdlibFunction {
        id: BYTES_SPLIT_N,
        symbol: "SplitN",
        returns_value: true,
        handler: bytes_split_impl::bytes_split_n,
    },
    StdlibFunction {
        id: BYTES_SPLIT_AFTER_N,
        symbol: "SplitAfterN",
        returns_value: true,
        handler: bytes_split_impl::bytes_split_after_n,
    },
    StdlibFunction {
        id: BYTES_SPLIT_AFTER,
        symbol: "SplitAfter",
        returns_value: true,
        handler: bytes_split_impl::bytes_split_after,
    },
    StdlibFunction {
        id: BYTES_REPLACE,
        symbol: "Replace",
        returns_value: true,
        handler: bytes_split_impl::bytes_replace,
    },
    StdlibFunction {
        id: BYTES_REPLACE_ALL,
        symbol: "ReplaceAll",
        returns_value: true,
        handler: bytes_split_impl::bytes_replace_all,
    },
    StdlibFunction {
        id: BYTES_FIELDS,
        symbol: "Fields",
        returns_value: true,
        handler: bytes_more_impl::bytes_fields,
    },
    StdlibFunction {
        id: BYTES_TRIM_SPACE,
        symbol: "TrimSpace",
        returns_value: true,
        handler: bytes_trim_space,
    },
    StdlibFunction {
        id: BYTES_TO_UPPER,
        symbol: "ToUpper",
        returns_value: true,
        handler: bytes_to_upper,
    },
    StdlibFunction {
        id: BYTES_TO_LOWER,
        symbol: "ToLower",
        returns_value: true,
        handler: bytes_to_lower,
    },
    StdlibFunction {
        id: BYTES_TO_TITLE,
        symbol: "ToTitle",
        returns_value: true,
        handler: bytes_more_impl::bytes_to_title,
    },
    StdlibFunction {
        id: BYTES_COMPARE,
        symbol: "Compare",
        returns_value: true,
        handler: bytes_compare,
    },
    StdlibFunction {
        id: BYTES_CONTAINS_ANY,
        symbol: "ContainsAny",
        returns_value: true,
        handler: bytes_contains_any,
    },
    StdlibFunction {
        id: BYTES_CONTAINS_RUNE,
        symbol: "ContainsRune",
        returns_value: true,
        handler: bytes_contains_rune,
    },
    StdlibFunction {
        id: BYTES_INDEX_ANY,
        symbol: "IndexAny",
        returns_value: true,
        handler: bytes_more_impl::bytes_index_any,
    },
    StdlibFunction {
        id: BYTES_LAST_INDEX_ANY,
        symbol: "LastIndexAny",
        returns_value: true,
        handler: bytes_more_impl::bytes_last_index_any,
    },
    StdlibFunction {
        id: BYTES_INDEX_RUNE,
        symbol: "IndexRune",
        returns_value: true,
        handler: bytes_more_impl::bytes_index_rune,
    },
    StdlibFunction {
        id: BYTES_CLONE,
        symbol: "Clone",
        returns_value: true,
        handler: bytes_more_impl::bytes_clone,
    },
    StdlibFunction {
        id: BYTES_EQUAL_FOLD,
        symbol: "EqualFold",
        returns_value: true,
        handler: bytes_more_impl::bytes_equal_fold,
    },
    StdlibFunction {
        id: BYTES_TRIM,
        symbol: "Trim",
        returns_value: true,
        handler: bytes_more_impl::bytes_trim,
    },
    StdlibFunction {
        id: BYTES_TRIM_LEFT,
        symbol: "TrimLeft",
        returns_value: true,
        handler: bytes_more_impl::bytes_trim_left,
    },
    StdlibFunction {
        id: BYTES_TRIM_RIGHT,
        symbol: "TrimRight",
        returns_value: true,
        handler: bytes_more_impl::bytes_trim_right,
    },
    StdlibFunction {
        id: BYTES_INDEX_FUNC,
        symbol: "IndexFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_index_func,
    },
    StdlibFunction {
        id: BYTES_LAST_INDEX_FUNC,
        symbol: "LastIndexFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_last_index_func,
    },
    StdlibFunction {
        id: BYTES_TRIM_FUNC,
        symbol: "TrimFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_trim_func,
    },
    StdlibFunction {
        id: BYTES_TRIM_LEFT_FUNC,
        symbol: "TrimLeftFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_trim_left_func,
    },
    StdlibFunction {
        id: BYTES_TRIM_RIGHT_FUNC,
        symbol: "TrimRightFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_trim_right_func,
    },
    StdlibFunction {
        id: BYTES_FIELDS_FUNC,
        symbol: "FieldsFunc",
        returns_value: true,
        handler: bytes_callback_impl::bytes_fields_func,
    },
    StdlibFunction {
        id: BYTES_MAP,
        symbol: "Map",
        returns_value: true,
        handler: bytes_callback_impl::bytes_map,
    },
    StdlibFunction {
        id: BYTES_CUT_PREFIX,
        symbol: "CutPrefix",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: BYTES_CUT_SUFFIX,
        symbol: "CutSuffix",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: BYTES_CUT,
        symbol: "Cut",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

fn bytes_contains(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (haystack, needle) = byte_slice_pair_args(vm, program, "bytes.Contains", args)?;
    Ok(Value::bool(contains_subslice(&haystack, &needle)))
}

fn bytes_equal(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (a, b) = byte_slice_pair_args(vm, program, "bytes.Equal", args)?;
    Ok(Value::bool(a == b))
}

fn bytes_has_prefix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (data, prefix) = byte_slice_pair_args(vm, program, "bytes.HasPrefix", args)?;
    Ok(Value::bool(data.starts_with(&prefix)))
}

fn bytes_has_suffix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (data, suffix) = byte_slice_pair_args(vm, program, "bytes.HasSuffix", args)?;
    Ok(Value::bool(data.ends_with(&suffix)))
}

fn bytes_index(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (haystack, needle) = byte_slice_pair_args(vm, program, "bytes.Index", args)?;
    let pos = find_subslice(&haystack, &needle);
    Ok(Value::int(pos.map(|i| i as i64).unwrap_or(-1)))
}

fn bytes_last_index(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (haystack, needle) = byte_slice_pair_args(vm, program, "bytes.LastIndex", args)?;
    let pos = rfind_subslice(&haystack, &needle);
    Ok(Value::int(pos.map(|i| i as i64).unwrap_or(-1)))
}

fn bytes_index_byte(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (data, byte_val) = byte_slice_and_int_args(vm, program, "bytes.IndexByte", args)?;
    let pos = data.iter().position(|&b| b == byte_val);
    Ok(Value::int(pos.map(|i| i as i64).unwrap_or(-1)))
}

fn bytes_last_index_byte(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (data, byte_val) = byte_slice_and_int_args(vm, program, "bytes.LastIndexByte", args)?;
    let pos = data.iter().rposition(|&b| b == byte_val);
    Ok(Value::int(pos.map(|i| i as i64).unwrap_or(-1)))
}

fn bytes_repeat(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, "bytes.Repeat", &args[0])?;
    let ValueData::Int(count) = args[1].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            "bytes.Repeat",
            "[]byte, int",
        )?);
    };
    if count < 0 {
        return Err(VmError::NegativeRepeatCount {
            function: vm.current_function_name(program)?,
            count,
        });
    }
    let repeated: Vec<u8> = data.repeat(count as usize);
    Ok(bytes_to_value(&repeated))
}

fn bytes_join(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let slices = byte_slice_of_slices_arg(vm, program, "bytes.Join", &args[0])?;
    let sep = extract_bytes(vm, program, "bytes.Join", &args[1])?;
    let mut result = Vec::new();
    for (i, slice) in slices.iter().enumerate() {
        if i > 0 {
            result.extend_from_slice(&sep);
        }
        result.extend_from_slice(slice);
    }
    Ok(bytes_to_value(&result))
}

fn bytes_trim_space(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.TrimSpace", args)?;
    Ok(bytes_to_value(&bytes_more_impl::trim_space_bytes(&data)))
}

fn bytes_to_upper(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.ToUpper", args)?;
    let s = String::from_utf8_lossy(&data);
    Ok(bytes_to_value(s.to_uppercase().as_bytes()))
}

fn bytes_to_lower(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let data = byte_slice_arg(vm, program, "bytes.ToLower", args)?;
    let s = String::from_utf8_lossy(&data);
    Ok(bytes_to_value(s.to_lowercase().as_bytes()))
}

fn bytes_compare(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (a, b) = byte_slice_pair_args(vm, program, "bytes.Compare", args)?;
    let result = a.cmp(&b);
    Ok(Value::int(match result {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

fn bytes_contains_any(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, "bytes.ContainsAny", &args[0])?;
    let ValueData::String(chars) = &args[1].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            "bytes.ContainsAny",
            "[]byte, string",
        )?);
    };
    let s = String::from_utf8_lossy(&data);
    Ok(Value::bool(s.contains(|c: char| chars.contains(c))))
}

fn bytes_contains_rune(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, "bytes.ContainsRune", &args[0])?;
    let ValueData::Int(rune) = args[1].data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            "bytes.ContainsRune",
            "[]byte, int",
        )?);
    };
    let Some(ch) = char::from_u32(rune as u32) else {
        return Ok(Value::bool(false));
    };
    let s = String::from_utf8_lossy(&data);
    Ok(Value::bool(s.contains(ch)))
}

pub(super) fn extract_bytes(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            builtin,
            "a []byte argument",
        )?);
    };
    slice
        .values_snapshot()
        .iter()
        .map(|v| match v.data {
            ValueData::Int(b) => Ok(b as u8),
            _ => {
                Err(invalid_bytes_argument(vm, program, builtin, "a []byte argument").unwrap_err())
            }
        })
        .collect()
}

pub(super) fn byte_slice_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<Vec<u8>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    extract_bytes(vm, program, builtin, &args[0])
}

pub(super) fn byte_slice_pair_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<u8>, Vec<u8>), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let a = extract_bytes(vm, program, builtin, &args[0])?;
    let b = extract_bytes(vm, program, builtin, &args[1])?;
    Ok((a, b))
}

fn byte_slice_and_int_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<u8>, u8), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let data = extract_bytes(vm, program, builtin, &args[0])?;
    let ValueData::Int(b) = args[1].data else {
        return Err(invalid_bytes_argument(vm, program, builtin, "[]byte, int")?);
    };
    Ok((data, b as u8))
}

fn byte_slice_of_slices_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<Vec<u8>>, VmError> {
    let ValueData::Slice(outer) = &value.data else {
        return Err(invalid_bytes_argument(
            vm,
            program,
            builtin,
            "a [][]byte argument",
        )?);
    };
    outer
        .values_snapshot()
        .iter()
        .map(|v| extract_bytes(vm, program, builtin, v))
        .collect()
}

pub(super) fn bytes_to_value(data: &[u8]) -> Value {
    Value::slice(data.iter().map(|&b| Value::int(b as i64)).collect())
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    find_subslice(haystack, needle).is_some()
}

pub(super) fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn rfind_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }
    haystack.windows(needle.len()).rposition(|w| w == needle)
}

pub(super) fn invalid_bytes_argument(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
) -> Result<VmError, VmError> {
    Ok(VmError::InvalidStringFunctionArgument {
        function: vm.current_function_name(program)?,
        builtin: builtin.into(),
        expected: expected.into(),
    })
}
