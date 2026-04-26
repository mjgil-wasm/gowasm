use crate::{Float64, Program, Value, ValueData, Vm, VmError};

use super::strconv_helpers_impl::{
    format_int_in_base, format_uint_in_base, parse_int, parse_uint, quote_rune, quote_rune_ascii,
    quote_string, quote_string_ascii, strconv_bool_arg, strconv_int_arg, strconv_int_pair_args,
    strconv_string_arg, strconv_string_int_arg_pair, strconv_string_int_int_args, unquote,
    unquote_char,
};
use super::{unsupported_multi_result_stdlib, StdlibFunction};

pub(super) const STRCONV_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: super::STRCONV_ITOA,
        symbol: "Itoa",
        returns_value: true,
        handler: strconv_itoa,
    },
    StdlibFunction {
        id: super::STRCONV_FORMAT_BOOL,
        symbol: "FormatBool",
        returns_value: true,
        handler: strconv_format_bool,
    },
    StdlibFunction {
        id: super::STRCONV_QUOTE,
        symbol: "Quote",
        returns_value: true,
        handler: strconv_quote,
    },
    StdlibFunction {
        id: super::STRCONV_CAN_BACKQUOTE,
        symbol: "CanBackquote",
        returns_value: true,
        handler: strconv_can_backquote,
    },
    StdlibFunction {
        id: super::STRCONV_FORMAT_INT,
        symbol: "FormatInt",
        returns_value: true,
        handler: strconv_format_int,
    },
    StdlibFunction {
        id: super::STRCONV_FORMAT_UINT,
        symbol: "FormatUint",
        returns_value: true,
        handler: strconv_format_uint,
    },
    StdlibFunction {
        id: super::STRCONV_QUOTE_TO_ASCII,
        symbol: "QuoteToASCII",
        returns_value: true,
        handler: strconv_quote_to_ascii,
    },
    StdlibFunction {
        id: super::STRCONV_QUOTE_RUNE_TO_ASCII,
        symbol: "QuoteRuneToASCII",
        returns_value: true,
        handler: strconv_quote_rune_to_ascii,
    },
    StdlibFunction {
        id: super::STRCONV_QUOTE_RUNE,
        symbol: "QuoteRune",
        returns_value: true,
        handler: strconv_quote_rune,
    },
    StdlibFunction {
        id: super::STRCONV_ATOI,
        symbol: "Atoi",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_PARSE_BOOL,
        symbol: "ParseBool",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_PARSE_INT,
        symbol: "ParseInt",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_UNQUOTE,
        symbol: "Unquote",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_UNQUOTE_CHAR,
        symbol: "UnquoteChar",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_FORMAT_FLOAT,
        symbol: "FormatFloat",
        returns_value: true,
        handler: strconv_format_float,
    },
    StdlibFunction {
        id: super::STRCONV_PARSE_FLOAT,
        symbol: "ParseFloat",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: super::STRCONV_PARSE_UINT,
        symbol: "ParseUint",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
];

pub(super) fn strconv_itoa(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_int_arg(vm, program, "strconv.Itoa", args)?;
    Ok(Value::string(value.to_string()))
}

pub(super) fn strconv_format_bool(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_bool_arg(vm, program, "strconv.FormatBool", args)?;
    Ok(Value::string(value.to_string()))
}

pub(super) fn strconv_quote(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.Quote", args)?;
    Ok(Value::string(quote_string(value)))
}

pub(super) fn strconv_quote_to_ascii(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.QuoteToASCII", args)?;
    Ok(Value::string(quote_string_ascii(value)))
}

pub(super) fn strconv_quote_rune_to_ascii(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_int_arg(vm, program, "strconv.QuoteRuneToASCII", args)?;
    Ok(Value::string(quote_rune_ascii(value)))
}

pub(super) fn strconv_quote_rune(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_int_arg(vm, program, "strconv.QuoteRune", args)?;
    Ok(Value::string(quote_rune(value)))
}

pub(super) fn strconv_can_backquote(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.CanBackquote", args)?;
    for ch in value.chars() {
        if ch == '\u{feff}' {
            return Ok(Value::bool(false));
        }
        if ch.len_utf8() == 1 && ((ch < ' ' && ch != '\t') || ch == '`' || ch == '\u{7f}') {
            return Ok(Value::bool(false));
        }
    }
    Ok(Value::bool(true))
}

