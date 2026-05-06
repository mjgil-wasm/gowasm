use super::{
    StdlibFunction, StdlibMethod, StdlibValue, StdlibValueInit, BASE64_ENCODING_DECODE_STRING,
    BASE64_ENCODING_ENCODE_TO_STRING, BASE64_RAW_STD_ENCODING_DECODE_STRING,
    BASE64_RAW_STD_ENCODING_ENCODE_TO_STRING, BASE64_RAW_URL_ENCODING_DECODE_STRING,
    BASE64_RAW_URL_ENCODING_ENCODE_TO_STRING, BASE64_STD_ENCODING_DECODE_STRING,
    BASE64_STD_ENCODING_ENCODE_TO_STRING, BASE64_URL_ENCODING_DECODE_STRING,
    BASE64_URL_ENCODING_ENCODE_TO_STRING,
};
use crate::{PointerTarget, Program, Value, ValueData, Vm, VmError, TYPE_BASE64_ENCODING};

const BASE64_RECEIVER_TYPE: &str = "*base64.Encoding";
const BASE64_ENCODING_KIND_FIELD: &str = "__encodingKind";
const BASE64_STD_KIND: i64 = 1;
const BASE64_URL_KIND: i64 = 2;
const BASE64_RAW_STD_KIND: i64 = 3;
const BASE64_RAW_URL_KIND: i64 = 4;
const BASE64_RECEIVER_EXPECTED: &str = "a valid *base64.Encoding receiver";

pub(super) const BASE64_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "StdEncoding",
        typ: "*base64.Encoding",
        value: StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: BASE64_ENCODING_KIND_FIELD,
            value: BASE64_STD_KIND,
        },
    },
    StdlibValue {
        symbol: "URLEncoding",
        typ: "*base64.Encoding",
        value: StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: BASE64_ENCODING_KIND_FIELD,
            value: BASE64_URL_KIND,
        },
    },
    StdlibValue {
        symbol: "RawStdEncoding",
        typ: "*base64.Encoding",
        value: StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: BASE64_ENCODING_KIND_FIELD,
            value: BASE64_RAW_STD_KIND,
        },
    },
    StdlibValue {
        symbol: "RawURLEncoding",
        typ: "*base64.Encoding",
        value: StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: BASE64_ENCODING_KIND_FIELD,
            value: BASE64_RAW_URL_KIND,
        },
    },
];

pub(super) const BASE64_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: BASE64_STD_ENCODING_ENCODE_TO_STRING,
        symbol: "StdEncodingEncodeToString",
        returns_value: true,
        handler: base64_std_encoding_encode_to_string,
    },
    StdlibFunction {
        id: BASE64_STD_ENCODING_DECODE_STRING,
        symbol: "StdEncodingDecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: BASE64_URL_ENCODING_ENCODE_TO_STRING,
        symbol: "URLEncodingEncodeToString",
        returns_value: true,
        handler: base64_url_encoding_encode_to_string,
    },
    StdlibFunction {
        id: BASE64_URL_ENCODING_DECODE_STRING,
        symbol: "URLEncodingDecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: BASE64_RAW_STD_ENCODING_ENCODE_TO_STRING,
        symbol: "RawStdEncodingEncodeToString",
        returns_value: true,
        handler: base64_raw_std_encoding_encode_to_string,
    },
    StdlibFunction {
        id: BASE64_RAW_STD_ENCODING_DECODE_STRING,
        symbol: "RawStdEncodingDecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: BASE64_RAW_URL_ENCODING_ENCODE_TO_STRING,
        symbol: "RawURLEncodingEncodeToString",
        returns_value: true,
        handler: base64_raw_url_encoding_encode_to_string,
    },
    StdlibFunction {
        id: BASE64_RAW_URL_ENCODING_DECODE_STRING,
        symbol: "RawURLEncodingDecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

pub(super) const BASE64_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: BASE64_RECEIVER_TYPE,
        method: "EncodeToString",
        function: BASE64_ENCODING_ENCODE_TO_STRING,
    },
    StdlibMethod {
        receiver_type: BASE64_RECEIVER_TYPE,
        method: "DecodeString",
        function: BASE64_ENCODING_DECODE_STRING,
    },
];

