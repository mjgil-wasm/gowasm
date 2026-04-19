use super::{
    url_shape::{
        escaped_fragment_text_with_hint, escaped_path_text_with_hint, userinfo_string_text,
    },
    userinfo::{optional_userinfo_field, redacted_userinfo, userinfo_field_value},
};
use crate::{PointerTarget, Program, Value, ValueData, Vm, VmError, TYPE_URL};

pub(super) fn url_url_string(
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

    let ValueData::Struct(fields) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.URL.String".into(),
            expected: "a url.URL receiver".into(),
        });
    };

    let scheme = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "Scheme",
    )?;
    let opaque = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "Opaque",
    )?;
    let host = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "Host",
    )?;
    let user = optional_userinfo_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "User",
    )?;
    let path = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "Path",
    )?;
    let raw_path = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "RawPath",
    )?;
    let force_query = bool_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "ForceQuery",
    )?;
    let raw_query = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "RawQuery",
    )?;
    let fragment = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "Fragment",
    )?;
    let raw_fragment = string_field(
        vm,
        program,
        fields,
        "url.URL.String",
        "a url.URL receiver",
        "RawFragment",
    )?;

    let mut rendered = String::new();
    if !scheme.is_empty() {
        rendered.push_str(scheme);
        rendered.push(':');
    }
    if !opaque.is_empty() {
        rendered.push_str(opaque);
    } else {
        if !scheme.is_empty() || !host.is_empty() || user.is_some() {
            if !host.is_empty() || !path.is_empty() || user.is_some() {
                rendered.push_str("//");
            }
            if let Some(user) = &user {
                rendered.push_str(&userinfo_string_text(
                    &user.username,
                    user.password_set.then_some(user.password.as_str()),
                ));
                rendered.push('@');
            }
        }
        if !host.is_empty() {
            rendered.push_str(host);
        }
        rendered.push_str(&escaped_path_text_with_hint(path, raw_path));
    }
    if force_query || !raw_query.is_empty() {
        rendered.push('?');
        rendered.push_str(raw_query);
    }
    if !fragment.is_empty() {
        rendered.push('#');
        rendered.push_str(&escaped_fragment_text_with_hint(fragment, raw_fragment));
    }

    Ok(Value::string(rendered))
}

pub(super) fn url_url_query(
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

    let url = url_receiver_value(vm, program, "(*url.URL).Query", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).Query".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let raw_query = string_field(
        vm,
        program,
        fields,
        "(*url.URL).Query",
        "a valid *url.URL receiver",
        "RawQuery",
    )?;
    Ok(super::values::parse_query_values(raw_query).0)
}

pub(super) fn url_url_redacted(
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

    if matches!(
        &args[0].data,
        ValueData::Pointer(pointer) if matches!(&pointer.target, PointerTarget::Nil)
    ) {
        return Ok(Value::string(String::new()));
    }

    let url = url_receiver_value(vm, program, "(*url.URL).Redacted", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).Redacted".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let redacted = redacted_userinfo(
        optional_userinfo_field(
            vm,
            program,
            fields,
            "(*url.URL).Redacted",
            "a valid *url.URL receiver",
            "User",
        )?
        .as_ref(),
    );
    let mut redacted_fields = fields.to_vec();
    if let Some((_, value)) = redacted_fields
        .iter_mut()
        .find(|(field_name, _)| field_name == "User")
    {
        *value = userinfo_field_value(vm, redacted.as_ref());
    } else if redacted.is_some() {
        redacted_fields.push(("User".into(), userinfo_field_value(vm, redacted.as_ref())));
    }

    url_url_string(
        vm,
        program,
        &[Value::struct_value(TYPE_URL, redacted_fields)],
    )
}

pub(super) fn url_url_is_abs(
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

    let url = url_receiver_value(vm, program, "(*url.URL).IsAbs", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).IsAbs".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let scheme = string_field(
        vm,
        program,
        fields,
        "(*url.URL).IsAbs",
        "a valid *url.URL receiver",
        "Scheme",
    )?;
    Ok(Value::bool(!scheme.is_empty()))
}

pub(super) fn url_url_hostname(
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

    let url = url_receiver_value(vm, program, "(*url.URL).Hostname", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).Hostname".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let host = string_field(
        vm,
        program,
        fields,
        "(*url.URL).Hostname",
        "a valid *url.URL receiver",
        "Host",
    )?;
    Ok(Value::string(split_host_port(host).0))
}

pub(super) fn url_url_port(
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

    let url = url_receiver_value(vm, program, "(*url.URL).Port", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).Port".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let host = string_field(
        vm,
        program,
        fields,
        "(*url.URL).Port",
        "a valid *url.URL receiver",
        "Host",
    )?;
    Ok(Value::string(split_host_port(host).1))
}

