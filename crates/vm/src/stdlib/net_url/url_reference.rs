use super::url_shape::{
    escaped_path_text_with_hint, parsed_path_fields_from_text, resolve_path_text, ParsedUrlFields,
};
use super::userinfo::{optional_userinfo_field, userinfo_field_value};
use crate::{Program, Value, ValueData, Vm, VmError, TYPE_URL, TYPE_URL_PTR};

pub(super) fn url_url_resolve_reference(
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

    let base = url_receiver_fields(vm, program, "(*url.URL).ResolveReference", &args[0])?;
    let reference = url_receiver_fields(vm, program, "(*url.URL).ResolveReference", &args[1])?;
    let resolved = resolve_reference_fields(&base, &reference);
    let resolved = url_value_from_fields(vm, resolved);
    Ok(vm.box_heap_value(resolved, TYPE_URL_PTR))
}

pub(super) fn url_join_path(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }

    let parsed = super::url_parse(vm, program, std::slice::from_ref(&args[0]))?;
    let Some(parsed_base) = parsed.first().cloned() else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.JoinPath".into(),
            expected: "a string argument".into(),
        });
    };
    let parsed_error = parsed.get(1).cloned().unwrap_or_else(Value::nil);
    if !matches!(parsed_error.data, ValueData::Nil) {
        return Ok(vec![Value::string(String::new()), parsed_error]);
    }

    let base = url_receiver_fields(vm, program, "url.JoinPath", &parsed_base)?;
    let elements = string_args_from(vm, program, "url.JoinPath", args, 1, "string arguments")?;
    let joined = join_path_fields(&base, &elements);
    let joined = url_value_from_fields(vm, joined);
    let rendered = super::url_methods::url_url_string(vm, program, &[joined])?;
    let ValueData::String(text) = rendered.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.JoinPath".into(),
            expected: "a string result".into(),
        });
    };
    Ok(vec![Value::string(text), Value::nil()])
}

pub(super) fn url_url_join_path(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }

    let base = url_receiver_fields(vm, program, "(*url.URL).JoinPath", &args[0])?;
    let elements = string_args_from(
        vm,
        program,
        "(*url.URL).JoinPath",
        args,
        1,
        "string arguments",
    )?;
    let joined = join_path_fields(&base, &elements);
    let joined = url_value_from_fields(vm, joined);
    Ok(vm.box_heap_value(joined, TYPE_URL_PTR))
}

pub(super) fn url_url_parse(
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

    let parsed = super::url_parse(vm, program, std::slice::from_ref(&args[1]))?;
    let Some(parsed_reference) = parsed.first().cloned() else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*url.URL).Parse".into(),
            expected: "a string argument".into(),
        });
    };
    let parsed_error = parsed.get(1).cloned().unwrap_or_else(Value::nil);
    if !matches!(parsed_error.data, ValueData::Nil) {
        return Ok(vec![Value::nil_pointer(TYPE_URL_PTR), parsed_error]);
    }

    let resolved = url_url_resolve_reference(vm, program, &[args[0].clone(), parsed_reference])?;
    Ok(vec![resolved, Value::nil()])
}

fn url_receiver_fields(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<ParsedUrlFields, VmError> {
    let url = url_receiver_value(vm, program, builtin, receiver)?;
    let ValueData::Struct(fields) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a valid *url.URL receiver".into(),
        });
    };

    Ok(ParsedUrlFields {
        scheme: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "Scheme",
        )?
        .into(),
        opaque: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "Opaque",
        )?
        .into(),
        user: optional_userinfo_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "User",
        )?,
        host: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "Host",
        )?
        .into(),
        path: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "Path",
        )?
        .into(),
        raw_path: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "RawPath",
        )?
        .into(),
        force_query: bool_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "ForceQuery",
        )?,
        raw_query: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "RawQuery",
        )?
        .into(),
        fragment: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "Fragment",
        )?
        .into(),
        raw_fragment: string_field(
            vm,
            program,
            fields,
            builtin,
            "a valid *url.URL receiver",
            "RawFragment",
        )?
        .into(),
    })
}

