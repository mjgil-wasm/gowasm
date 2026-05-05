use super::json_tags_impl::{
    json_field_value_by_path, json_struct_field_specs, json_value_is_empty,
};
use super::{StdlibFunction, JSON_MARSHAL, JSON_MARSHAL_INDENT, JSON_UNMARSHAL, JSON_VALID};
use crate::{describe_value, MapValue, PointerTarget, Program, Value, ValueData, Vm, VmError};

pub(super) const JSON_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: JSON_MARSHAL,
        symbol: "Marshal",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: JSON_MARSHAL_INDENT,
        symbol: "MarshalIndent",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: JSON_VALID,
        symbol: "Valid",
        returns_value: true,
        handler: json_valid,
    },
    StdlibFunction {
        id: JSON_UNMARSHAL,
        symbol: "Unmarshal",
        returns_value: true,
        handler: super::json_decode_impl::json_unmarshal,
    },
];

#[derive(Clone, Copy)]
enum JsonLayout<'a> {
    Compact,
    Pretty { prefix: &'a str, indent: &'a str },
}

enum JsonMarshalError {
    Message(String),
    Vm(VmError),
}

impl From<String> for JsonMarshalError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for JsonMarshalError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}

enum CustomMarshalKind {
    Json,
    Text,
}

pub(super) fn json_marshal(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    marshal_json_result(vm, program, args, JsonLayout::Compact)
}

pub(super) fn json_marshal_indent(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let ValueData::String(prefix) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "json.MarshalIndent".into(),
            expected: "value, prefix string, indent string".into(),
        });
    };
    let ValueData::String(indent) = &args[2].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "json.MarshalIndent".into(),
            expected: "value, prefix string, indent string".into(),
        });
    };
    marshal_json_result(
        vm,
        program,
        &args[..1],
        JsonLayout::Pretty {
            prefix: prefix.as_str(),
            indent: indent.as_str(),
        },
    )
}

fn marshal_json_result(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    layout: JsonLayout<'_>,
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let mut seen_pointers = Vec::new();
    match marshal_json_value(vm, program, &args[0], &mut seen_pointers, layout, 0) {
        Ok(json) => Ok(vec![bytes_to_value(json.as_bytes()), Value::nil()]),
        Err(JsonMarshalError::Message(error)) => Ok(vec![Value::nil_slice(), Value::error(error)]),
        Err(JsonMarshalError::Vm(error)) => Err(error),
    }
}

fn json_valid(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let bytes = byte_slice_arg(vm, program, "json.Valid", args)?;
    Ok(Value::bool(
        serde_json::from_slice::<serde_json::Value>(&bytes).is_ok(),
    ))
}

fn marshal_json_value(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    seen_pointers: &mut Vec<PointerTarget>,
    layout: JsonLayout<'_>,
    depth: usize,
) -> Result<String, JsonMarshalError> {
    if matches!(&value.data, ValueData::Nil) {
        return Ok("null".into());
    }
    if matches!(&value.data, ValueData::Pointer(pointer) if pointer.is_nil()) {
        return Ok("null".into());
    }
    if let Some(custom) = marshal_json_custom_value(vm, program, value)? {
        return Ok(custom);
    }

    match &value.data {
        ValueData::Nil => Ok("null".into()),
        ValueData::Int(number) => Ok(number.to_string()),
        ValueData::Float(number) => marshal_float(number.0).map_err(Into::into),
        ValueData::String(text) => Ok(quote_json_string(text)),
        ValueData::Bool(boolean) => Ok(boolean.to_string()),
        ValueData::Error(_) => Err("json: unsupported type: error".into()),
        ValueData::Array(array) => marshal_json_array(
            vm,
            program,
            &array.values_snapshot(),
            seen_pointers,
            layout,
            depth,
        ),
        ValueData::Slice(slice) => {
            if slice.is_nil {
                Ok("null".into())
            } else {
                marshal_json_array(
                    vm,
                    program,
                    &slice.values_snapshot(),
                    seen_pointers,
                    layout,
                    depth,
                )
            }
        }
        ValueData::Map(map) => marshal_json_map(vm, program, map, seen_pointers, layout, depth),
        ValueData::Channel(_) => Err("json: unsupported type: channel".into()),
        ValueData::Pointer(pointer) => {
            if pointer.is_nil() {
                return Ok("null".into());
            }
            if seen_pointers.contains(&pointer.target) {
                return Err("json: unsupported value: encountered cyclic pointer".into());
            }
            seen_pointers.push(pointer.target.clone());
            let target = vm
                .deref_pointer(program, value)
                .map_err(JsonMarshalError::Vm)?;
            let result = marshal_json_value(vm, program, &target, seen_pointers, layout, depth);
            seen_pointers.pop();
            result
        }
        ValueData::Function(_) => Err("json: unsupported type: function".into()),
        ValueData::Struct(fields) => {
            marshal_json_struct(vm, program, value, fields, seen_pointers, layout, depth)
        }
    }
}

