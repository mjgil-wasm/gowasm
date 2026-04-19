#[path = "json_decode_interface.rs"]
mod interface_decode;

use super::json_tags_impl::{
    json_struct_field_specs, json_struct_top_level_field_specs, JsonQuotedFieldKind,
};
use crate::{
    explicit_concrete_type_for_value, program_type_inventory, value_runtime_type, ArrayValue,
    ConcreteType, MapValue, PointerTarget, PointerValue, Program, RuntimeTypeInfo, RuntimeTypeKind,
    SliceValue, Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_BOOL, TYPE_FLOAT64, TYPE_INT,
    TYPE_POINTER, TYPE_STRING,
};
use interface_decode::{decode_json_interface, target_is_interface};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

thread_local! {
    static JSON_PARTIAL_DECODE: RefCell<Option<Value>> = const { RefCell::new(None) };
}

fn clear_partial_decode() {
    JSON_PARTIAL_DECODE.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

fn set_partial_decode(value: Value) {
    JSON_PARTIAL_DECODE.with(|slot| {
        *slot.borrow_mut() = Some(value);
    });
}

fn take_partial_decode() -> Option<Value> {
    JSON_PARTIAL_DECODE.with(|slot| slot.borrow_mut().take())
}

pub(super) fn json_unmarshal(
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

    let bytes = byte_slice_arg(vm, program, "json.Unmarshal", &args[0])?;
    let parsed = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(parsed) => parsed,
        Err(_) => return Ok(Value::error("json: invalid JSON")),
    };

    let ValueData::Pointer(pointer) = &args[1].data else {
        return Ok(Value::error(
            "json: Unmarshal expects a non-nil pointer target",
        ));
    };
    if pointer.is_nil() {
        return Ok(Value::error(
            "json: Unmarshal expects a non-nil pointer target",
        ));
    }

    clear_partial_decode();
    let current = vm.deref_pointer(program, &args[1])?;
    let decoded = match decode_json_value(vm, program, &current, &parsed) {
        Ok(decoded) => decoded,
        Err(error) => {
            if let Some(partial) = take_partial_decode() {
                vm.store_indirect(program, &args[1], partial)?;
            }
            return Ok(Value::error(error));
        }
    };
    vm.store_indirect(program, &args[1], decoded)?;
    Ok(Value::nil())
}

fn decode_json_value(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    if let ValueData::Pointer(pointer) = &current.data {
        return decode_json_pointer(vm, program, current, pointer, parsed);
    }

    if target_is_interface(program, current) {
        return decode_json_interface(current, parsed);
    }

    if parsed.is_null() {
        return Ok(match &current.data {
            ValueData::Struct(_) | ValueData::Array(_) => current.clone(),
            ValueData::Int(_) | ValueData::Float(_) | ValueData::String(_) | ValueData::Bool(_) => {
                current.clone()
            }
            ValueData::Slice(slice) => Value {
                typ: current.typ,
                data: ValueData::Slice(SliceValue {
                    values: Rc::new(RefCell::new(Vec::new())),
                    start: 0,
                    len: 0,
                    cap: 0,
                    is_nil: true,
                    concrete_type: slice.concrete_type.clone(),
                }),
            },
            ValueData::Map(map) => Value {
                typ: current.typ,
                data: ValueData::Map(MapValue::nil(
                    (*map.zero_value).clone(),
                    map.concrete_type.clone(),
                )),
            },
            _ => return Err(unsupported_target(current)),
        });
    }

    match &current.data {
        ValueData::Int(_) => decode_json_int(current, parsed),
        ValueData::Float(_) => decode_json_float(current, parsed),
        ValueData::String(_) => decode_json_string(current, parsed),
        ValueData::Bool(_) => decode_json_bool(current, parsed),
        ValueData::Struct(fields) => decode_json_struct(vm, program, current, fields, parsed),
        ValueData::Array(array) => decode_json_array(vm, program, current, array, parsed),
        ValueData::Slice(slice) => decode_json_slice(vm, program, current, slice, parsed),
        ValueData::Map(map) => decode_json_map(vm, program, current, map, parsed),
        _ => Err(unsupported_target(current)),
    }
}

