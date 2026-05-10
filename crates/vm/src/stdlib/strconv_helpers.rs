use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) fn strconv_int_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<i64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::Int(value) = args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "an int argument".into(),
        });
    };
    Ok(value)
}

pub(super) fn strconv_int_pair_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(i64, i64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::Int(value) = args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "int arguments".into(),
        });
    };
    let ValueData::Int(base) = args[1].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "int arguments".into(),
        });
    };
    Ok((value, base))
}

pub(super) fn strconv_string_int_arg_pair<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, i64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::String(value) = &args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument followed by an int argument".into(),
        });
    };
    let ValueData::Int(number) = args[1].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument followed by an int argument".into(),
        });
    };
    Ok((value, number))
}

pub(super) fn strconv_string_int_int_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, i64, i64), VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }

    let ValueData::String(value) = &args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument followed by two int arguments".into(),
        });
    };
    let ValueData::Int(base) = args[1].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument followed by two int arguments".into(),
        });
    };
    let ValueData::Int(bit_size) = args[2].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument followed by two int arguments".into(),
        });
    };
    Ok((value, base, bit_size))
}

pub(super) fn strconv_bool_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<bool, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::Bool(value) = args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a bool argument".into(),
        });
    };
    Ok(value)
}

pub(super) fn strconv_string_arg<'a>(
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

    let ValueData::String(value) = &args[0].data else {
        return Err(VmError::InvalidStrconvFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };
    Ok(value)
}

pub(super) fn format_int_in_base(value: i64, base: u32) -> String {
    let negative = value < 0;
    let mut magnitude = if negative {
        -(value as i128)
    } else {
        value as i128
    };

    if magnitude == 0 {
        return "0".into();
    }

    let mut digits = Vec::new();
    while magnitude > 0 {
        let digit = (magnitude % base as i128) as u8;
        let ch = if digit < 10 {
            (b'0' + digit) as char
        } else {
            (b'a' + (digit - 10)) as char
        };
        digits.push(ch);
        magnitude /= base as i128;
    }

    if negative {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

pub(super) fn format_uint_in_base(mut value: u64, base: u32) -> String {
    if value == 0 {
        return "0".into();
    }

    let mut digits = Vec::new();
    while value > 0 {
        let digit = (value % base as u64) as u8;
        let ch = if digit < 10 {
            (b'0' + digit) as char
        } else {
            (b'a' + (digit - 10)) as char
        };
        digits.push(ch);
        value /= base as u64;
    }
    digits.iter().rev().collect()
}

pub(super) fn parse_int(value: &str, base: i64, bit_size: i64) -> Result<i64, String> {
    if value.is_empty() {
        return Err("invalid syntax".into());
    }
    if base != 0 && !(2..=36).contains(&base) {
        return Err(format!("invalid base {base}"));
    }
    if !(bit_size == 0 || (1..=64).contains(&bit_size)) {
        return Err(format!("invalid bit size {bit_size}"));
    }

    let (negative, unsigned) = match value.as_bytes()[0] {
        b'+' => (false, &value[1..]),
        b'-' => (true, &value[1..]),
        _ => (false, value),
    };
    if unsigned.is_empty() {
        return Err("invalid syntax".into());
    }

    let (unsigned, base) = detect_parse_int_base(unsigned, base);
    if unsigned.is_empty() {
        return Err("invalid syntax".into());
    }

    let magnitude =
        u128::from_str_radix(unsigned, base as u32).map_err(|error| match error.kind() {
            std::num::IntErrorKind::InvalidDigit => String::from("invalid syntax"),
            _ => String::from("value out of range"),
        })?;

    let bits = if bit_size == 0 { 64 } else { bit_size as u32 };
    let (min, max) = signed_bounds(bits);
    let signed = if negative {
        if magnitude > (i128::MAX as u128) + 1 {
            return Err("value out of range".into());
        }
        -(magnitude as i128)
    } else {
        if magnitude > i128::MAX as u128 {
            return Err("value out of range".into());
        }
        magnitude as i128
    };

    if signed < min || signed > max {
        return Err("value out of range".into());
    }
    Ok(signed as i64)
}

fn detect_parse_int_base(value: &str, base: i64) -> (&str, i64) {
    if base != 0 {
        return (value, base);
    }
    if let Some(rest) = value
        .strip_prefix("0b")
        .or_else(|| value.strip_prefix("0B"))
    {
        return (rest, 2);
    }
    if let Some(rest) = value
        .strip_prefix("0o")
        .or_else(|| value.strip_prefix("0O"))
    {
        return (rest, 8);
    }
    if let Some(rest) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return (rest, 16);
    }
    if value.len() > 1 && value.starts_with('0') {
        return (&value[1..], 8);
    }
    (value, 10)
}

pub(super) fn parse_uint(value: &str, base: i64, bit_size: i64) -> Result<u64, String> {
    if value.is_empty() {
        return Err("invalid syntax".into());
    }
    if base != 0 && !(2..=36).contains(&base) {
        return Err(format!("invalid base {base}"));
    }
    if !(bit_size == 0 || (1..=64).contains(&bit_size)) {
        return Err(format!("invalid bit size {bit_size}"));
    }

    let unsigned = match value.as_bytes()[0] {
        b'+' => &value[1..],
        b'-' => return Err("invalid syntax".into()),
        _ => value,
    };
    if unsigned.is_empty() {
        return Err("invalid syntax".into());
    }

    let (unsigned, base) = detect_parse_int_base(unsigned, base);
    if unsigned.is_empty() {
        return Err("invalid syntax".into());
    }

    let magnitude =
        u128::from_str_radix(unsigned, base as u32).map_err(|error| match error.kind() {
            std::num::IntErrorKind::InvalidDigit => String::from("invalid syntax"),
            _ => String::from("value out of range"),
        })?;

    let bits = if bit_size == 0 { 64 } else { bit_size as u32 };
    let max = if bits == 64 {
        u64::MAX as u128
    } else {
        (1u128 << bits) - 1
    };
    if magnitude > max {
        return Err("value out of range".into());
    }
    Ok(magnitude as u64)
}

fn signed_bounds(bits: u32) -> (i128, i128) {
    let shift = bits - 1;
    let min = -(1i128 << shift);
    let max = (1i128 << shift) - 1;
    (min, max)
}

pub(super) fn unquote(value: &str) -> Result<String, &'static str> {
    if value.len() < 2 {
        return Err("invalid syntax");
    }

    if value.starts_with('"') && value.ends_with('"') {
        return unquote_interpreted(&value[1..value.len() - 1], '"');
    }
    if value.starts_with('`') && value.ends_with('`') {
        return Ok(value[1..value.len() - 1].to_string());
    }
    if value.starts_with('\'') && value.ends_with('\'') {
        let unquoted = unquote_interpreted(&value[1..value.len() - 1], '\'')?;
        if unquoted.chars().count() != 1 {
            return Err("invalid syntax");
        }
        return Ok(unquoted);
    }
    Err("invalid syntax")
}

