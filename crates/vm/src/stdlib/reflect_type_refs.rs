use super::*;

pub(super) const TYPE_REF_FIELD_PKG_PATH: &str = "__refFieldPkgPath";

const TYPE_REF_KIND_FIELD: &str = "__refKind";
const TYPE_REF_TYPE_ID_FIELD: &str = "__typeId";
const TYPE_REF_LEN_FIELD: &str = "__refLen";
const TYPE_REF_ELEM_FIELD: &str = "__refElem";
const TYPE_REF_KEY_FIELD: &str = "__refKey";
const TYPE_REF_VALUE_FIELD: &str = "__refValue";
const TYPE_REF_PARAMS_FIELD: &str = "__refParams";
const TYPE_REF_RESULTS_FIELD: &str = "__refResults";
const TYPE_REF_CHANNEL_DIR_FIELD: &str = "__refChannelDir";
const TYPE_REF_FIELD_NAME: &str = "__refFieldName";
const TYPE_REF_FIELD_EMBEDDED: &str = "__refFieldEmbedded";
const TYPE_REF_FIELD_TAG: &str = "__refFieldTag";

pub(super) fn encode_optional_concrete_type_ref(typ: Option<&ConcreteType>) -> Value {
    typ.map_or_else(Value::nil, encode_concrete_type_ref)
}

pub(super) fn encode_concrete_type_refs(types: &[ConcreteType]) -> Vec<Value> {
    types.iter().map(encode_concrete_type_ref).collect()
}

pub(super) fn reflect_struct_fields(info: &RuntimeTypeInfo) -> Vec<Value> {
    info.fields
        .iter()
        .map(|field| reflect_struct_field_ref_value(info, field))
        .collect()
}

pub(super) fn reflect_type_signature_item(
    vm: &Vm,
    program: &Program,
    receiver: &Value,
    field: &str,
    index: usize,
    method: &str,
) -> Result<Value, VmError> {
    let items = hidden_slice(receiver, field)?;
    let Some(item) = items.get(index) else {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} index out of range"),
        ));
    };
    let inventory = reflect_inventory(vm, program)?;
    resolve_concrete_type_ref_value(item, &inventory, vm, program)
}

pub(super) fn reflect_struct_field_type(
    vm: &Vm,
    program: &Program,
    field_desc: &Value,
) -> Result<Value, VmError> {
    let inventory = reflect_inventory(vm, program)?;
    let type_ref = hidden_field(field_desc, TYPE_REF_VALUE_FIELD)?;
    resolve_concrete_type_ref_value(&type_ref, &inventory, vm, program)
}

pub(super) fn materialize_reflect_struct_field(
    vm: &Vm,
    program: &Program,
    field_desc: &Value,
) -> Result<Value, VmError> {
    Ok(Value::struct_value(
        TYPE_REFLECT_STRUCT_FIELD,
        vec![
            (
                "Name".into(),
                hidden_field(field_desc, TYPE_REF_FIELD_NAME)?,
            ),
            (
                "PkgPath".into(),
                hidden_field(field_desc, TYPE_REF_FIELD_PKG_PATH)?,
            ),
            (
                "Type".into(),
                reflect_struct_field_type(vm, program, field_desc)?,
            ),
            (
                "Tag".into(),
                Value {
                    typ: crate::TYPE_REFLECT_STRUCT_TAG,
                    data: hidden_field(field_desc, TYPE_REF_FIELD_TAG)?.data,
                },
            ),
            (
                "Anonymous".into(),
                hidden_field(field_desc, TYPE_REF_FIELD_EMBEDDED)?,
            ),
        ],
    ))
}

fn encode_concrete_type_ref(typ: &ConcreteType) -> Value {
    match typ {
        ConcreteType::TypeId(type_id) => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("typeId")),
                (TYPE_REF_TYPE_ID_FIELD.into(), Value::int(type_id.0 as i64)),
            ],
        ),
        ConcreteType::Array { len, element } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("array")),
                (TYPE_REF_LEN_FIELD.into(), Value::int(*len as i64)),
                (
                    TYPE_REF_ELEM_FIELD.into(),
                    encode_concrete_type_ref(element.as_ref()),
                ),
            ],
        ),
        ConcreteType::Slice { element } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("slice")),
                (
                    TYPE_REF_ELEM_FIELD.into(),
                    encode_concrete_type_ref(element.as_ref()),
                ),
            ],
        ),
        ConcreteType::Map { key, value } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("map")),
                (
                    TYPE_REF_KEY_FIELD.into(),
                    encode_concrete_type_ref(key.as_ref()),
                ),
                (
                    TYPE_REF_VALUE_FIELD.into(),
                    encode_concrete_type_ref(value.as_ref()),
                ),
            ],
        ),
        ConcreteType::Pointer { element } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("pointer")),
                (
                    TYPE_REF_ELEM_FIELD.into(),
                    encode_concrete_type_ref(element.as_ref()),
                ),
            ],
        ),
        ConcreteType::Function { params, results } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("function")),
                (
                    TYPE_REF_PARAMS_FIELD.into(),
                    Value::slice(encode_concrete_type_refs(params)),
                ),
                (
                    TYPE_REF_RESULTS_FIELD.into(),
                    Value::slice(encode_concrete_type_refs(results)),
                ),
            ],
        ),
        ConcreteType::Channel { direction, element } => Value::struct_value(
            TYPE_EMPTY_STRUCT,
            vec![
                (TYPE_REF_KIND_FIELD.into(), Value::string("channel")),
                (
                    TYPE_REF_CHANNEL_DIR_FIELD.into(),
                    Value::int(match direction {
                        RuntimeChannelDirection::Bidirectional => 0,
                        RuntimeChannelDirection::SendOnly => 1,
                        RuntimeChannelDirection::ReceiveOnly => 2,
                    }),
                ),
                (
                    TYPE_REF_ELEM_FIELD.into(),
                    encode_concrete_type_ref(element.as_ref()),
                ),
            ],
        ),
    }
}