fn decode_json_int(current: &Value, parsed: &serde_json::Value) -> Result<Value, String> {
    let serde_json::Value::Number(number) = parsed else {
        return Err(type_mismatch(parsed, "int"));
    };
    let value = parse_json_int(number).map_err(|_| number_mismatch(number, "int"))?;
    Ok(Value {
        typ: current.typ,
        data: ValueData::Int(value),
    })
}

fn decode_json_float(current: &Value, parsed: &serde_json::Value) -> Result<Value, String> {
    let serde_json::Value::Number(number) = parsed else {
        return Err(type_mismatch(parsed, "float64"));
    };
    let value = parse_json_float(number).map_err(|_| number_mismatch(number, "float64"))?;
    Ok(Value {
        typ: current.typ,
        data: ValueData::Float(crate::Float64(value)),
    })
}

fn decode_json_string(current: &Value, parsed: &serde_json::Value) -> Result<Value, String> {
    let serde_json::Value::String(text) = parsed else {
        return Err(type_mismatch(parsed, "string"));
    };
    Ok(Value {
        typ: current.typ,
        data: ValueData::String(text.clone()),
    })
}

fn decode_json_bool(current: &Value, parsed: &serde_json::Value) -> Result<Value, String> {
    let serde_json::Value::Bool(boolean) = parsed else {
        return Err(type_mismatch(parsed, "bool"));
    };
    Ok(Value {
        typ: current.typ,
        data: ValueData::Bool(*boolean),
    })
}

fn decode_json_struct(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    fields: &[(String, Value)],
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    let serde_json::Value::Object(object) = parsed else {
        return Err(type_mismatch(parsed, "struct"));
    };

    let mut updated_fields = fields.to_vec();
    let mut handled_keys = HashSet::new();
    for spec in json_struct_top_level_field_specs(program, vm, current, fields)? {
        let Some(raw_member) = struct_object_member(object, spec.key.as_str()) else {
            continue;
        };
        let decoded_string_member;
        let member = if let Some(kind) = spec.quoted_kind {
            decoded_string_member =
                Some(decode_string_tag_member(kind, raw_member).map_err(|error| {
                    struct_field_error(vm, program, current, spec.key.as_str(), error)
                })?);
            decoded_string_member
                .as_ref()
                .expect("decoded string-tag member should exist")
        } else {
            raw_member
        };
        let struct_value = Value {
            typ: current.typ,
            data: ValueData::Struct(updated_fields),
        };
        let ValueData::Struct(fields) =
            decode_json_field_path(vm, program, &struct_value, &spec.path, member)
                .map_err(|error| {
                    struct_field_error(vm, program, current, spec.key.as_str(), error)
                })?
                .data
        else {
            unreachable!("decoding through a struct field path should preserve the struct shape");
        };
        handled_keys.insert(spec.key);
        updated_fields = fields;
    }

    for spec in json_struct_field_specs(program, vm, current, fields)? {
        if handled_keys.contains(&spec.key) {
            continue;
        }
        let Some(raw_member) = struct_object_member(object, spec.key.as_str()) else {
            continue;
        };
        let decoded_string_member;
        let member = if let Some(kind) = spec.quoted_kind {
            decoded_string_member =
                Some(decode_string_tag_member(kind, raw_member).map_err(|error| {
                    struct_field_error(vm, program, current, spec.key.as_str(), error)
                })?);
            decoded_string_member
                .as_ref()
                .expect("decoded string-tag member should exist")
        } else {
            raw_member
        };
        let struct_value = Value {
            typ: current.typ,
            data: ValueData::Struct(updated_fields),
        };
        let ValueData::Struct(fields) =
            decode_json_field_path(vm, program, &struct_value, &spec.path, member)
                .map_err(|error| {
                    struct_field_error(vm, program, current, spec.key.as_str(), error)
                })?
                .data
        else {
            unreachable!("decoding through a struct field path should preserve the struct shape");
        };
        updated_fields = fields;
    }

    Ok(Value {
        typ: current.typ,
        data: ValueData::Struct(updated_fields),
    })
}

