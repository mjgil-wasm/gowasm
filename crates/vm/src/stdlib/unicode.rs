use super::{
    StdlibConstant, StdlibConstantValue, StdlibFunction, UNICODE_IS_CONTROL, UNICODE_IS_DIGIT,
    UNICODE_IS_GRAPHIC, UNICODE_IS_LETTER, UNICODE_IS_LOWER, UNICODE_IS_MARK, UNICODE_IS_NUMBER,
    UNICODE_IS_PRINT, UNICODE_IS_PUNCT, UNICODE_IS_SPACE, UNICODE_IS_SYMBOL, UNICODE_IS_TITLE,
    UNICODE_IS_UPPER, UNICODE_SIMPLE_FOLD, UNICODE_TO, UNICODE_TO_LOWER, UNICODE_TO_TITLE,
    UNICODE_TO_UPPER,
};
use crate::{Program, Value, ValueData, Vm, VmError};
use icu_casemap::{CaseMapper, ClosureSink};
use icu_properties::{props::WhiteSpace, CodePointSetData};
use unicode_general_category::{get_general_category, GeneralCategory};

pub(super) const UNICODE_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: UNICODE_IS_DIGIT,
        symbol: "IsDigit",
        returns_value: true,
        handler: unicode_is_digit,
    },
    StdlibFunction {
        id: UNICODE_IS_LETTER,
        symbol: "IsLetter",
        returns_value: true,
        handler: unicode_is_letter,
    },
    StdlibFunction {
        id: UNICODE_IS_SPACE,
        symbol: "IsSpace",
        returns_value: true,
        handler: unicode_is_space,
    },
    StdlibFunction {
        id: UNICODE_IS_UPPER,
        symbol: "IsUpper",
        returns_value: true,
        handler: unicode_is_upper,
    },
    StdlibFunction {
        id: UNICODE_IS_LOWER,
        symbol: "IsLower",
        returns_value: true,
        handler: unicode_is_lower,
    },
    StdlibFunction {
        id: UNICODE_IS_NUMBER,
        symbol: "IsNumber",
        returns_value: true,
        handler: unicode_is_number,
    },
    StdlibFunction {
        id: UNICODE_IS_PRINT,
        symbol: "IsPrint",
        returns_value: true,
        handler: unicode_is_print,
    },
    StdlibFunction {
        id: UNICODE_IS_GRAPHIC,
        symbol: "IsGraphic",
        returns_value: true,
        handler: unicode_is_graphic,
    },
    StdlibFunction {
        id: UNICODE_IS_PUNCT,
        symbol: "IsPunct",
        returns_value: true,
        handler: unicode_is_punct,
    },
    StdlibFunction {
        id: UNICODE_IS_SYMBOL,
        symbol: "IsSymbol",
        returns_value: true,
        handler: unicode_is_symbol,
    },
    StdlibFunction {
        id: UNICODE_IS_MARK,
        symbol: "IsMark",
        returns_value: true,
        handler: unicode_is_mark,
    },
    StdlibFunction {
        id: UNICODE_IS_TITLE,
        symbol: "IsTitle",
        returns_value: true,
        handler: unicode_is_title,
    },
    StdlibFunction {
        id: UNICODE_IS_CONTROL,
        symbol: "IsControl",
        returns_value: true,
        handler: unicode_is_control,
    },
    StdlibFunction {
        id: UNICODE_TO_UPPER,
        symbol: "ToUpper",
        returns_value: true,
        handler: unicode_to_upper,
    },
    StdlibFunction {
        id: UNICODE_TO_LOWER,
        symbol: "ToLower",
        returns_value: true,
        handler: unicode_to_lower,
    },
    StdlibFunction {
        id: UNICODE_TO_TITLE,
        symbol: "ToTitle",
        returns_value: true,
        handler: unicode_to_title,
    },
    StdlibFunction {
        id: UNICODE_TO,
        symbol: "To",
        returns_value: true,
        handler: unicode_to,
    },
    StdlibFunction {
        id: UNICODE_SIMPLE_FOLD,
        symbol: "SimpleFold",
        returns_value: true,
        handler: unicode_simple_fold,
    },
];

pub(super) const UNICODE_CONSTANTS: &[StdlibConstant] = &[
    StdlibConstant {
        symbol: "Version",
        typ: "string",
        value: StdlibConstantValue::String("15.0.0"),
    },
    StdlibConstant {
        symbol: "MaxRune",
        typ: "int",
        value: StdlibConstantValue::Int(0x10_FFFF),
    },
    StdlibConstant {
        symbol: "ReplacementChar",
        typ: "int",
        value: StdlibConstantValue::Int(0xFFFD),
    },
    StdlibConstant {
        symbol: "MaxASCII",
        typ: "int",
        value: StdlibConstantValue::Int(0x7F),
    },
    StdlibConstant {
        symbol: "MaxLatin1",
        typ: "int",
        value: StdlibConstantValue::Int(0xFF),
    },
    StdlibConstant {
        symbol: "UpperCase",
        typ: "int",
        value: StdlibConstantValue::Int(0),
    },
    StdlibConstant {
        symbol: "LowerCase",
        typ: "int",
        value: StdlibConstantValue::Int(1),
    },
    StdlibConstant {
        symbol: "TitleCase",
        typ: "int",
        value: StdlibConstantValue::Int(2),
    },
];

