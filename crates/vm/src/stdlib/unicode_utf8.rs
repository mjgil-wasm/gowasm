use super::{
    StdlibFunction, UTF8_DECODE_RUNE_IN_STRING, UTF8_FULL_RUNE_IN_STRING, UTF8_RUNE_COUNT,
    UTF8_RUNE_COUNT_IN_STRING, UTF8_RUNE_LEN, UTF8_VALID, UTF8_VALID_RUNE, UTF8_VALID_STRING,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const UTF8_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: UTF8_RUNE_COUNT_IN_STRING,
        symbol: "RuneCountInString",
        returns_value: true,
        handler: utf8_rune_count_in_string,
    },
    StdlibFunction {
        id: UTF8_VALID_STRING,
        symbol: "ValidString",
        returns_value: true,
        handler: utf8_valid_string,
    },
    StdlibFunction {
        id: UTF8_RUNE_LEN,
        symbol: "RuneLen",
        returns_value: true,
        handler: utf8_rune_len,
    },
    StdlibFunction {
        id: UTF8_VALID_RUNE,
        symbol: "ValidRune",
        returns_value: true,
        handler: utf8_valid_rune,
    },
    StdlibFunction {
        id: UTF8_RUNE_COUNT,
        symbol: "RuneCount",
        returns_value: true,
        handler: utf8_rune_count,
    },
    StdlibFunction {
        id: UTF8_VALID,
        symbol: "Valid",
        returns_value: true,
        handler: utf8_valid,
    },
    StdlibFunction {
        id: UTF8_FULL_RUNE_IN_STRING,
        symbol: "FullRuneInString",
        returns_value: true,
        handler: utf8_full_rune_in_string,
    },
    StdlibFunction {
        id: UTF8_DECODE_RUNE_IN_STRING,
        symbol: "DecodeRuneInString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

pub(super) const UTF8_CONSTANTS: &[super::StdlibConstant] = &[
    super::StdlibConstant {
        symbol: "RuneError",
        typ: "int",
        value: super::StdlibConstantValue::Int(0xFFFD),
    },
    super::StdlibConstant {
        symbol: "MaxRune",
        typ: "int",
        value: super::StdlibConstantValue::Int(0x0010_FFFF),
    },
    super::StdlibConstant {
        symbol: "UTFMax",
        typ: "int",
        value: super::StdlibConstantValue::Int(4),
    },
    super::StdlibConstant {
        symbol: "RuneSelf",
        typ: "int",
        value: super::StdlibConstantValue::Int(0x80),
    },
];

fn string_arg<'a>(
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
    match &args[0].data {
        ValueData::String(s) => Ok(s.as_str()),
        _ => Err(invalid_utf8_argument(
            vm,
            program,
            builtin,
            "a string argument",
        )?),
    }
}

fn int_arg(vm: &mut Vm, program: &Program, builtin: &str, args: &[Value]) -> Result<i64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Int(v) => Ok(*v),
        _ => Err(invalid_utf8_argument(
            vm,
            program,
            builtin,
            "an int argument",
        )?),
    }
}

fn byte_slice_arg(
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
    match &args[0].data {
        ValueData::Slice(slice) => {
            let values = slice.values_snapshot();
            let mut bytes = Vec::with_capacity(values.len());
            for v in &values {
                match &v.data {
                    ValueData::Int(n) => bytes.push(*n as u8),
                    _ => {
                        return Err(invalid_utf8_argument(
                            vm,
                            program,
                            builtin,
                            "a []byte argument",
                        )?)
                    }
                }
            }
            Ok(bytes)
        }
        _ => Err(invalid_utf8_argument(
            vm,
            program,
            builtin,
            "a []byte argument",
        )?),
    }
}

fn invalid_utf8_argument(
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

fn utf8_rune_count_in_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let s = string_arg(vm, program, "utf8.RuneCountInString", args)?;
    Ok(Value::int(s.chars().count() as i64))
}

fn utf8_valid_string(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let s = string_arg(vm, program, "utf8.ValidString", args)?;
    let _ = s;
    Ok(Value::bool(true))
}

fn utf8_rune_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let r = int_arg(vm, program, "utf8.RuneLen", args)?;
    let r = r as u32;
    let len = match char::from_u32(r) {
        None => -1i64,
        Some(c) => c.len_utf8() as i64,
    };
    Ok(Value::int(len))
}

fn utf8_valid_rune(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let r = int_arg(vm, program, "utf8.ValidRune", args)?;
    Ok(Value::bool(char::from_u32(r as u32).is_some()))
}

fn utf8_rune_count(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let bytes = byte_slice_arg(vm, program, "utf8.RuneCount", args)?;
    Ok(Value::int(count_utf8_runes(&bytes) as i64))
}

fn utf8_valid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let bytes = byte_slice_arg(vm, program, "utf8.Valid", args)?;
    Ok(Value::bool(std::str::from_utf8(&bytes).is_ok()))
}

fn utf8_full_rune_in_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let s = string_arg(vm, program, "utf8.FullRuneInString", args)?;
    Ok(Value::bool(!s.is_empty()))
}

pub(super) fn utf8_decode_rune_in_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let s = string_arg(vm, program, "utf8.DecodeRuneInString", args)?;
    if s.is_empty() {
        return Ok(vec![Value::int(0xFFFD), Value::int(0)]);
    }
    let c = s.chars().next().unwrap();
    Ok(vec![Value::int(c as i64), Value::int(c.len_utf8() as i64)])
}

fn count_utf8_runes(bytes: &[u8]) -> usize {
    let mut count = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        let width = valid_utf8_width(&bytes[index..]).unwrap_or(1);
        index += width;
        count += 1;
    }

    count
}

fn valid_utf8_width(bytes: &[u8]) -> Option<usize> {
    let first = *bytes.first()?;
    match first {
        0x00..=0x7f => Some(1),
        0xc2..=0xdf => continuation(bytes, 1).then_some(2),
        0xe0 => continuation_in_range(bytes, 1, 0xa0, 0xbf)
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xe1..=0xec | 0xee..=0xef => continuation(bytes, 1)
            .then_some(())
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xed => continuation_in_range(bytes, 1, 0x80, 0x9f)
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xf0 => continuation_in_range(bytes, 1, 0x90, 0xbf)
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        0xf1..=0xf3 => continuation(bytes, 1)
            .then_some(())
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        0xf4 => continuation_in_range(bytes, 1, 0x80, 0x8f)
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        _ => None,
    }
}

fn continuation(bytes: &[u8], index: usize) -> bool {
    matches!(bytes.get(index), Some(value) if (0x80..=0xbf).contains(value))
}

fn continuation_in_range(bytes: &[u8], index: usize, low: u8, high: u8) -> Option<()> {
    matches!(bytes.get(index), Some(value) if (low..=high).contains(value)).then_some(())
}