fn decode_string_tag_member(
    kind: JsonQuotedFieldKind,
    member: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match member {
        serde_json::Value::Null => Ok(serde_json::Value::Null),
        serde_json::Value::String(text) => {
            let decoded = serde_json::from_str::<serde_json::Value>(text)
                .map_err(|_| string_tag_decode_error(text, kind))?;
            validate_string_tag_value(kind, &decoded, text)?;
            Ok(decoded)
        }
        _ => Err(format!(
            "json: invalid use of ,string struct tag, trying to unmarshal unquoted value into {}",
            kind.describe()
        )),
    }
}

fn validate_string_tag_value(
    kind: JsonQuotedFieldKind,
    decoded: &serde_json::Value,
    original: &str,
) -> Result<(), String> {
    match decoded {
        serde_json::Value::Null => Ok(()),
        serde_json::Value::Bool(_) if kind == JsonQuotedFieldKind::Bool => Ok(()),
        serde_json::Value::Number(_) if kind == JsonQuotedFieldKind::Int => Ok(()),
        serde_json::Value::Number(_) if kind == JsonQuotedFieldKind::Float64 => Ok(()),
        serde_json::Value::String(_) if kind == JsonQuotedFieldKind::String => Ok(()),
        _ => Err(invalid_string_tag_value(original, kind)),
    }
}

fn invalid_string_tag_value(original: &str, kind: JsonQuotedFieldKind) -> String {
    format!(
        "json: invalid use of ,string struct tag, trying to unmarshal {} into {}",
        quoted_json_string(original),
        kind.describe()
    )
}

fn string_tag_decode_error(original: &str, kind: JsonQuotedFieldKind) -> String {
    if kind == JsonQuotedFieldKind::Float64 {
        if let Ok(value) = original.parse::<f64>() {
            if !value.is_finite() {
                return number_text_mismatch(original, "float64");
            }
        }
    }
    invalid_string_tag_value(original, kind)
}

fn quoted_json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a Rust string into JSON should succeed")
}

fn should_reuse_sequence_element(value: &Value) -> bool {
    matches!(
        value.data,
        ValueData::Pointer(_)
            | ValueData::Struct(_)
            | ValueData::Array(_)
            | ValueData::Slice(_)
            | ValueData::Map(_)
    )
}

fn parse_json_int(number: &serde_json::Number) -> Result<i64, ()> {
    number.to_string().parse::<i64>().map_err(|_| ())
}

fn parse_json_float(number: &serde_json::Number) -> Result<f64, ()> {
    number
        .to_string()
        .parse::<f64>()
        .map_err(|_| ())
        .and_then(|value| {
            if value.is_finite() {
                Ok(value)
            } else {
                Err(())
            }
        })
}

fn struct_object_member<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    if let Some(member) = object.get(key) {
        return Some(member);
    }
    object
        .iter()
        .find(|(member_name, _)| json_field_key_matches(member_name, key))
        .map(|(_, member)| member)
}

fn json_field_key_matches(member_name: &str, key: &str) -> bool {
    member_name == key
        || member_name
            .chars()
            .flat_map(char::to_lowercase)
            .eq(key.chars().flat_map(char::to_lowercase))
}