pub(super) fn url_url_marshal_binary(
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

    let url = url_receiver_value(vm, program, "(*url.URL).MarshalBinary", &args[0])?;
    let rendered = url_url_string(vm, program, &[url])?;
    let ValueData::String(text) = rendered.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).MarshalBinary".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    Ok(vec![bytes_to_value(text.as_bytes()), Value::nil()])
}

pub(super) fn url_url_unmarshal_binary(
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

    let _ = url_receiver_value(vm, program, "(*url.URL).UnmarshalBinary", &args[0])?;
    let bytes = byte_slice_arg(vm, program, "(*url.URL).UnmarshalBinary", &args[1])?;
    let parsed = super::url_parse(
        vm,
        program,
        &[Value::string(String::from_utf8_lossy(&bytes).into_owned())],
    )?;
    let Some(parsed_url) = parsed.first() else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).UnmarshalBinary".into(),
            expected: "a []byte argument".into(),
        });
    };
    let parsed_error = parsed.get(1).cloned().unwrap_or_else(Value::nil);
    if !matches!(parsed_error.data, ValueData::Nil) {
        return Ok(parsed_error);
    }

    let parsed_value = vm.deref_pointer(program, parsed_url)?;
    vm.store_indirect(program, &args[0], parsed_value)?;
    Ok(Value::nil())
}

pub(super) fn url_url_request_uri(
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

    let url = url_receiver_value(vm, program, "(*url.URL).RequestURI", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).RequestURI".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let path = string_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "Path",
    )?;
    let opaque = string_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "Opaque",
    )?;
    let raw_path = string_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "RawPath",
    )?;
    let force_query = bool_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "ForceQuery",
    )?;
    let raw_query = string_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "RawQuery",
    )?;
    let scheme = string_field(
        vm,
        program,
        fields,
        "(*url.URL).RequestURI",
        "a valid *url.URL receiver",
        "Scheme",
    )?;

    let mut result = if opaque.is_empty() {
        let mut path_text = escaped_path_text_with_hint(path, raw_path);
        if path_text.is_empty() {
            path_text.push('/');
        }
        path_text
    } else if opaque.starts_with("//") {
        format!("{scheme}:{opaque}")
    } else {
        opaque.into()
    };
    if force_query || !raw_query.is_empty() {
        result.push('?');
        result.push_str(raw_query);
    }
    Ok(Value::string(result))
}

pub(super) fn url_url_escaped_path(
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

    let url = url_receiver_value(vm, program, "(*url.URL).EscapedPath", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).EscapedPath".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let path = string_field(
        vm,
        program,
        fields,
        "(*url.URL).EscapedPath",
        "a valid *url.URL receiver",
        "Path",
    )?;
    let raw_path = string_field(
        vm,
        program,
        fields,
        "(*url.URL).EscapedPath",
        "a valid *url.URL receiver",
        "RawPath",
    )?;
    Ok(Value::string(escaped_path_text_with_hint(path, raw_path)))
}

pub(super) fn url_url_escaped_fragment(
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

    let url = url_receiver_value(vm, program, "(*url.URL).EscapedFragment", &args[0])?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).EscapedFragment".into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    let fragment = string_field(
        vm,
        program,
        fields,
        "(*url.URL).EscapedFragment",
        "a valid *url.URL receiver",
        "Fragment",
    )?;
    let raw_fragment = string_field(
        vm,
        program,
        fields,
        "(*url.URL).EscapedFragment",
        "a valid *url.URL receiver",
        "RawFragment",
    )?;
    Ok(Value::string(escaped_fragment_text_with_hint(
        fragment,
        raw_fragment,
    )))
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

fn url_receiver_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<Value, VmError> {
    let url = vm.deref_pointer(program, receiver)?;
    if url.typ != TYPE_URL {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a valid *url.URL receiver".into(),
        });
    }
    let ValueData::Struct(_) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };
    Ok(url)
}

fn split_host_port(host_port: &str) -> (String, String) {
    let mut host = host_port;
    let mut port = "";

    if let Some(colon) = host.rfind(':') {
        if valid_optional_port(&host[colon..]) {
            host = &host[..colon];
            port = &host_port[colon + 1..];
        }
    }

    if host.starts_with('[') && host.ends_with(']') {
        host = &host[1..host.len() - 1];
    }

    (host.into(), port.into())
}

fn bytes_to_value(data: &[u8]) -> Value {
    Value::slice(data.iter().map(|&byte| Value::int(byte as i64)).collect())
}

fn byte_slice_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
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
                function: vm
                    .current_function_name(program)
                    .unwrap_or_else(|_| builtin.into()),
                builtin: builtin.into(),
                expected: "a []byte argument".into(),
            }),
        })
        .collect()
}

fn valid_optional_port(port: &str) -> bool {
    if port.is_empty() {
        return true;
    }
    port.starts_with(':') && port[1..].bytes().all(|byte| byte.is_ascii_digit())
}