fn url_value_from_fields(vm: &mut Vm, fields: ParsedUrlFields) -> Value {
    let user = userinfo_field_value(vm, fields.user.as_ref());
    Value::struct_value(
        TYPE_URL,
        vec![
            ("Scheme".into(), Value::string(fields.scheme)),
            ("Opaque".into(), Value::string(fields.opaque)),
            ("User".into(), user),
            ("Host".into(), Value::string(fields.host)),
            ("Path".into(), Value::string(fields.path)),
            ("RawPath".into(), Value::string(fields.raw_path)),
            ("ForceQuery".into(), Value::bool(fields.force_query)),
            ("RawQuery".into(), Value::string(fields.raw_query)),
            ("Fragment".into(), Value::string(fields.fragment)),
            ("RawFragment".into(), Value::string(fields.raw_fragment)),
        ],
    )
}

fn resolve_reference_fields(
    base: &ParsedUrlFields,
    reference: &ParsedUrlFields,
) -> ParsedUrlFields {
    let mut url = reference.clone();
    if reference.scheme.is_empty() {
        url.scheme = base.scheme.clone();
    }

    if !reference.scheme.is_empty() || !reference.host.is_empty() || reference.user.is_some() {
        let escaped_reference = escaped_path_text_with_hint(&reference.path, &reference.raw_path);
        let resolved_path = resolve_path_text(&escaped_reference, "");
        let (path, raw_path) = parsed_path_fields_from_text(&resolved_path)
            .expect("resolved absolute reference path should stay valid");
        url.path = path;
        url.raw_path = raw_path;
        return url;
    }

    if !reference.opaque.is_empty() {
        url.user = None;
        url.host.clear();
        url.path.clear();
        url.raw_path.clear();
        return url;
    }

    if reference.path.is_empty() && !reference.force_query && reference.raw_query.is_empty() {
        url.raw_query = base.raw_query.clone();
        if reference.fragment.is_empty() {
            url.fragment = base.fragment.clone();
            url.raw_fragment = base.raw_fragment.clone();
        }
    }

    if reference.path.is_empty() && !base.opaque.is_empty() {
        url.opaque = base.opaque.clone();
        url.user = None;
        url.host.clear();
        url.path.clear();
        url.raw_path.clear();
        return url;
    }

    let escaped_base = escaped_path_text_with_hint(&base.path, &base.raw_path);
    let escaped_reference = escaped_path_text_with_hint(&reference.path, &reference.raw_path);
    let resolved_path = resolve_path_text(&escaped_base, &escaped_reference);
    let (path, raw_path) = parsed_path_fields_from_text(&resolved_path)
        .expect("resolved relative reference path should stay valid");
    url.user = base.user.clone();
    url.host = base.host.clone();
    url.path = path;
    url.raw_path = raw_path;
    url
}

fn join_path_fields(base: &ParsedUrlFields, elements: &[String]) -> ParsedUrlFields {
    let mut url = base.clone();
    let mut parts = Vec::with_capacity(elements.len() + 1);
    parts.push(escaped_path_text_with_hint(&base.path, &base.raw_path));
    parts.extend(elements.iter().cloned());

    let preserve_trailing = parts.last().is_some_and(|part| part.ends_with('/'));
    let mut joined = if parts.first().is_some_and(|part| !part.starts_with('/')) {
        parts[0].insert(0, '/');
        let joined = path_join_text(&parts);
        joined
            .strip_prefix('/')
            .expect("relative JoinPath text should keep its synthetic leading slash")
            .to_owned()
    } else {
        path_join_text(&parts)
    };

    if preserve_trailing && !joined.ends_with('/') {
        joined.push('/');
    }

    if let Ok((path, raw_path)) = parsed_path_fields_from_text(&joined) {
        url.path = path;
        url.raw_path = raw_path;
    }
    url
}

fn path_join_text(parts: &[String]) -> String {
    let items: Vec<&str> = parts.iter().map(String::as_str).collect();
    super::super::path_impl::join(&items)
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

fn string_args_from(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
    start: usize,
    expected: &str,
) -> Result<Vec<String>, VmError> {
    let mut values = Vec::with_capacity(args.len().saturating_sub(start));
    for arg in &args[start..] {
        let ValueData::String(text) = &arg.data else {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: expected.into(),
            });
        };
        values.push(text.clone());
    }
    Ok(values)
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