pub(super) const BASE64_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: BASE64_ENCODING_ENCODE_TO_STRING,
        symbol: "EncodeToString",
        returns_value: true,
        handler: base64_encoding_encode_to_string,
    },
    StdlibFunction {
        id: BASE64_ENCODING_DECODE_STRING,
        symbol: "DecodeString",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

#[derive(Debug, Clone, Copy)]
struct Base64Encoding {
    alphabet: [u8; 64],
    padded: bool,
    encode_symbol: &'static str,
    decode_symbol: &'static str,
}

const STD_ALPHABET: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_ALPHABET: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

const STD_ENCODING: Base64Encoding = Base64Encoding {
    alphabet: STD_ALPHABET,
    padded: true,
    encode_symbol: "base64.StdEncodingEncodeToString",
    decode_symbol: "base64.StdEncodingDecodeString",
};
const URL_ENCODING: Base64Encoding = Base64Encoding {
    alphabet: URL_ALPHABET,
    padded: true,
    encode_symbol: "base64.URLEncodingEncodeToString",
    decode_symbol: "base64.URLEncodingDecodeString",
};
const RAW_STD_ENCODING: Base64Encoding = Base64Encoding {
    alphabet: STD_ALPHABET,
    padded: false,
    encode_symbol: "base64.RawStdEncodingEncodeToString",
    decode_symbol: "base64.RawStdEncodingDecodeString",
};
const RAW_URL_ENCODING: Base64Encoding = Base64Encoding {
    alphabet: URL_ALPHABET,
    padded: false,
    encode_symbol: "base64.RawURLEncodingEncodeToString",
    decode_symbol: "base64.RawURLEncodingDecodeString",
};

fn base64_encoding_encode_to_string(
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
    let encoding =
        *base64_encoding_from_receiver(vm, program, "(*base64.Encoding).EncodeToString", &args[0])?;
    base64_encode_to_string(vm, program, &args[1..], &encoding)
}

fn base64_std_encoding_encode_to_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    base64_encode_to_string(vm, program, args, &STD_ENCODING)
}

fn base64_url_encoding_encode_to_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    base64_encode_to_string(vm, program, args, &URL_ENCODING)
}

fn base64_raw_std_encoding_encode_to_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    base64_encode_to_string(vm, program, args, &RAW_STD_ENCODING)
}

fn base64_raw_url_encoding_encode_to_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    base64_encode_to_string(vm, program, args, &RAW_URL_ENCODING)
}

pub(super) fn base64_std_encoding_decode_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    base64_decode_string(vm, program, args, &STD_ENCODING)
}

pub(super) fn base64_url_encoding_decode_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    base64_decode_string(vm, program, args, &URL_ENCODING)
}

pub(super) fn base64_raw_std_encoding_decode_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    base64_decode_string(vm, program, args, &RAW_STD_ENCODING)
}

pub(super) fn base64_raw_url_encoding_decode_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    base64_decode_string(vm, program, args, &RAW_URL_ENCODING)
}

pub(super) fn base64_encoding_decode_string(
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
    let encoding =
        *base64_encoding_from_receiver(vm, program, "(*base64.Encoding).DecodeString", &args[0])?;
    base64_decode_string(vm, program, &args[1..], &encoding)
}

fn base64_encode_to_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    encoding: &Base64Encoding,
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let bytes = extract_byte_slice(vm, program, &args[0], encoding.encode_symbol)?;
    Ok(Value::string(encode_bytes(&bytes, encoding)))
}

fn base64_decode_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    encoding: &Base64Encoding,
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(input) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: encoding.decode_symbol.into(),
            expected: "a string argument".into(),
        });
    };
    let (decoded, error) = decode_bytes(input.as_bytes(), encoding);
    Ok(vec![
        bytes_to_value(&decoded),
        error.map_or_else(Value::nil, Value::error),
    ])
}