fn marshal_float(value: f64) -> Result<String, String> {
    if value.is_nan() {
        return Err("json: unsupported value: NaN".into());
    }
    if value.is_infinite() {
        return Err(if value.is_sign_positive() {
            "json: unsupported value: +Inf".into()
        } else {
            "json: unsupported value: -Inf".into()
        });
    }
    Ok(value.to_string())
}

fn marshal_json_array(
    vm: &mut Vm,
    program: &Program,
    values: &[Value],
    seen_pointers: &mut Vec<PointerTarget>,
    layout: JsonLayout<'_>,
    depth: usize,
) -> Result<String, JsonMarshalError> {
    let mut encoded = Vec::with_capacity(values.len());
    for value in values {
        encoded.push(marshal_json_value(
            vm,
            program,
            value,
            seen_pointers,
            layout,
            depth + 1,
        )?);
    }
    render_collection('[', ']', &encoded, layout, depth)
}

fn marshal_json_map(
    vm: &mut Vm,
    program: &Program,
    map: &MapValue,
    seen_pointers: &mut Vec<PointerTarget>,
    layout: JsonLayout<'_>,
    depth: usize,
) -> Result<String, JsonMarshalError> {
    let Some(entries) = &map.entries else {
        return Ok("null".into());
    };
    let entries = entries.borrow();

    let mut encoded_entries = Vec::with_capacity(entries.len());
    for (key, value) in entries.iter() {
        let ValueData::String(key_text) = &key.data else {
            return Err("json: unsupported type: map with non-string keys".into());
        };
        encoded_entries.push((
            key_text.clone(),
            marshal_json_value(vm, program, value, seen_pointers, layout, depth + 1)?,
        ));
    }
    encoded_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let members = encoded_entries
        .into_iter()
        .map(|(key, value)| match layout {
            JsonLayout::Compact => format!("{}:{value}", quote_json_string(&key)),
            JsonLayout::Pretty { .. } => format!("{}: {value}", quote_json_string(&key)),
        })
        .collect::<Vec<_>>();
    render_collection('{', '}', &members, layout, depth)
}

fn marshal_json_struct(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    fields: &[(String, Value)],
    seen_pointers: &mut Vec<PointerTarget>,
    layout: JsonLayout<'_>,
    depth: usize,
) -> Result<String, JsonMarshalError> {
    let mut encoded_fields = Vec::new();
    for spec in
        json_struct_field_specs(program, vm, current, fields).map_err(JsonMarshalError::Message)?
    {
        let Some(value) = json_field_value_by_path(vm, program, current, &spec.path) else {
            continue;
        };
        if spec.omit_empty && json_value_is_empty(&value) {
            continue;
        }
        let rendered_value = if spec.quoted_kind.is_some() {
            quote_json_string(&marshal_json_value(
                vm,
                program,
                &value,
                seen_pointers,
                layout,
                depth + 1,
            )?)
        } else {
            marshal_json_value(vm, program, &value, seen_pointers, layout, depth + 1)?
        };
        encoded_fields.push(match layout {
            JsonLayout::Compact => format!("{}:{rendered_value}", quote_json_string(&spec.key)),
            JsonLayout::Pretty { .. } => {
                format!("{}: {rendered_value}", quote_json_string(&spec.key))
            }
        });
    }
    render_collection('{', '}', &encoded_fields, layout, depth)
}