pub(super) fn unicode_is_digit(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsDigit", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_decimal_digit).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_letter(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsLetter", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_letter).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_space(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsSpace", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_space).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_upper(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsUpper", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_upper_letter).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_lower(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsLower", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_lower_letter).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_number(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsNumber", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_number).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_print(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsPrint", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_printable).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_graphic(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsGraphic", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_graphic).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_punct(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsPunct", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_punctuation).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_symbol(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsSymbol", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_symbol).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_mark(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsMark", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_mark).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_control(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsControl", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_control).unwrap_or(false),
    ))
}

pub(super) fn unicode_is_title(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.IsTitle", args)?;
    Ok(Value::bool(
        scalar_value(rune).map(is_titlecase).unwrap_or(false),
    ))
}

pub(super) fn unicode_to_upper(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.ToUpper", args)?;
    Ok(Value::int(map_rune(rune, |ch| {
        let mapped = CaseMapper::new().simple_uppercase(ch);
        if mapped == ch && ch == 'ß' {
            core::iter::once('ẞ')
        } else {
            core::iter::once(mapped)
        }
    })))
}

pub(super) fn unicode_to_lower(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.ToLower", args)?;
    Ok(Value::int(map_rune(rune, |ch| {
        core::iter::once(CaseMapper::new().simple_lowercase(ch))
    })))
}

pub(super) fn unicode_to_title(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.ToTitle", args)?;
    Ok(Value::int(map_rune(rune, |ch| {
        core::iter::once(CaseMapper::new().simple_titlecase(ch))
    })))
}

pub(super) fn unicode_simple_fold(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let rune = unicode_arg(vm, program, "unicode.SimpleFold", args)?;
    Ok(Value::int(
        scalar_value(rune)
            .map(simple_fold_cycle)
            .map(|ch| ch as i64)
            .unwrap_or(rune),
    ))
}

pub(super) fn unicode_to(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (case, rune) = unicode_case_and_rune_args(vm, program, "unicode.To", args)?;
    match case {
        0 => Ok(Value::int(map_rune(rune, |ch| {
            let mapped = CaseMapper::new().simple_uppercase(ch);
            if mapped == ch && ch == 'ß' {
                core::iter::once('ẞ')
            } else {
                core::iter::once(mapped)
            }
        }))),
        1 => Ok(Value::int(map_rune(rune, |ch| {
            core::iter::once(CaseMapper::new().simple_lowercase(ch))
        }))),
        2 => Ok(Value::int(map_rune(rune, |ch| {
            core::iter::once(CaseMapper::new().simple_titlecase(ch))
        }))),
        _ => Ok(Value::int(char::REPLACEMENT_CHARACTER as i64)),
    }
}

fn unicode_arg(
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

    let ValueData::Int(rune) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "an int argument".into(),
        });
    };
    Ok(rune)
}

fn unicode_case_and_rune_args(
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

    let ValueData::Int(case) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "int arguments".into(),
        });
    };
    let ValueData::Int(rune) = args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "int arguments".into(),
        });
    };
    Ok((case, rune))
}

fn scalar_value(rune: i64) -> Option<char> {
    u32::try_from(rune).ok().and_then(char::from_u32)
}

fn is_decimal_digit(ch: char) -> bool {
    matches!(get_general_category(ch), GeneralCategory::DecimalNumber)
}

fn is_number(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
    )
}

fn is_letter(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

fn is_upper_letter(ch: char) -> bool {
    matches!(get_general_category(ch), GeneralCategory::UppercaseLetter)
}

fn is_lower_letter(ch: char) -> bool {
    matches!(get_general_category(ch), GeneralCategory::LowercaseLetter)
}

fn is_space(ch: char) -> bool {
    match ch {
        '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | ' ' | '\u{0085}' | '\u{00A0}' => true,
        _ if (ch as u32) <= 0xFF => false,
        _ => CodePointSetData::new::<WhiteSpace>().contains(ch),
    }
}

fn is_control(ch: char) -> bool {
    (ch as u32) <= 0xFF && matches!(get_general_category(ch), GeneralCategory::Control)
}

fn is_graphic(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
            | GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::OtherPunctuation
            | GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::OtherSymbol
            | GeneralCategory::SpaceSeparator
    )
}

fn is_printable(ch: char) -> bool {
    ch == ' ' || (is_graphic(ch) && get_general_category(ch) != GeneralCategory::SpaceSeparator)
}

fn is_punctuation(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::OtherPunctuation
    )
}

fn is_symbol(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::OtherSymbol
    )
}

fn is_mark(ch: char) -> bool {
    matches!(
        get_general_category(ch),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    )
}

fn is_titlecase(ch: char) -> bool {
    matches!(get_general_category(ch), GeneralCategory::TitlecaseLetter)
}

#[derive(Default)]
struct CharClosure {
    chars: Vec<char>,
}

impl ClosureSink for CharClosure {
    fn add_char(&mut self, c: char) {
        self.chars.push(c);
    }

    fn add_string(&mut self, _string: &str) {}
}

fn simple_fold_cycle(ch: char) -> char {
    let mut closure = CharClosure::default();
    closure.chars.push(ch);
    CaseMapper::new().add_case_closure_to(ch, &mut closure);
    closure.chars.sort_unstable();
    closure.chars.dedup();
    closure
        .chars
        .iter()
        .copied()
        .find(|candidate| *candidate > ch)
        .unwrap_or(closure.chars[0])
}

fn map_rune<I>(rune: i64, mapper: impl Fn(char) -> I) -> i64
where
    I: Iterator<Item = char>,
{
    scalar_value(rune)
        .and_then(|ch| mapper(ch).next())
        .map(|ch| ch as i64)
        .unwrap_or(rune)
}