fn encode_bytes(data: &[u8], encoding: &Base64Encoding) -> String {
    let mut result = String::with_capacity(encoded_len(data.len(), encoding.padded));
    let mut chunks = data.chunks_exact(3);
    for chunk in &mut chunks {
        let block = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | u32::from(chunk[2]);
        result.push(encoding.alphabet[((block >> 18) & 0x3f) as usize] as char);
        result.push(encoding.alphabet[((block >> 12) & 0x3f) as usize] as char);
        result.push(encoding.alphabet[((block >> 6) & 0x3f) as usize] as char);
        result.push(encoding.alphabet[(block & 0x3f) as usize] as char);
    }

    match chunks.remainder() {
        [] => {}
        [b0] => {
            let block = (u32::from(*b0)) << 16;
            result.push(encoding.alphabet[((block >> 18) & 0x3f) as usize] as char);
            result.push(encoding.alphabet[((block >> 12) & 0x3f) as usize] as char);
            if encoding.padded {
                result.push('=');
                result.push('=');
            }
        }
        [b0, b1] => {
            let block = ((u32::from(*b0)) << 16) | ((u32::from(*b1)) << 8);
            result.push(encoding.alphabet[((block >> 18) & 0x3f) as usize] as char);
            result.push(encoding.alphabet[((block >> 12) & 0x3f) as usize] as char);
            result.push(encoding.alphabet[((block >> 6) & 0x3f) as usize] as char);
            if encoding.padded {
                result.push('=');
            }
        }
        _ => unreachable!(),
    }

    result
}

fn decode_bytes(input: &[u8], encoding: &Base64Encoding) -> (Vec<u8>, Option<String>) {
    let mut output = Vec::with_capacity(decoded_capacity(input.len()));
    let mut index = 0;

    if encoding.padded {
        while index + 4 <= input.len() {
            let chunk = &input[index..index + 4];
            let c0 = match decode_char(chunk[0], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index),
            };
            let c1 = match decode_char(chunk[1], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 1),
            };

            if chunk[2] == b'=' {
                if chunk[3] != b'=' || index + 4 != input.len() {
                    return invalid_base64(&output, index + 2);
                }
                output.push((c0 << 2) | (c1 >> 4));
                return (output, None);
            }

            let c2 = match decode_char(chunk[2], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 2),
            };

            if chunk[3] == b'=' {
                if index + 4 != input.len() {
                    return invalid_base64(&output, index + 3);
                }
                output.push((c0 << 2) | (c1 >> 4));
                output.push(((c1 & 0x0f) << 4) | (c2 >> 2));
                return (output, None);
            }

            let c3 = match decode_char(chunk[3], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 3),
            };

            output.push((c0 << 2) | (c1 >> 4));
            output.push(((c1 & 0x0f) << 4) | (c2 >> 2));
            output.push(((c2 & 0x03) << 6) | c3);
            index += 4;
        }
    } else {
        while index + 4 <= input.len() {
            let chunk = &input[index..index + 4];
            let c0 = match decode_char(chunk[0], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index),
            };
            let c1 = match decode_char(chunk[1], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 1),
            };
            let c2 = match decode_char(chunk[2], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 2),
            };
            let c3 = match decode_char(chunk[3], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 3),
            };

            output.push((c0 << 2) | (c1 >> 4));
            output.push(((c1 & 0x0f) << 4) | (c2 >> 2));
            output.push(((c2 & 0x03) << 6) | c3);
            index += 4;
        }
    }

    let remainder = &input[index..];
    if encoding.padded {
        if remainder.is_empty() {
            return (output, None);
        }
        return invalid_base64(&output, index);
    }

    if let Some(offset) = remainder.iter().position(|&byte| byte == b'=') {
        return invalid_base64(&output, index + offset);
    }

    let remainder_len = remainder.len();
    if remainder_len == 1 {
        return invalid_base64(&output, index);
    }
    if remainder_len >= 2 {
        let c0 = match decode_char(remainder[0], &encoding.alphabet) {
            Some(value) => value,
            None => return invalid_base64(&output, index),
        };
        let c1 = match decode_char(remainder[1], &encoding.alphabet) {
            Some(value) => value,
            None => return invalid_base64(&output, index + 1),
        };
        output.push((c0 << 2) | (c1 >> 4));
        if remainder_len == 3 {
            let c2 = match decode_char(remainder[2], &encoding.alphabet) {
                Some(value) => value,
                None => return invalid_base64(&output, index + 2),
            };
            output.push(((c1 & 0x0f) << 4) | (c2 >> 2));
        }
    }

    (output, None)
}