fn decode_json_field_path(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    path: &[usize],
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    if path.is_empty() {
        return decode_json_value(vm, program, current, parsed);
    }

    match &current.data {
        ValueData::Struct(fields) => {
            let Some((field_name, field_value)) = fields.get(path[0]).cloned() else {
                return Err(unsupported_target(current));
            };
            let updated_field =
                match decode_json_field_path(vm, program, &field_value, &path[1..], parsed) {
                    Ok(updated_field) => updated_field,
                    Err(error) => {
                        if let Some(partial_field) = take_partial_decode() {
                            let mut partial_fields = fields.clone();
                            partial_fields[path[0]] = (field_name.clone(), partial_field);
                            set_partial_decode(Value {
                                typ: current.typ,
                                data: ValueData::Struct(partial_fields),
                            });
                        }
                        return Err(error);
                    }
                };
            let mut updated_fields = fields.clone();
            updated_fields[path[0]] = (field_name, updated_field);
            Ok(Value {
                typ: current.typ,
                data: ValueData::Struct(updated_fields),
            })
        }
        ValueData::Pointer(pointer) => {
            if pointer.is_nil() {
                let element_type = value_element_type(vm, program, current)?;
                let template = zero_value_for_concrete_type(program, &element_type)?;
                match decode_json_field_path(vm, program, &template, path, parsed) {
                    Ok(updated) => Ok(box_pointer_like(vm, current, updated)),
                    Err(error) => {
                        if let Some(partial_inner) = take_partial_decode() {
                            set_partial_decode(box_pointer_like(vm, current, partial_inner));
                        }
                        Err(error)
                    }
                }
            } else {
                let inner = vm
                    .deref_pointer(program, current)
                    .map_err(|_| unsupported_target(current))?;
                match decode_json_field_path(vm, program, &inner, path, parsed) {
                    Ok(updated) => {
                        vm.store_indirect(program, current, updated)
                            .map_err(|_| unsupported_target(current))?;
                        Ok(current.clone())
                    }
                    Err(error) => {
                        if let Some(partial_inner) = take_partial_decode() {
                            vm.store_indirect(program, current, partial_inner)
                                .map_err(|_| unsupported_target(current))?;
                            set_partial_decode(current.clone());
                        }
                        Err(error)
                    }
                }
            }
        }
        _ => Err(unsupported_target(current)),
    }
}

fn decode_json_array(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    array: &ArrayValue,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    let serde_json::Value::Array(items) = parsed else {
        return Err(type_mismatch(parsed, "array"));
    };
    if items.len() > array.len() {
        return Err(format!(
            "json: cannot unmarshal array of length {} into array target of length {}",
            items.len(),
            array.len()
        ));
    }

    let element_type = value_element_type(vm, program, current)?;
    let current_values = array.values_snapshot();
    let mut decoded = (0..array.len())
        .map(|_| zero_value_for_concrete_type(program, &element_type))
        .collect::<Result<Vec<_>, String>>()?;
    for (index, item) in items.iter().enumerate() {
        let zero_template = zero_value_for_concrete_type(program, &element_type)?;
        let template = current_values
            .get(index)
            .filter(|value| should_reuse_sequence_element(value))
            .cloned()
            .unwrap_or(zero_template);
        match decode_json_value(vm, program, &template, item) {
            Ok(value) => decoded[index] = value,
            Err(error) => {
                set_partial_decode(Value {
                    typ: current.typ,
                    data: ValueData::Array(ArrayValue {
                        values: Rc::new(RefCell::new(decoded)),
                        concrete_type: array.concrete_type.clone(),
                    }),
                });
                return Err(error);
            }
        }
    }

    Ok(Value {
        typ: current.typ,
        data: ValueData::Array(ArrayValue {
            values: Rc::new(RefCell::new(decoded)),
            concrete_type: array.concrete_type.clone(),
        }),
    })
}

