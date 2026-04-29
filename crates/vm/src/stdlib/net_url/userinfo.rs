use super::url_shape::{userinfo_string_text, ParsedUserinfoFields};
use crate::{
    PointerTarget, Program, Value, ValueData, Vm, VmError, TYPE_URL_USERINFO, TYPE_URL_USERINFO_PTR,
};

const USERINFO_EXPECTED: &str = "a valid *url.Userinfo receiver";

pub(super) fn url_user(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(username) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.User".into(),
            expected: "a string argument".into(),
        });
    };

    Ok(userinfo_field_value(
        vm,
        Some(&ParsedUserinfoFields {
            username: username.clone(),
            password: String::new(),
            password_set: false,
        }),
    ))
}

pub(super) fn url_user_password(
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

    let ValueData::String(username) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.UserPassword".into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(password) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.UserPassword".into(),
            expected: "string arguments".into(),
        });
    };

    Ok(userinfo_field_value(
        vm,
        Some(&ParsedUserinfoFields {
            username: username.clone(),
            password: password.clone(),
            password_set: true,
        }),
    ))
}

pub(super) fn url_userinfo_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let Some(userinfo) = parsed_userinfo_receiver(vm, program, "(*url.Userinfo).String", &args[0])?
    else {
        return Ok(Value::string(String::new()));
    };

    Ok(Value::string(userinfo_string_text(
        &userinfo.username,
        userinfo.password_set.then_some(userinfo.password.as_str()),
    )))
}

pub(super) fn url_userinfo_username(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let Some(userinfo) =
        parsed_userinfo_receiver(vm, program, "(*url.Userinfo).Username", &args[0])?
    else {
        return Ok(Value::string(String::new()));
    };

    Ok(Value::string(userinfo.username))
}

pub(crate) fn url_userinfo_password(
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

    let Some(userinfo) =
        parsed_userinfo_receiver(vm, program, "(*url.Userinfo).Password", &args[0])?
    else {
        return Ok(vec![Value::string(String::new()), Value::bool(false)]);
    };

    Ok(vec![
        Value::string(userinfo.password),
        Value::bool(userinfo.password_set),
    ])
}

pub(super) fn optional_userinfo_field(
    vm: &Vm,
    program: &Program,
    fields: &[(String, Value)],
    builtin: &str,
    expected: &str,
    name: &str,
) -> Result<Option<ParsedUserinfoFields>, VmError> {
    let Some((_, value)) = fields.iter().find(|(field_name, _)| field_name == name) else {
        return Ok(None);
    };
    parsed_userinfo_value(vm, program, builtin, expected, value)
}

pub(super) fn userinfo_field_value(vm: &mut Vm, userinfo: Option<&ParsedUserinfoFields>) -> Value {
    match userinfo {
        Some(userinfo) => vm.box_heap_value(
            Value::struct_value(
                TYPE_URL_USERINFO,
                vec![
                    ("username".into(), Value::string(userinfo.username.clone())),
                    ("password".into(), Value::string(userinfo.password.clone())),
                    ("passwordSet".into(), Value::bool(userinfo.password_set)),
                ],
            ),
            TYPE_URL_USERINFO_PTR,
        ),
        None => Value::nil_pointer(TYPE_URL_USERINFO_PTR),
    }
}

pub(super) fn redacted_userinfo(
    userinfo: Option<&ParsedUserinfoFields>,
) -> Option<ParsedUserinfoFields> {
    let userinfo = userinfo?;
    if !userinfo.password_set {
        return Some(userinfo.clone());
    }
    Some(ParsedUserinfoFields {
        username: userinfo.username.clone(),
        password: "xxxxx".into(),
        password_set: true,
    })
}

fn parsed_userinfo_receiver(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<ParsedUserinfoFields>, VmError> {
    parsed_userinfo_value(vm, program, builtin, USERINFO_EXPECTED, value)
}

fn parsed_userinfo_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
    value: &Value,
) -> Result<Option<ParsedUserinfoFields>, VmError> {
    if matches!(
        &value.data,
        ValueData::Pointer(pointer) if matches!(&pointer.target, PointerTarget::Nil)
    ) {
        return Ok(None);
    }

    let userinfo = vm.deref_pointer(program, value)?;
    if userinfo.typ != TYPE_URL_USERINFO {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    }

    let ValueData::Struct(fields) = &userinfo.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    };

    Ok(Some(ParsedUserinfoFields {
        username: string_field(vm, program, fields, builtin, expected, "username")?.into(),
        password: string_field(vm, program, fields, builtin, expected, "password")?.into(),
        password_set: bool_field(vm, program, fields, builtin, expected, "passwordSet")?,
    }))
}

fn string_field<'a>(
    vm: &Vm,
    program: &Program,
    fields: &'a [(String, Value)],
    builtin: &str,
    expected: &str,
    name: &str,
) -> Result<&'a str, VmError> {
    let Some((_, value)) = fields.iter().find(|(field_name, _)| field_name == name) else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    };
    let ValueData::String(text) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    };
    Ok(text)
}

fn bool_field(
    vm: &Vm,
    program: &Program,
    fields: &[(String, Value)],
    builtin: &str,
    expected: &str,
    name: &str,
) -> Result<bool, VmError> {
    let Some((_, value)) = fields.iter().find(|(field_name, _)| field_name == name) else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    };
    let ValueData::Bool(value) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: expected.into(),
        });
    };
    Ok(*value)
}