fn decode_char(byte: u8, alphabet: &[u8; 64]) -> Option<u8> {
    alphabet
        .iter()
        .position(|candidate| *candidate == byte)
        .map(|index| index as u8)
}

fn encoded_len(input_len: usize, padded: bool) -> usize {
    if padded {
        input_len.div_ceil(3) * 4
    } else {
        let full = (input_len / 3) * 4;
        match input_len % 3 {
            0 => full,
            1 => full + 2,
            2 => full + 3,
            _ => unreachable!(),
        }
    }
}

fn decoded_capacity(input_len: usize) -> usize {
    (input_len / 4) * 3 + 2
}

fn invalid_base64(decoded: &[u8], index: usize) -> (Vec<u8>, Option<String>) {
    (
        decoded.to_vec(),
        Some(format!("illegal base64 data at input byte {index}")),
    )
}

fn base64_encoding_from_receiver<'a>(
    vm: &'a Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<&'a Base64Encoding, VmError> {
    let ValueData::Pointer(pointer) = &value.data else {
        return Err(invalid_base64_receiver(vm, program, builtin));
    };
    if matches!(&pointer.target, PointerTarget::Nil) {
        return Err(invalid_base64_receiver(vm, program, builtin));
    }

    let encoding = vm.deref_pointer(program, value)?;
    if encoding.typ != TYPE_BASE64_ENCODING {
        return Err(invalid_base64_receiver(vm, program, builtin));
    }

    let ValueData::Struct(fields) = &encoding.data else {
        return Err(invalid_base64_receiver(vm, program, builtin));
    };
    let Some((_, kind_value)) = fields
        .iter()
        .find(|(name, _)| name == BASE64_ENCODING_KIND_FIELD)
    else {
        return Err(invalid_base64_receiver(vm, program, builtin));
    };
    let ValueData::Int(kind) = kind_value.data else {
        return Err(invalid_base64_receiver(vm, program, builtin));
    };
    base64_encoding_for_kind(kind).ok_or_else(|| invalid_base64_receiver(vm, program, builtin))
}

fn base64_encoding_for_kind(kind: i64) -> Option<&'static Base64Encoding> {
    match kind {
        BASE64_STD_KIND => Some(&STD_ENCODING),
        BASE64_URL_KIND => Some(&URL_ENCODING),
        BASE64_RAW_STD_KIND => Some(&RAW_STD_ENCODING),
        BASE64_RAW_URL_KIND => Some(&RAW_URL_ENCODING),
        _ => None,
    }
}

fn invalid_base64_receiver(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| builtin.to_string()),
        builtin: builtin.into(),
        expected: BASE64_RECEIVER_EXPECTED.into(),
    }
}

fn extract_byte_slice(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    builtin: &'static str,
) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a []byte argument".into(),
        });
    };
    slice
        .values_snapshot()
        .iter()
        .map(|value| match value.data {
            ValueData::Int(byte) => Ok(byte as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: builtin.into(),
                builtin: builtin.into(),
                expected: "byte element".into(),
            }),
        })
        .collect()
}

fn bytes_to_value(data: &[u8]) -> Value {
    Value::slice(data.iter().map(|&byte| Value::int(byte as i64)).collect())
}
