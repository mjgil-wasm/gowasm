use super::{
    errors_impl::joined_error_value, StdlibFunction, FMT_ERRORF, FMT_PRINT, FMT_PRINTF,
    FMT_PRINTLN, FMT_SPRINT, FMT_SPRINTF, FMT_SPRINTLN,
};
use crate::{format_value, Float64, Program, Value, ValueData, Vm, VmError};

#[path = "fmt_render.rs"]
mod render;

pub(super) use render::render_print_value;
use render::{render_pointer_verb, render_type_verb, render_verb_value};

pub(super) const FMT_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: FMT_PRINTLN,
        symbol: "Println",
        returns_value: false,
        handler: fmt_println,
    },
    StdlibFunction {
        id: FMT_SPRINTF,
        symbol: "Sprintf",
        returns_value: true,
        handler: fmt_sprintf,
    },
    StdlibFunction {
        id: FMT_PRINTF,
        symbol: "Printf",
        returns_value: false,
        handler: fmt_printf,
    },
    StdlibFunction {
        id: FMT_SPRINT,
        symbol: "Sprint",
        returns_value: true,
        handler: fmt_sprint,
    },
    StdlibFunction {
        id: FMT_SPRINTLN,
        symbol: "Sprintln",
        returns_value: true,
        handler: fmt_sprintln,
    },
    StdlibFunction {
        id: FMT_ERRORF,
        symbol: "Errorf",
        returns_value: true,
        handler: fmt_errorf,
    },
    StdlibFunction {
        id: FMT_PRINT,
        symbol: "Print",
        returns_value: false,
        handler: fmt_print,
    },
];

fn fmt_println(vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    for (index, value) in args.iter().enumerate() {
        if index > 0 {
            vm.stdout.push(' ');
        }
        let rendered = render_print_value(vm, _program, value)?;
        vm.stdout.push_str(&rendered);
    }
    vm.stdout.push('\n');
    Ok(Value::nil())
}

fn fmt_sprintf(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let result = sprintf_impl(_vm, _program, args)?;
    Ok(Value::string(result))
}

fn fmt_printf(vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let result = sprintf_impl(vm, _program, args)?;
    vm.stdout.push_str(&result);
    Ok(Value::nil())
}

fn fmt_print(vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let rendered = sprint_impl(vm, _program, args)?;
    vm.stdout.push_str(&rendered);
    Ok(Value::nil())
}

fn fmt_sprint(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::string(sprint_impl(_vm, _program, args)?))
}

fn fmt_sprintln(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::string(sprintln_impl(_vm, _program, args)?))
}

pub(super) fn sprint_impl(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<String, VmError> {
    let mut out = String::new();
    for (index, value) in args.iter().enumerate() {
        if index > 0 && !is_string_value(value) && !is_string_value(&args[index - 1]) {
            out.push(' ');
        }
        out.push_str(&render_print_value(vm, program, value)?);
    }
    Ok(out)
}

pub(super) fn sprintln_impl(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<String, VmError> {
    let mut out = String::new();
    for (index, value) in args.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(&render_print_value(vm, program, value)?);
    }
    out.push('\n');
    Ok(out)
}

fn fmt_errorf(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Ok(Value::error(""));
    }
    let wrapped = find_wrapped_errors(args);
    let patched_args = patch_w_verb(args);
    let effective = patched_args.as_deref().unwrap_or(args);
    let message = sprintf_impl(_vm, _program, effective)?;
    match wrapped.len() {
        0 => Ok(Value::error(message)),
        1 => Ok(Value::wrapped_error(
            message,
            wrapped.into_iter().next().unwrap(),
        )),
        _ => Ok(joined_error_value(message, wrapped)),
    }
}