pub(super) fn unquote_char(value: &str, quote: i64) -> Result<(i64, bool, String), &'static str> {
    let quote = decode_unquote_char_quote(quote)?;
    let (decoded, tail) = unquote_char_prefix(value, quote)?;
    Ok((decoded.value as i64, decoded.multibyte, tail.to_string()))
}

fn unquote_interpreted(mut value: &str, quote: char) -> Result<String, &'static str> {
    let mut out = String::with_capacity(value.len());
    while !value.is_empty() {
        let (decoded, tail) = unquote_char_prefix(value, Some(quote))?;
        out.push(decoded.value);
        value = tail;
    }
    Ok(out)
}

struct DecodedChar {
    value: char,
    multibyte: bool,
}

fn unquote_char_prefix(
    value: &str,
    quote: Option<char>,
) -> Result<(DecodedChar, &str), &'static str> {
    let mut chars = value.chars();
    let first = chars.next().ok_or("invalid syntax")?;

    if first != '\\' {
        if quote.is_some_and(|quote| quote == first) {
            return Err("invalid syntax");
        }
        return Ok((
            DecodedChar {
                value: first,
                multibyte: first.len_utf8() > 1,
            },
            chars.as_str(),
        ));
    }

    let escaped = chars.next().ok_or("invalid syntax")?;
    let (decoded, tail) = match escaped {
        'a' => (
            DecodedChar {
                value: '\u{07}',
                multibyte: false,
            },
            chars.as_str(),
        ),
        'b' => (
            DecodedChar {
                value: '\u{08}',
                multibyte: false,
            },
            chars.as_str(),
        ),
        'f' => (
            DecodedChar {
                value: '\u{0c}',
                multibyte: false,
            },
            chars.as_str(),
        ),
        'n' => (
            DecodedChar {
                value: '\n',
                multibyte: false,
            },
            chars.as_str(),
        ),
        'r' => (
            DecodedChar {
                value: '\r',
                multibyte: false,
            },
            chars.as_str(),
        ),
        't' => (
            DecodedChar {
                value: '\t',
                multibyte: false,
            },
            chars.as_str(),
        ),
        'v' => (
            DecodedChar {
                value: '\u{0b}',
                multibyte: false,
            },
            chars.as_str(),
        ),
        '\\' => (
            DecodedChar {
                value: '\\',
                multibyte: false,
            },
            chars.as_str(),
        ),
        '\'' if quote == Some('\'') => (
            DecodedChar {
                value: '\'',
                multibyte: false,
            },
            chars.as_str(),
        ),
        '"' if quote == Some('"') => (
            DecodedChar {
                value: '"',
                multibyte: false,
            },
            chars.as_str(),
        ),
        '0'..='7' => decode_octal_escape(escaped, chars.as_str())?,
        'x' => decode_byte_escape(chars.as_str(), 2, 16)?,
        'u' => decode_unicode_escape(chars.as_str(), 4)?,
        'U' => decode_unicode_escape(chars.as_str(), 8)?,
        _ => return Err("invalid syntax"),
    };
    Ok((decoded, tail))
}

