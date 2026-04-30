use crate::{Program, Value, ValueData, Vm, VmError};
use icu_casemap::{CaseMapper, ClosureSink};

pub(super) fn string_pair_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::String(left) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(right) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok((left, right))
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

pub(super) fn equal_fold(left: &str, right: &str) -> bool {
    let mut left_chars = left.chars();
    let mut right_chars = right.chars();
    loop {
        match (left_chars.next(), right_chars.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if chars_equal_fold(left, right) => {}
            _ => return false,
        }
    }
}

fn chars_equal_fold(left: char, right: char) -> bool {
    if left == right {
        return true;
    }

    let mut closure = CharClosure::default();
    closure.chars.push(left);
    CaseMapper::new().add_case_closure_to(left, &mut closure);
    closure.chars.contains(&right)
}

pub(super) fn string_int_args<'a>(
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

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string and int argument".into(),
        });
    };
    let ValueData::Int(count) = args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string and int argument".into(),
        });
    };
    Ok((text, count))
}

pub(super) fn string_byte_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, u8), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string and byte argument".into(),
        });
    };
    let ValueData::Int(value) = args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string and byte argument".into(),
        });
    };
    let Ok(byte) = u8::try_from(value) else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string and byte argument".into(),
        });
    };
    Ok((text, byte))
}

pub(super) fn string_arg<'a>(
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

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };
    Ok(text)
}

pub(super) fn string_triple_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str, &'a str), VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(from) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(to) = &args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok((text, from, to))
}

pub(super) fn string_triple_int_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str, &'a str, i64), VmError> {
    if args.len() != 4 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 4,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    let ValueData::String(from) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    let ValueData::String(to) = &args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    let ValueData::Int(count) = args[3].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments plus an int count".into(),
        });
    };
    Ok((text, from, to, count))
}

pub(super) fn string_slice_and_string_args<'a>(
    vm: &mut Vm,
    program: &Program,
    args: &'a [Value],
) -> Result<(Vec<String>, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::Slice(slice) = &args[0].data else {
        return Err(VmError::InvalidJoinArgument {
            function: vm.current_function_name(program)?,
        });
    };
    let ValueData::String(sep) = &args[1].data else {
        return Err(VmError::InvalidJoinArgument {
            function: vm.current_function_name(program)?,
        });
    };

    let values = slice.values_snapshot();
    let mut parts = Vec::with_capacity(values.len());
    for value in &values {
        let ValueData::String(text) = &value.data else {
            return Err(VmError::InvalidJoinArgument {
                function: vm.current_function_name(program)?,
            });
        };
        parts.push(text.clone());
    }

    Ok((parts, sep))
}

pub(super) fn index_rune(text: &str, rune: i64) -> i64 {
    if (0..=0x7f).contains(&rune) {
        return text
            .find(rune as u8 as char)
            .map(|index| index as i64)
            .unwrap_or(-1);
    }
    if rune == 0xfffd {
        return text
            .char_indices()
            .find(|(_, ch)| *ch == char::REPLACEMENT_CHARACTER)
            .map(|(index, _)| index as i64)
            .unwrap_or(-1);
    }
    let Ok(rune) = u32::try_from(rune) else {
        return -1;
    };
    let Some(ch) = char::from_u32(rune) else {
        return -1;
    };
    text.find(ch).map(|index| index as i64).unwrap_or(-1)
}

pub(super) fn replace_n(text: &str, old: &str, new: &str, count: i64) -> String {
    if old == new || count == 0 {
        return text.to_string();
    }

    let match_count = if old.is_empty() {
        text.chars().count() + 1
    } else {
        text.match_indices(old).count()
    };
    if match_count == 0 {
        return text.to_string();
    }

    let count = if count < 0 {
        match_count
    } else {
        (count as usize).min(match_count)
    };

    if old.is_empty() {
        let mut boundaries = Vec::with_capacity(text.chars().count() + 1);
        boundaries.push(0);
        for (index, ch) in text.char_indices() {
            boundaries.push(index + ch.len_utf8());
        }

        let mut result = String::with_capacity(text.len() + count * new.len());
        let mut start = 0;
        for boundary in boundaries.into_iter().take(count) {
            result.push_str(&text[start..boundary]);
            result.push_str(new);
            start = boundary;
        }
        result.push_str(&text[start..]);
        return result;
    }

    let mut result = String::with_capacity(
        text.len() + count.saturating_mul(new.len().saturating_sub(old.len())),
    );
    let mut start = 0;
    for _ in 0..count {
        let index = start
            + text[start..]
                .find(old)
                .expect("match count should stay in sync");
        result.push_str(&text[start..index]);
        result.push_str(new);
        start = index + old.len();
    }
    result.push_str(&text[start..]);
    result
}