fn render_collection(
    open: char,
    close: char,
    items: &[String],
    layout: JsonLayout<'_>,
    depth: usize,
) -> Result<String, JsonMarshalError> {
    if items.is_empty() {
        return Ok(format!("{open}{close}"));
    }
    match layout {
        JsonLayout::Compact => Ok(format!("{open}{}{close}", items.join(","))),
        JsonLayout::Pretty { prefix, indent } => {
            let child_indent = indent.repeat(depth + 1);
            let current_indent = indent.repeat(depth);
            let mut rendered = String::new();
            rendered.push(open);
            for (index, item) in items.iter().enumerate() {
                rendered.push('\n');
                rendered.push_str(prefix);
                rendered.push_str(&child_indent);
                rendered.push_str(item);
                if index + 1 != items.len() {
                    rendered.push(',');
                }
            }
            rendered.push('\n');
            rendered.push_str(prefix);
            rendered.push_str(&current_indent);
            rendered.push(close);
            Ok(rendered)
        }
    }
}

fn marshal_json_custom_value(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
) -> Result<Option<String>, JsonMarshalError> {
    if let Some(rendered) =
        invoke_custom_json_method(vm, program, value, "MarshalJSON", CustomMarshalKind::Json)?
    {
        return Ok(Some(rendered));
    }
    invoke_custom_json_method(vm, program, value, "MarshalText", CustomMarshalKind::Text)
}

fn invoke_custom_json_method(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    method: &str,
    kind: CustomMarshalKind,
) -> Result<Option<String>, JsonMarshalError> {
    let results = match vm.invoke_method_results(program, value.clone(), method, Vec::new()) {
        Ok(results) => results,
        Err(VmError::UnknownMethod { .. }) => return Ok(None),
        Err(error) => return Err(JsonMarshalError::Vm(error)),
    };
    if results.len() != 2 {
        return Err(format!(
            "json: error calling {method} for {}: expected ([]byte, error) result",
            describe_value(value)
        )
        .into());
    }

    if let ValueData::Error(error) = &results[1].data {
        return Err(format!(
            "json: error calling {method} for {}: {}",
            describe_value(value),
            error.message
        )
        .into());
    }
    if !matches!(&results[1].data, ValueData::Nil) {
        return Err(format!(
            "json: error calling {method} for {}: expected ([]byte, error) result",
            describe_value(value)
        )
        .into());
    }

    let bytes = byte_slice_value(vm, program, method, &results[0]).map_err(|_| {
        JsonMarshalError::Message(format!(
            "json: error calling {method} for {}: expected ([]byte, error) result",
            describe_value(value)
        ))
    })?;

    match kind {
        CustomMarshalKind::Json => {
            if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
                return Err(format!(
                    "json: error calling {method} for {}: returned invalid JSON",
                    describe_value(value)
                )
                .into());
            }
            let rendered = String::from_utf8(bytes).expect("validated JSON must be UTF-8");
            Ok(Some(rendered))
        }
        CustomMarshalKind::Text => {
            let text = String::from_utf8_lossy(&bytes);
            Ok(Some(quote_json_string(text.as_ref())))
        }
    }
}

fn quote_json_string(value: &str) -> String {
    serde_json::to_string(value)
        .expect("JSON string escaping should succeed")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

fn bytes_to_value(data: &[u8]) -> Value {
    Value::slice(data.iter().map(|&b| Value::int(b as i64)).collect())
}

fn byte_slice_arg(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<Vec<u8>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
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
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a []byte argument".into(),
            }),
        })
        .collect()
}

fn byte_slice_value(
    vm: &Vm,
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
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "a []byte argument".into(),
            }),
        })
        .collect()
}
