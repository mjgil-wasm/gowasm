use super::{
    StdlibFunction, HEX_DECODED_LEN, HEX_DECODE_STRING, HEX_ENCODED_LEN, HEX_ENCODE_TO_STRING,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const HEX_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: HEX_ENCODE_TO_STRING,
        symbol: "EncodeToString",
        returns_value: true,
        handler: hex_encode_to_string,
    },
    StdlibFunction {
        id: HEX_DECODE_STRING,
        symbol: "DecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: HEX_ENCODED_LEN,
        symbol: "EncodedLen",
        returns_value: true,
        handler: hex_encoded_len,
    },
    StdlibFunction {
        id: HEX_DECODED_LEN,
        symbol: "DecodedLen",
        returns_value: true,
        handler: hex_decoded_len,
    },
];

fn hex_encode_to_string(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let bytes = extract_byte_slice(vm, program, &args[0])?;
    let mut result = String::with_capacity(bytes.len() * 2);
    for b in &bytes {
        result.push(HEX_TABLE[(*b >> 4) as usize]);
        result.push(HEX_TABLE[(*b & 0x0f) as usize]);
    }
    Ok(Value::string(result))
}

pub(super) fn hex_decode_string(
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
    let ValueData::String(s) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "hex.DecodeString".into(),
            expected: "a string argument".into(),
        });
    };

    if s.len() % 2 != 0 {
        return Ok(vec![
            Value::slice(vec![]),
            Value::error("encoding/hex: odd length hex string"),
        ]);
    }

    let chars: Vec<u8> = s.bytes().collect();
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < chars.len() {
        let high = match from_hex_char(chars[i]) {
            Some(v) => v,
            None => {
                return Ok(vec![
                    bytes_to_value(&bytes),
                    Value::error(invalid_hex_byte_error(chars[i])),
                ]);
            }
        };
        let low = match from_hex_char(chars[i + 1]) {
            Some(v) => v,
            None => {
                return Ok(vec![
                    bytes_to_value(&bytes),
                    Value::error(invalid_hex_byte_error(chars[i + 1])),
                ]);
            }
        };
        bytes.push((high << 4) | low);
        i += 2;
    }
    Ok(vec![bytes_to_value(&bytes), Value::nil()])
}

fn hex_encoded_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Int(n) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "hex.EncodedLen".into(),
            expected: "an int argument".into(),
        });
    };
    Ok(Value::int(n * 2))
}

fn hex_decoded_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Int(x) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "hex.DecodedLen".into(),
            expected: "an int argument".into(),
        });
    };
    Ok(Value::int(x / 2))
}

fn extract_byte_slice(vm: &mut Vm, program: &Program, value: &Value) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "hex.EncodeToString".into(),
            expected: "a []byte argument".into(),
        });
    };
    slice
        .values_snapshot()
        .iter()
        .map(|v| match v.data {
            ValueData::Int(b) => Ok(b as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: "hex".into(),
                builtin: "hex.EncodeToString".into(),
                expected: "byte element".into(),
            }),
        })
        .collect()
}

fn bytes_to_value(data: &[u8]) -> Value {
    Value::slice(data.iter().map(|&b| Value::int(b as i64)).collect())
}

const HEX_TABLE: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

fn from_hex_char(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn invalid_hex_byte_error(byte: u8) -> String {
    format!(
        "encoding/hex: invalid byte: U+{:04X} '{}'",
        byte, byte as char
    )
}