fn decode_json_slice(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    slice: &SliceValue,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    let serde_json::Value::Array(items) = parsed else {
        return Err(type_mismatch(parsed, "slice"));
    };

    let element_type = value_element_type(vm, program, current)?;
    let current_values = slice.values_snapshot();
    let mut decoded = (0..items.len())
        .map(|_| zero_value_for_concrete_type(program, &element_type))
        .collect::<Result<Vec<_>, String>>()?;
    for (index, item) in items.iter().enumerate() {
        let zero_template = zero_value_for_concrete_type(program, &element_type)?;
        let template = current_values
            .get(index)
            .filter(|value| should_reuse_sequence_element(value))
            .cloned()
            .unwrap_or(zero_template);
        match decode_json_value(vm, program, &template, item) {
            Ok(value) => decoded[index] = value,
            Err(error) => {
                set_partial_decode(Value {
                    typ: current.typ,
                    data: ValueData::Slice(SliceValue {
                        cap: decoded.len(),
                        len: decoded.len(),
                        values: Rc::new(RefCell::new(decoded)),
                        start: 0,
                        is_nil: false,
                        concrete_type: slice.concrete_type.clone(),
                    }),
                });
                return Err(error);
            }
        }
    }

    Ok(Value {
        typ: current.typ,
        data: ValueData::Slice(SliceValue {
            cap: decoded.len(),
            len: decoded.len(),
            values: Rc::new(RefCell::new(decoded)),
            start: 0,
            is_nil: false,
            concrete_type: slice.concrete_type.clone(),
        }),
    })
}

fn decode_json_map(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    map: &MapValue,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    let serde_json::Value::Object(object) = parsed else {
        return Err(type_mismatch(parsed, "map"));
    };

    let (key_type, value_type) = map_key_and_value_types(vm, program, current)?;
    let mut decoded = map.entries_snapshot();
    for (member_name, member_value) in object {
        let key = decode_json_map_key(program, &key_type, member_name)?;
        let template = zero_value_for_concrete_type(program, &value_type)?;
        let value = decode_json_value(vm, program, &template, member_value)?;
        if let Some((_, existing_value)) = decoded
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            *existing_value = value;
        } else {
            decoded.push((key, value));
        }
    }

    Ok(Value {
        typ: current.typ,
        data: ValueData::Map(MapValue::with_entries(
            decoded,
            (*map.zero_value).clone(),
            map.concrete_type.clone(),
        )),
    })
}

fn decode_json_pointer(
    vm: &mut Vm,
    program: &Program,
    current: &Value,
    pointer: &PointerValue,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    if parsed.is_null() {
        return Ok(nil_pointer_like(current));
    }

    if pointer.is_nil() {
        let element_type = value_element_type(vm, program, current)?;
        let template = zero_value_for_concrete_type(program, &element_type)?;
        return match decode_json_value(vm, program, &template, parsed) {
            Ok(decoded) => Ok(box_pointer_like(vm, current, decoded)),
            Err(error) => {
                if let Some(partial_inner) = take_partial_decode() {
                    set_partial_decode(box_pointer_like(vm, current, partial_inner));
                }
                Err(error)
            }
        };
    }

    let inner = vm
        .deref_pointer(program, current)
        .map_err(|_| unsupported_target(current))?;
    match decode_json_value(vm, program, &inner, parsed) {
        Ok(decoded) => {
            vm.store_indirect(program, current, decoded)
                .map_err(|_| unsupported_target(current))?;
            Ok(current.clone())
        }
        Err(error) => {
            if let Some(partial_inner) = take_partial_decode() {
                vm.store_indirect(program, current, partial_inner)
                    .map_err(|_| unsupported_target(current))?;
                set_partial_decode(current.clone());
            }
            Err(error)
        }
    }
}

fn value_element_type(vm: &Vm, program: &Program, current: &Value) -> Result<ConcreteType, String> {
    value_runtime_type(program, vm, current)
        .and_then(|info| info.elem.map(|element| *element))
        .ok_or_else(|| unsupported_target(current))
}

fn map_key_and_value_types(
    vm: &Vm,
    program: &Program,
    current: &Value,
) -> Result<(ConcreteType, ConcreteType), String> {
    let Some(info) = value_runtime_type(program, vm, current) else {
        return Err(unsupported_target(current));
    };
    let Some(key) = info.key.map(|key| *key) else {
        return Err(unsupported_target(current));
    };
    let Some(value) = info.elem.map(|value| *value) else {
        return Err(unsupported_target(current));
    };
    Ok((key, value))
}