fn reflect_struct_field_ref_value(owner: &RuntimeTypeInfo, field: &RuntimeTypeField) -> Value {
    Value::struct_value(
        TYPE_EMPTY_STRUCT,
        vec![
            (
                TYPE_REF_FIELD_NAME.into(),
                Value::string(field.name.clone()),
            ),
            (
                TYPE_REF_FIELD_PKG_PATH.into(),
                Value::string(struct_field_pkg_path(owner, field)),
            ),
            (
                TYPE_REF_VALUE_FIELD.into(),
                encode_concrete_type_ref(&field.typ),
            ),
            (
                TYPE_REF_FIELD_TAG.into(),
                Value::string(field.tag.clone().unwrap_or_default()),
            ),
            (TYPE_REF_FIELD_EMBEDDED.into(), Value::bool(field.embedded)),
        ],
    )
}

pub(super) fn resolve_concrete_type_ref_value(
    value: &Value,
    inventory: &ProgramTypeInventory,
    vm: &Vm,
    program: &Program,
) -> Result<Value, VmError> {
    if value.typ == TYPE_REFLECT_RTYPE {
        return Ok(value.clone());
    }
    if matches!(&value.data, ValueData::Nil) {
        return Ok(Value::nil());
    }
    let concrete = decode_concrete_type_ref(value)?;
    let info = inventory
        .resolve_concrete_type(&concrete)
        .ok_or_else(|| reflect_panic(vm, program, "missing concrete type metadata"))?;
    build_reflect_type_value(&info, inventory, vm, program)
}

fn decode_concrete_type_ref(value: &Value) -> Result<ConcreteType, VmError> {
    let kind = hidden_string(value, TYPE_REF_KIND_FIELD)?;
    match kind.as_str() {
        "typeId" => Ok(ConcreteType::TypeId(TypeId(
            positive_ref_int(value, TYPE_REF_TYPE_ID_FIELD)? as u32,
        ))),
        "array" => Ok(ConcreteType::Array {
            len: positive_ref_int(value, TYPE_REF_LEN_FIELD)? as usize,
            element: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_ELEM_FIELD,
            )?)?),
        }),
        "slice" => Ok(ConcreteType::Slice {
            element: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_ELEM_FIELD,
            )?)?),
        }),
        "map" => Ok(ConcreteType::Map {
            key: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_KEY_FIELD,
            )?)?),
            value: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_VALUE_FIELD,
            )?)?),
        }),
        "pointer" => Ok(ConcreteType::Pointer {
            element: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_ELEM_FIELD,
            )?)?),
        }),
        "function" => Ok(ConcreteType::Function {
            params: decode_concrete_type_ref_slice(&hidden_slice(value, TYPE_REF_PARAMS_FIELD)?)?,
            results: decode_concrete_type_ref_slice(&hidden_slice(value, TYPE_REF_RESULTS_FIELD)?)?,
        }),
        "channel" => Ok(ConcreteType::Channel {
            direction: decode_channel_direction(value)?,
            element: Box::new(decode_concrete_type_ref(&hidden_field(
                value,
                TYPE_REF_ELEM_FIELD,
            )?)?),
        }),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "encoded reflect type reference".into(),
        }),
    }
}

fn decode_concrete_type_ref_slice(items: &[Value]) -> Result<Vec<ConcreteType>, VmError> {
    items.iter().map(decode_concrete_type_ref).collect()
}

fn decode_channel_direction(value: &Value) -> Result<RuntimeChannelDirection, VmError> {
    match hidden_field(value, TYPE_REF_CHANNEL_DIR_FIELD)?.data {
        ValueData::Int(0) => Ok(RuntimeChannelDirection::Bidirectional),
        ValueData::Int(1) => Ok(RuntimeChannelDirection::SendOnly),
        ValueData::Int(2) => Ok(RuntimeChannelDirection::ReceiveOnly),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "encoded channel direction".into(),
        }),
    }
}

fn positive_ref_int(value: &Value, field: &str) -> Result<i64, VmError> {
    match hidden_field(value, field)?.data {
        ValueData::Int(number) if number >= 0 => Ok(number),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "non-negative integer".into(),
        }),
    }
}