pub(super) fn strconv_format_int(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (value, base) = strconv_int_pair_args(vm, program, "strconv.FormatInt", args)?;
    if !(2..=36).contains(&base) {
        return Err(VmError::InvalidFormatIntBase {
            function: vm.current_function_name(program)?,
            base,
        });
    }
    Ok(Value::string(format_int_in_base(value, base as u32)))
}

fn strconv_format_uint(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (value, base) = strconv_int_pair_args(vm, program, "strconv.FormatUint", args)?;
    if !(2..=36).contains(&base) {
        return Err(VmError::InvalidFormatIntBase {
            function: vm.current_function_name(program)?,
            base,
        });
    }
    Ok(Value::string(format_uint_in_base(
        value as u64,
        base as u32,
    )))
}

pub(super) fn strconv_atoi(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.Atoi", args)?;
    match parse_int(value, 10, 0) {
        Ok(number) => Ok(vec![Value::int(number), Value::nil()]),
        Err(detail) => Ok(vec![
            Value::int(0),
            Value::error(format!("strconv.Atoi: parsing {value:?}: {detail}")),
        ]),
    }
}

pub(super) fn strconv_parse_bool(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.ParseBool", args)?;
    let parsed = match value {
        "1" | "t" | "T" | "true" | "TRUE" | "True" => Some(true),
        "0" | "f" | "F" | "false" | "FALSE" | "False" => Some(false),
        _ => None,
    };

    match parsed {
        Some(value) => Ok(vec![Value::bool(value), Value::nil()]),
        None => Ok(vec![
            Value::bool(false),
            Value::error(format!(
                "strconv.ParseBool: parsing {value:?}: invalid syntax"
            )),
        ]),
    }
}

pub(super) fn strconv_parse_int(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (value, base, bit_size) =
        strconv_string_int_int_args(vm, program, "strconv.ParseInt", args)?;
    match parse_int(value, base, bit_size) {
        Ok(number) => Ok(vec![Value::int(number), Value::nil()]),
        Err(detail) => Ok(vec![
            Value::int(0),
            Value::error(format!("strconv.ParseInt: parsing {value:?}: {detail}")),
        ]),
    }
}

pub(super) fn strconv_unquote(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let value = strconv_string_arg(vm, program, "strconv.Unquote", args)?;
    match unquote(value) {
        Ok(unquoted) => Ok(vec![Value::string(unquoted), Value::nil()]),
        Err(detail) => Ok(vec![
            Value::string(""),
            Value::error(format!("strconv.Unquote: parsing {value:?}: {detail}")),
        ]),
    }
}

pub(super) fn strconv_unquote_char(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (value, quote) = strconv_string_int_arg_pair(vm, program, "strconv.UnquoteChar", args)?;
    match unquote_char(value, quote) {
        Ok((decoded, multibyte, tail)) => Ok(vec![
            Value::int(decoded),
            Value::bool(multibyte),
            Value::string(tail),
            Value::nil(),
        ]),
        Err(detail) => Ok(vec![
            Value::int(0),
            Value::bool(false),
            Value::string(""),
            Value::error(format!("strconv.UnquoteChar: parsing {value:?}: {detail}")),
        ]),
    }
}

fn strconv_format_float(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 4 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 4,
            actual: args.len(),
        });
    }
    let ValueData::Float(Float64(f)) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strconv.FormatFloat".into(),
            expected: "a float64 first argument".into(),
        });
    };
    let ValueData::Int(fmt_byte) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strconv.FormatFloat".into(),
            expected: "a byte (int) second argument".into(),
        });
    };
    let ValueData::Int(prec) = &args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strconv.FormatFloat".into(),
            expected: "an int third argument".into(),
        });
    };
    let ValueData::Int(bit_size) = &args[3].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "strconv.FormatFloat".into(),
            expected: "an int fourth argument".into(),
        });
    };
    let fmt_char = *fmt_byte as u8 as char;
    let prec = *prec as i32;
    if prec < 0 && matches!(fmt_char, 'g' | 'G') {
        let rendered = match *bit_size {
            32 => format_general_shortest_f32(*f as f32, fmt_char),
            _ => format_general_shortest(*f, fmt_char),
        };
        return Ok(Value::string(rendered));
    }
    let value = match *bit_size {
        32 => (*f as f32) as f64,
        _ => *f,
    };
    Ok(Value::string(format_float_value(value, fmt_char, prec)))
}