fn decode_json_map_key(
    program: &Program,
    key_type: &ConcreteType,
    member_name: &str,
) -> Result<Value, String> {
    let info = concrete_type_info(program, key_type).ok_or_else(|| {
        "json: unsupported unmarshal target: map with non-string keys".to_string()
    })?;
    if info.kind != RuntimeTypeKind::String {
        return Err("json: unsupported unmarshal target: map with non-string keys".into());
    }
    Ok(Value {
        typ: info.type_id.unwrap_or(TYPE_STRING),
        data: ValueData::String(member_name.to_string()),
    })
}

fn concrete_type_info(program: &Program, typ: &ConcreteType) -> Option<RuntimeTypeInfo> {
    program_type_inventory(program)?.resolve_concrete_type(typ)
}

fn nil_pointer_like(current: &Value) -> Value {
    let mut value = current.clone();
    let ValueData::Pointer(pointer) = &mut value.data else {
        unreachable!("nil_pointer_like expects a pointer value");
    };
    pointer.target = PointerTarget::Nil;
    value
}

fn box_pointer_like(vm: &mut Vm, current: &Value, pointee: Value) -> Value {
    let mut pointer = vm.box_heap_value(pointee, current.typ);
    let ValueData::Pointer(pointer_value) = &mut pointer.data else {
        unreachable!("box_heap_value should always return a pointer");
    };
    pointer_value.concrete_type = explicit_concrete_type_for_value(current);
    pointer
}

fn zero_value_for_concrete_type(program: &Program, typ: &ConcreteType) -> Result<Value, String> {
    match typ {
        ConcreteType::TypeId(type_id) => {
            let info = program_type_inventory(program)
                .and_then(|inventory| inventory.type_info_for_type_id(*type_id))
                .ok_or_else(|| {
                    format!("json: unsupported unmarshal target metadata for type `{type_id:?}`")
                })?;
            zero_value_for_type_info(program, &info)
        }
        ConcreteType::Array { len, element } => {
            let zero = zero_value_for_concrete_type(program, element)?;
            Ok(Value::array_typed(vec![zero; *len], typ.clone()))
        }
        ConcreteType::Slice { .. } => Ok(Value::nil_slice_typed(typ.clone())),
        ConcreteType::Map { value, .. } => {
            let zero = zero_value_for_concrete_type(program, value)?;
            Ok(Value::nil_map_typed(zero, typ.clone()))
        }
        ConcreteType::Pointer { .. } => Ok(Value::nil_pointer_typed(TYPE_POINTER, typ.clone())),
        _ => Err(format!(
            "json: unsupported unmarshal target metadata for `{typ:?}`"
        )),
    }
}

