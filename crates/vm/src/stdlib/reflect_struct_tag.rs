use super::*;

const STRUCT_TAG_RECEIVER_EXPECTED: &str = "a reflect.StructTag receiver and string key argument";

pub(super) fn reflect_struct_tag_get(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (tag, key) = struct_tag_key_args(vm, program, "reflect.StructTag.Get", args)?;
    Ok(Value::string(
        lookup_struct_tag_value(tag, key).unwrap_or_default(),
    ))
}

pub(crate) fn reflect_struct_tag_lookup(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (tag, key) = struct_tag_key_args(vm, program, "reflect.StructTag.Lookup", args)?;
    let Some(value) = lookup_struct_tag_value(tag, key) else {
        return Ok(vec![Value::string(""), Value::bool(false)]);
    };
    Ok(vec![Value::string(value), Value::bool(true)])
}

fn struct_tag_key_args<'a>(
    vm: &Vm,
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

    let ValueData::String(tag) = &args[0].data else {
        return Err(invalid_struct_tag_argument(vm, program, builtin)?);
    };
    if args[0].typ != crate::TYPE_REFLECT_STRUCT_TAG {
        return Err(invalid_struct_tag_argument(vm, program, builtin)?);
    }

    let ValueData::String(key) = &args[1].data else {
        return Err(invalid_struct_tag_argument(vm, program, builtin)?);
    };

    Ok((tag, key))
}

fn invalid_struct_tag_argument(
    vm: &Vm,
    program: &Program,
    builtin: &str,
) -> Result<VmError, VmError> {
    Ok(VmError::InvalidStringFunctionArgument {
        function: vm.current_function_name(program)?,
        builtin: builtin.into(),
        expected: STRUCT_TAG_RECEIVER_EXPECTED.into(),
    })
}

fn lookup_struct_tag_value(tag: &str, key: &str) -> Option<String> {
    let mut remaining = tag;
    while !remaining.is_empty() {
        while matches!(remaining.as_bytes().first(), Some(b' ')) {
            remaining = &remaining[1..];
        }
        if remaining.is_empty() {
            break;
        }

        let bytes = remaining.as_bytes();
        let mut name_end = 0;
        while name_end < bytes.len() {
            let ch = bytes[name_end];
            if ch <= b' ' || ch == b':' || ch == b'"' || ch == 0x7f {
                break;
            }
            name_end += 1;
        }
        if name_end == 0 || name_end + 1 >= bytes.len() {
            break;
        }
        if bytes[name_end] != b':' || bytes[name_end + 1] != b'"' {
            break;
        }

        let name = &remaining[..name_end];
        remaining = &remaining[name_end + 1..];
        let quoted_end = quoted_value_end(remaining)?;
        let quoted_value = &remaining[..=quoted_end];
        remaining = &remaining[quoted_end + 1..];

        if name == key {
            return stdlib_unquote(quoted_value).ok();
        }
    }
    None
}

fn quoted_value_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some(index),
            b'\\' => {
                index += 1;
                if index >= bytes.len() {
                    return None;
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}