fn format_float_value(f: f64, fmt: char, prec: i32) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "+Inf".into()
        } else {
            "-Inf".into()
        };
    }
    match fmt {
        'f' => {
            if prec < 0 {
                format!("{f}")
            } else {
                let p = prec as usize;
                format!("{f:.p$}")
            }
        }
        'e' | 'E' => {
            if f == 0.0 {
                if prec < 0 {
                    return format!("0{fmt}+00");
                }
                let p = prec as usize;
                return format!("0.{:0>p$}{fmt}+00", "");
            }
            let abs = f.abs();
            let exp = abs.log10().floor() as i32;
            let mantissa = abs / 10.0_f64.powi(exp);
            let sign = if f < 0.0 { "-" } else { "" };
            if prec < 0 {
                format!("{sign}{mantissa}{fmt}{exp:+03}")
            } else {
                let p = prec as usize;
                format!("{sign}{mantissa:.p$}{fmt}{exp:+03}")
            }
        }
        'g' | 'G' => {
            let exp_char = if fmt == 'g' { 'e' } else { 'E' };
            if f == 0.0 {
                return "0".into();
            }
            let abs = f.abs();
            let exp = abs.log10().floor() as i32;
            let effective_prec = if prec < 0 { 0 } else { prec };
            if exp < -4 || exp >= effective_prec.max(1) {
                let e_prec = if prec < 0 { -1 } else { (prec - 1).max(0) };
                format_float_value(f, exp_char, e_prec)
            } else {
                let f_prec = if prec < 0 {
                    -1
                } else {
                    (prec - 1 - exp).max(0)
                };
                format_float_value(f, 'f', f_prec)
            }
        }
        'b' => {
            let bits = f.to_bits();
            let sign = if f < 0.0 { "-" } else { "" };
            let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
            let exp = ((bits >> 52) & 0x7FF) as i64 - 1023 - 52;
            format!("{sign}{mantissa}p{exp:+}")
        }
        _ => format!("%{fmt}(ERROR)"),
    }
}

pub(super) fn strconv_parse_float(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (value, bit_size) = strconv_string_int_arg_pair(vm, program, "strconv.ParseFloat", args)?;
    let (parsed, error) = parse_float_for_bit_size(value, bit_size);
    Ok(vec![
        Value::float(parsed),
        error
            .map(|detail| Value::error(format!("strconv.ParseFloat: parsing {value:?}: {detail}")))
            .unwrap_or_else(Value::nil),
    ])
}

pub(super) fn strconv_parse_uint(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (value, base, bit_size) =
        strconv_string_int_int_args(vm, program, "strconv.ParseUint", args)?;
    match parse_uint(value, base, bit_size) {
        Ok(number) => Ok(vec![Value::int(number as i64), Value::nil()]),
        Err(detail) => Ok(vec![
            Value::int(0),
            Value::error(format!("strconv.ParseUint: parsing {value:?}: {detail}")),
        ]),
    }
}

fn parse_float_for_bit_size(value: &str, bit_size: i64) -> (f64, Option<&'static str>) {
    match value.parse::<f64>() {
        Ok(parsed) => {
            if parsed.is_infinite() && !is_parse_float_infinity_literal(value) {
                return (
                    if parsed.is_sign_negative() {
                        f64::NEG_INFINITY
                    } else {
                        f64::INFINITY
                    },
                    Some("value out of range"),
                );
            }
            if bit_size == 32 {
                let narrowed = parsed as f32;
                if narrowed.is_infinite() && parsed.is_finite() {
                    let overflow = if parsed.is_sign_negative() {
                        f64::NEG_INFINITY
                    } else {
                        f64::INFINITY
                    };
                    return (overflow, Some("value out of range"));
                }
                return (narrowed as f64, None);
            }
            (parsed, None)
        }
        Err(_) => (0.0, Some("invalid syntax")),
    }
}

fn is_parse_float_infinity_literal(value: &str) -> bool {
    let trimmed = value.trim();
    let unsigned = trimmed
        .strip_prefix('+')
        .or_else(|| trimmed.strip_prefix('-'))
        .unwrap_or(trimmed);
    unsigned.eq_ignore_ascii_case("inf")
}

fn format_general_shortest(value: f64, fmt: char) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Inf".into()
        } else {
            "+Inf".into()
        };
    }

    let rendered = value.to_string();
    if fmt == 'G' {
        rendered.replace('e', "E")
    } else {
        rendered
    }
}

fn format_general_shortest_f32(value: f32, fmt: char) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Inf".into()
        } else {
            "+Inf".into()
        };
    }

    let rendered = value.to_string();
    if fmt == 'G' {
        rendered.replace('e', "E")
    } else {
        rendered
    }
}