fn zero_value_for_type_info(program: &Program, info: &RuntimeTypeInfo) -> Result<Value, String> {
    match info.kind {
        RuntimeTypeKind::Int => Ok(Value {
            typ: info.type_id.unwrap_or(TYPE_INT),
            data: ValueData::Int(0),
        }),
        RuntimeTypeKind::Float64 => Ok(Value {
            typ: info.type_id.unwrap_or(TYPE_FLOAT64),
            data: ValueData::Float(crate::Float64(0.0)),
        }),
        RuntimeTypeKind::String => Ok(Value {
            typ: info.type_id.unwrap_or(TYPE_STRING),
            data: ValueData::String(String::new()),
        }),
        RuntimeTypeKind::Bool => Ok(Value {
            typ: info.type_id.unwrap_or(TYPE_BOOL),
            data: ValueData::Bool(false),
        }),
        RuntimeTypeKind::Interface => Ok(Value {
            typ: info.type_id.unwrap_or(TYPE_ANY),
            data: ValueData::Nil,
        }),
        RuntimeTypeKind::Struct => Ok(Value {
            typ: info
                .type_id
                .expect("named struct zero value should have a type id"),
            data: ValueData::Struct(
                info.fields
                    .iter()
                    .map(|field| {
                        Ok((
                            field.name.clone(),
                            zero_value_for_concrete_type(program, &field.typ)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            ),
        }),
        RuntimeTypeKind::Array => {
            let len = info
                .len
                .ok_or_else(|| "json: runtime array metadata was incomplete".to_string())?;
            let element = info
                .elem
                .as_deref()
                .ok_or_else(|| "json: runtime array metadata was incomplete".to_string())?;
            let zero = zero_value_for_concrete_type(program, element)?;
            let mut value = Value::array_typed(
                vec![zero; len],
                ConcreteType::TypeId(
                    info.type_id
                        .expect("named array zero value should have a type id"),
                ),
            );
            value.typ = info
                .type_id
                .expect("named array zero value should have a type id");
            Ok(value)
        }
        RuntimeTypeKind::Slice => {
            let type_id = info
                .type_id
                .expect("named slice zero value should have a type id");
            let mut value = Value::nil_slice_typed(ConcreteType::TypeId(type_id));
            value.typ = type_id;
            Ok(value)
        }
        RuntimeTypeKind::Map => {
            let type_id = info
                .type_id
                .expect("named map zero value should have a type id");
            let value_type = info
                .elem
                .as_deref()
                .ok_or_else(|| "json: runtime map metadata was incomplete".to_string())?;
            let zero = zero_value_for_concrete_type(program, value_type)?;
            let mut value = Value::nil_map_typed(zero, ConcreteType::TypeId(type_id));
            value.typ = type_id;
            Ok(value)
        }
        RuntimeTypeKind::Pointer => {
            let type_id = info
                .type_id
                .expect("named pointer zero value should have a type id");
            Ok(Value::nil_pointer_typed(
                type_id,
                ConcreteType::TypeId(type_id),
            ))
        }
        _ => Err(format!(
            "json: unsupported unmarshal target metadata for `{}`",
            info.display_name
        )),
    }
}

fn unsupported_target(current: &Value) -> String {
    format!(
        "json: unsupported unmarshal target: {}",
        target_kind(current)
    )
}

fn type_mismatch(parsed: &serde_json::Value, target: &str) -> String {
    format!(
        "json: cannot unmarshal {} into {} target",
        json_kind(parsed),
        target
    )
}

fn number_mismatch(number: &serde_json::Number, target: &str) -> String {
    number_text_mismatch(number.to_string().as_str(), target)
}

fn number_text_mismatch(number: &str, target: &str) -> String {
    format!("json: cannot unmarshal number {number} into {target} target")
}

fn struct_field_error(
    vm: &Vm,
    program: &Program,
    current: &Value,
    field_key: &str,
    error: String,
) -> String {
    let Some(rest) = error.strip_prefix("json: cannot unmarshal ") else {
        return error;
    };
    let Some((source, target)) = rest.split_once(" into ") else {
        return error;
    };
    let Some(target) = target.strip_suffix(" target") else {
        return error;
    };
    let struct_name = value_runtime_type(program, vm, current)
        .map(|info| info.display_name)
        .unwrap_or_else(|| "struct".to_string());
    format!(
        "json: cannot unmarshal {source} into Go struct field {struct_name}.{field_key} of type {target}"
    )
}

fn json_kind(parsed: &serde_json::Value) -> &'static str {
    match parsed {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn target_kind(current: &Value) -> &'static str {
    match &current.data {
        ValueData::Nil => "nil",
        ValueData::Int(_) => "int",
        ValueData::Float(_) => "float64",
        ValueData::String(_) => "string",
        ValueData::Bool(_) => "bool",
        ValueData::Error(_) => "error",
        ValueData::Array(_) => "array",
        ValueData::Slice(_) => "slice",
        ValueData::Map(_) => "map",
        ValueData::Channel(_) => "channel",
        ValueData::Pointer(pointer) => {
            if pointer.is_nil() {
                "nil pointer"
            } else {
                "pointer"
            }
        }
        ValueData::Function(_) => "function",
        ValueData::Struct(_) => "struct",
    }
}

fn byte_slice_arg(
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