fn decode_byte_escape(
    value: &str,
    digits: usize,
    radix: u32,
) -> Result<(DecodedChar, &str), &'static str> {
    let encoded = value.get(..digits).ok_or("invalid syntax")?;
    let valid = match radix {
        8 => encoded.bytes().all(|byte| (b'0'..=b'7').contains(&byte)),
        16 => encoded.bytes().all(|byte| byte.is_ascii_hexdigit()),
        _ => false,
    };
    if !valid {
        return Err("invalid syntax");
    }
    let scalar = u32::from_str_radix(encoded, radix).map_err(|_| "invalid syntax")?;
    if scalar > 0xFF {
        return Err("invalid syntax");
    }
    let decoded = char::from_u32(scalar).ok_or("invalid syntax")?;
    Ok((
        DecodedChar {
            value: decoded,
            multibyte: false,
        },
        &value[digits..],
    ))
}

fn decode_unicode_escape(value: &str, digits: usize) -> Result<(DecodedChar, &str), &'static str> {
    let encoded = value.get(..digits).ok_or("invalid syntax")?;
    if !encoded.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("invalid syntax");
    }
    let scalar = u32::from_str_radix(encoded, 16).map_err(|_| "invalid syntax")?;
    let decoded = char::from_u32(scalar).ok_or("invalid syntax")?;
    Ok((
        DecodedChar {
            value: decoded,
            multibyte: decoded.len_utf8() > 1,
        },
        &value[digits..],
    ))
}

fn decode_octal_escape(first: char, value: &str) -> Result<(DecodedChar, &str), &'static str> {
    let tail = value.get(..2).ok_or("invalid syntax")?;
    let mut encoded = String::with_capacity(3);
    encoded.push(first);
    encoded.push_str(tail);
    decode_byte_escape(&encoded, 3, 8).map(|(decoded, _)| (decoded, &value[2..]))
}

fn decode_unquote_char_quote(value: i64) -> Result<Option<char>, &'static str> {
    match value {
        0 => Ok(None),
        34 => Ok(Some('"')),
        39 => Ok(Some('\'')),
        96 => Ok(Some('`')),
        _ => Err("invalid quote"),
    }
}

pub(super) fn quote_string_ascii(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        append_escaped_ascii_rune(&mut quoted, ch, '"');
    }
    quoted.push('"');
    quoted
}

pub(super) fn quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        append_escaped_rune(&mut quoted, ch, '"');
    }
    quoted.push('"');
    quoted
}

pub(super) fn quote_rune_ascii(value: i64) -> String {
    let rune = u32::try_from(value)
        .ok()
        .and_then(char::from_u32)
        .unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut quoted = String::with_capacity(10);
    quoted.push('\'');
    append_escaped_ascii_rune(&mut quoted, rune, '\'');
    quoted.push('\'');
    quoted
}

pub(super) fn quote_rune(value: i64) -> String {
    let rune = u32::try_from(value)
        .ok()
        .and_then(char::from_u32)
        .unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut quoted = String::with_capacity(10);
    quoted.push('\'');
    append_escaped_rune(&mut quoted, rune, '\'');
    quoted.push('\'');
    quoted
}

fn append_escaped_ascii_rune(out: &mut String, ch: char, quote: char) {
    if ch == quote || ch == '\\' {
        out.push('\\');
        out.push(ch);
        return;
    }
    if ch.is_ascii() && (' '..='~').contains(&ch) {
        out.push(ch);
        return;
    }
    match ch {
        '\u{07}' => out.push_str("\\a"),
        '\u{08}' => out.push_str("\\b"),
        '\u{0c}' => out.push_str("\\f"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\u{0b}' => out.push_str("\\v"),
        _ if ch.is_ascii() => {
            let value = ch as u32;
            out.push_str(&format!("\\x{value:02x}"));
        }
        _ => {
            let value = ch as u32;
            if value < 0x10000 {
                out.push_str(&format!("\\u{value:04x}"));
            } else {
                out.push_str(&format!("\\U{value:08x}"));
            }
        }
    }
}

fn append_escaped_rune(out: &mut String, ch: char, quote: char) {
    if ch == quote || ch == '\\' {
        out.push('\\');
        out.push(ch);
        return;
    }
    match ch {
        '\u{07}' => out.push_str("\\a"),
        '\u{08}' => out.push_str("\\b"),
        '\u{0c}' => out.push_str("\\f"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\u{0b}' => out.push_str("\\v"),
        _ if !ch.is_control() => out.push(ch),
        _ => append_escaped_ascii_rune(out, ch, quote),
    }
}