fn find_wrapped_errors(args: &[Value]) -> Vec<Value> {
    let format_str = match &args[0].data {
        ValueData::String(s) => s.as_str(),
        _ => return Vec::new(),
    };
    let format_args = &args[1..];
    let mut arg_index = 0;
    let mut wrapped = Vec::new();
    let bytes = format_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        i += 1;
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'%' {
            i += 1;
            continue;
        }
        while i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b' ' | b'0' | b'#') {
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
        if i >= bytes.len() {
            break;
        }
        let verb = bytes[i];
        i += 1;
        if verb == b'w' {
            if let Some(arg) = format_args.get(arg_index) {
                wrapped.push(arg.clone());
            }
        }
        arg_index += 1;
    }
    wrapped
}

fn patch_w_verb(args: &[Value]) -> Option<Vec<Value>> {
    let format_str = match &args[0].data {
        ValueData::String(s) => s.clone(),
        _ => return None,
    };
    if !format_str.contains("%w") {
        return None;
    }
    let patched = format_str.replace("%w", "%v");
    let mut result = vec![Value::string(patched)];
    result.extend_from_slice(&args[1..]);
    Some(result)
}

fn is_string_value(value: &Value) -> bool {
    matches!(&value.data, ValueData::String(_))
}

pub(super) fn sprintf_impl(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<String, VmError> {
    if args.is_empty() {
        return Ok(String::new());
    }
    let format_str = match &args[0].data {
        ValueData::String(s) => s.as_str(),
        _ => return render_print_value(vm, program, &args[0]),
    };
    let format_args = &args[1..];
    let mut out = String::new();
    let mut arg_index = 0;
    let bytes = format_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }
        i += 1;
        if i >= bytes.len() {
            out.push('%');
            break;
        }
        if bytes[i] == b'%' {
            out.push('%');
            i += 1;
            continue;
        }
        let mut flags = FormatFlags::default();
        while i < bytes.len() {
            match bytes[i] {
                b'-' => flags.left_align = true,
                b'+' => flags.plus_sign = true,
                b' ' => flags.space_sign = true,
                b'0' => flags.zero_pad = true,
                b'#' => flags.alternate = true,
                _ => break,
            }
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'*' {
            if arg_index < format_args.len() {
                if let ValueData::Int(w) = &format_args[arg_index].data {
                    flags.width = Some(*w as usize);
                }
                arg_index += 1;
            }
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i > start {
                flags.width = std::str::from_utf8(&bytes[start..i])
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i < bytes.len() && bytes[i] == b'*' {
                if arg_index < format_args.len() {
                    if let ValueData::Int(p) = &format_args[arg_index].data {
                        flags.precision = Some(*p as usize);
                    }
                    arg_index += 1;
                }
                i += 1;
            } else {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                flags.precision = if i > start {
                    std::str::from_utf8(&bytes[start..i])
                        .ok()
                        .and_then(|s| s.parse().ok())
                } else {
                    Some(0)
                };
            }
        }
        if i >= bytes.len() {
            out.push_str("%!(NOVERB)");
            break;
        }
        let verb = bytes[i] as char;
        i += 1;
        if arg_index < format_args.len() {
            let formatted = format_verb(vm, program, verb, &format_args[arg_index], &flags)?;
            out.push_str(&formatted);
            arg_index += 1;
        } else {
            out.push_str(&format!("%!{verb}(MISSING)"));
        }
    }
    for extra in format_args.iter().skip(arg_index) {
        out.push_str(&format!(
            "%!(EXTRA {})",
            render_print_value(vm, program, extra)?
        ));
    }
    Ok(out)
}

#[derive(Default)]
struct FormatFlags {
    width: Option<usize>,
    precision: Option<usize>,
    left_align: bool,
    plus_sign: bool,
    space_sign: bool,
    zero_pad: bool,
    alternate: bool,
}

fn format_verb(
    vm: &mut Vm,
    program: &Program,
    verb: char,
    value: &Value,
    flags: &FormatFlags,
) -> Result<String, VmError> {
    let raw = match verb {
        'v' | 's' | 'q' => render_verb_value(vm, program, verb, value, flags)?,
        'd' => match &value.data {
            ValueData::Int(n) => format_int_with_sign(*n, flags),
            ValueData::Bool(b) => {
                if *b {
                    "1".into()
                } else {
                    "0".into()
                }
            }
            _ => format!("%!d({})", format_value(value)),
        },
        'f' => format_float_f(value, flags),
        'e' => format_float_e(value, flags, 'e'),
        'E' => format_float_e(value, flags, 'E'),
        'g' => format_float_g(value, flags, 'e'),
        'G' => format_float_g(value, flags, 'E'),
        't' => match &value.data {
            ValueData::Bool(b) => b.to_string(),
            _ => format!("%!t({})", format_value(value)),
        },
        'c' => match &value.data {
            ValueData::Int(n) => char::from_u32(*n as u32)
                .map(|c| c.to_string())
                .unwrap_or_else(|| format!("{}", char::REPLACEMENT_CHARACTER)),
            _ => format!("%!c({})", format_value(value)),
        },
        'x' => format_int_base(value, 16, false, flags),
        'X' => format_int_base(value, 16, true, flags),
        'o' => format_int_base(value, 8, false, flags),
        'O' => {
            let s = format_int_base(value, 8, false, flags);
            if !s.starts_with("0o") && !s.starts_with("%!") {
                if let Some(rest) = s.strip_prefix('-') {
                    format!("-0o{rest}")
                } else {
                    format!("0o{s}")
                }
            } else {
                s
            }
        }
        'b' => format_int_base(value, 2, false, flags),
        'T' => render_type_verb(vm, program, value),
        'p' => match render_pointer_verb(value) {
            Some(pointer) => pointer,
            None => format!("%!p({})", format_value(value)),
        },
        _ => format!("%!{verb}({})", format_value(value)),
    };
    Ok(apply_width(&raw, flags))
}

fn format_int_with_sign(n: i64, flags: &FormatFlags) -> String {
    if flags.plus_sign {
        format!("{n:+}")
    } else if flags.space_sign && n >= 0 {
        format!(" {n}")
    } else {
        n.to_string()
    }
}

fn format_int_base(value: &Value, base: u32, upper: bool, flags: &FormatFlags) -> String {
    let n = match &value.data {
        ValueData::Int(n) => *n,
        ValueData::String(s) => {
            if base == 16 {
                return s
                    .bytes()
                    .map(|b| {
                        if upper {
                            format!("{b:02X}")
                        } else {
                            format!("{b:02x}")
                        }
                    })
                    .collect();
            }
            return format!(
                "%!{}({})",
                if upper { 'X' } else { 'x' },
                format_value(value)
            );
        }
        _ => {
            return format!(
                "%!{}({})",
                if upper { 'X' } else { 'x' },
                format_value(value)
            )
        }
    };
    let abs = (n as i128).unsigned_abs();
    let digits = match base {
        2 => format!("{abs:b}"),
        8 => format!("{abs:o}"),
        16 => {
            if upper {
                format!("{abs:X}")
            } else {
                format!("{abs:x}")
            }
        }
        _ => unreachable!(),
    };
    let prefix = if flags.alternate {
        match base {
            2 => "0b",
            8 => "0",
            16 => {
                if upper {
                    "0X"
                } else {
                    "0x"
                }
            }
            _ => "",
        }
    } else {
        ""
    };
    if n < 0 {
        format!("-{prefix}{digits}")
    } else {
        format!("{prefix}{digits}")
    }
}

fn format_float_f(value: &Value, flags: &FormatFlags) -> String {
    match &value.data {
        ValueData::Float(Float64(f)) => {
            if f.is_nan() {
                return "NaN".into();
            }
            if f.is_infinite() {
                return if f.is_sign_positive() {
                    "+Inf".into()
                } else {
                    "-Inf".into()
                };
            }
            let prec = flags.precision.unwrap_or(6);
            let mut s = format!("{f:.prec$}");
            if flags.plus_sign && *f >= 0.0 && !f.is_nan() {
                s.insert(0, '+');
            } else if flags.space_sign && *f >= 0.0 && !f.is_nan() {
                s.insert(0, ' ');
            }
            s
        }
        ValueData::Int(n) => {
            let prec = flags.precision.unwrap_or(6);
            format!("{:.prec$}", *n as f64)
        }
        _ => format!("%!f({})", format_value(value)),
    }
}

fn format_float_e(value: &Value, flags: &FormatFlags, exp_char: char) -> String {
    match &value.data {
        ValueData::Float(Float64(f)) => {
            let prec = flags.precision.unwrap_or(6);
            let mut s = format_scientific(*f, prec, exp_char);
            if flags.plus_sign && *f >= 0.0 && !f.is_nan() {
                s.insert(0, '+');
            }
            s
        }
        _ => format!("%!{exp_char}({})", format_value(value)),
    }
}

fn format_float_g(value: &Value, flags: &FormatFlags, exp_char: char) -> String {
    match &value.data {
        ValueData::Float(Float64(f)) => {
            let prec = flags.precision.unwrap_or(usize::MAX);
            if prec == 0 {
                return format_float_e(value, flags, exp_char);
            }
            let e_str = format_scientific(*f, 10, exp_char);
            let exp_val = parse_exponent(&e_str);
            if exp_val < -4 || exp_val >= prec as i32 {
                format_float_e(
                    value,
                    &FormatFlags {
                        precision: Some(prec.saturating_sub(1)),
                        ..*flags
                    },
                    exp_char,
                )
            } else {
                format_float_f(
                    value,
                    &FormatFlags {
                        precision: Some(
                            prec.saturating_sub(1)
                                .saturating_sub(exp_val.max(0) as usize),
                        ),
                        ..*flags
                    },
                )
            }
        }
        _ => format!("%!{exp_char}({})", format_value(value)),
    }
}

fn format_scientific(f: f64, prec: usize, exp_char: char) -> String {
    if f == 0.0 {
        return format!("0.{:0>width$}{exp_char}+00", "", width = prec);
    }
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
    let abs = f.abs();
    let exp = abs.log10().floor() as i32;
    let mantissa = abs / 10.0_f64.powi(exp);
    let sign = if f < 0.0 { "-" } else { "" };
    format!("{sign}{mantissa:.prec$}{exp_char}{:+03}", exp)
}

fn parse_exponent(s: &str) -> i32 {
    if let Some(pos) = s.rfind(['e', 'E']) {
        s[pos + 1..].parse().unwrap_or(0)
    } else {
        0
    }
}

fn maybe_truncate(s: &str, precision: Option<usize>) -> String {
    match precision {
        Some(p) => s.chars().take(p).collect(),
        None => s.to_string(),
    }
}

fn apply_width(s: &str, flags: &FormatFlags) -> String {
    let Some(width) = flags.width else {
        return s.to_string();
    };
    if flags.left_align {
        let len = s.chars().count();
        if len >= width {
            return s.to_string();
        }
        let pad = width - len;
        format!("{s}{}", " ".repeat(pad))
    } else if flags.zero_pad && !flags.left_align {
        let (sign, rest) = if s.starts_with('-') || s.starts_with('+') || s.starts_with(' ') {
            (&s[..1], &s[1..])
        } else {
            ("", s)
        };
        let (prefix, suffix) = if rest.starts_with("0x")
            || rest.starts_with("0X")
            || rest.starts_with("0b")
            || rest.starts_with("0B")
            || rest.starts_with("0o")
        {
            (&rest[..2], &rest[2..])
        } else if rest.starts_with('0') && rest.len() > 1 {
            (&rest[..1], &rest[1..])
        } else {
            ("", rest)
        };
        let effective_len = sign.chars().count() + suffix.chars().count();
        if effective_len >= width {
            return s.to_string();
        }
        let pad = width - effective_len;
        format!("{sign}{prefix}{}{suffix}", "0".repeat(pad))
    } else {
        let len = s.chars().count();
        if len >= width {
            return s.to_string();
        }
        let pad = width - len;
        format!("{}{s}", " ".repeat(pad))
    }
}
