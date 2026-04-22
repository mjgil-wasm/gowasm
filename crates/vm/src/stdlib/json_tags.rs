use std::collections::HashMap;

use crate::{
    program_type_inventory, value_runtime_type, Program, ProgramTypeInventory, RuntimeTypeField,
    RuntimeTypeInfo, RuntimeTypeKind, Value, ValueData, Vm,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct JsonStructFieldSpec {
    pub(super) path: Vec<usize>,
    pub(super) key: String,
    pub(super) omit_empty: bool,
    pub(super) quoted_kind: Option<JsonQuotedFieldKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonStructFieldCandidate {
    path: Vec<usize>,
    key: String,
    omit_empty: bool,
    quoted_kind: Option<JsonQuotedFieldKind>,
    tagged: bool,
    depth: usize,
    order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonTagInfo {
    key: String,
    omit_empty: bool,
    quoted_kind: Option<JsonQuotedFieldKind>,
    tagged: bool,
    explicit_name: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JsonQuotedFieldKind {
    Bool,
    Int,
    Float64,
    String,
}

impl JsonQuotedFieldKind {
    pub(super) fn describe(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float64 => "float64",
            Self::String => "string",
        }
    }

    fn for_runtime_field(
        field: &RuntimeTypeField,
        inventory: &ProgramTypeInventory,
    ) -> Option<Self> {
        let info = inventory.resolve_concrete_type(&field.typ)?;
        Self::for_runtime_kind(&info.kind)
    }

    fn for_value(value: &Value) -> Option<Self> {
        match &value.data {
            ValueData::Bool(_) => Some(Self::Bool),
            ValueData::Int(_) => Some(Self::Int),
            ValueData::Float(_) => Some(Self::Float64),
            ValueData::String(_) => Some(Self::String),
            _ => None,
        }
    }

    fn for_runtime_kind(kind: &RuntimeTypeKind) -> Option<Self> {
        match kind {
            RuntimeTypeKind::Bool => Some(Self::Bool),
            RuntimeTypeKind::Int => Some(Self::Int),
            RuntimeTypeKind::Float64 => Some(Self::Float64),
            RuntimeTypeKind::String => Some(Self::String),
            _ => None,
        }
    }
}

pub(super) fn json_struct_field_specs(
    program: &Program,
    vm: &Vm,
    current: &Value,
    fields: &[(String, Value)],
) -> Result<Vec<JsonStructFieldSpec>, String> {
    if let Some(info) =
        value_runtime_type(program, vm, current).filter(|info| info.fields.len() == fields.len())
    {
        if let Some(inventory) = program_type_inventory(program) {
            return dominant_specs(runtime_field_candidates(&info, &inventory)?);
        }
    }
    dominant_specs(fallback_field_candidates(fields)?)
}

pub(super) fn json_struct_top_level_field_specs(
    program: &Program,
    vm: &Vm,
    current: &Value,
    fields: &[(String, Value)],
) -> Result<Vec<JsonStructFieldSpec>, String> {
    let Some(info) =
        value_runtime_type(program, vm, current).filter(|info| info.fields.len() == fields.len())
    else {
        return Ok(Vec::new());
    };
    let Some(inventory) = program_type_inventory(program) else {
        return Ok(Vec::new());
    };
    dominant_specs(top_level_runtime_field_candidates(&info, &inventory)?)
}

pub(super) fn json_field_value_by_path(
    vm: &Vm,
    program: &Program,
    current: &Value,
    path: &[usize],
) -> Option<Value> {
    if path.is_empty() {
        return Some(current.clone());
    }

    match &current.data {
        ValueData::Struct(fields) => {
            let next = fields.get(path[0])?.1.clone();
            json_field_value_by_path(vm, program, &next, &path[1..])
        }
        ValueData::Pointer(pointer) => {
            if pointer.is_nil() {
                None
            } else {
                let inner = vm.deref_pointer(program, current).ok()?;
                json_field_value_by_path(vm, program, &inner, path)
            }
        }
        _ => None,
    }
}

pub(super) fn json_value_is_empty(value: &Value) -> bool {
    match &value.data {
        ValueData::Nil => true,
        ValueData::Int(number) => *number == 0,
        ValueData::Float(number) => number.0 == 0.0,
        ValueData::String(text) => text.is_empty(),
        ValueData::Bool(boolean) => !boolean,
        ValueData::Error(_) => false,
        ValueData::Array(array) => array.is_empty(),
        ValueData::Slice(slice) => slice.is_nil || slice.is_empty(),
        ValueData::Map(map) => map.is_nil() || map.len() == 0,
        ValueData::Channel(_) => false,
        ValueData::Pointer(pointer) => pointer.is_nil(),
        ValueData::Function(_) => false,
        ValueData::Struct(_) => false,
    }
}

fn runtime_field_candidates(
    info: &RuntimeTypeInfo,
    inventory: &ProgramTypeInventory,
) -> Result<Vec<JsonStructFieldCandidate>, String> {
    let mut candidates = Vec::new();
    let mut order = 0;
    collect_runtime_field_candidates(info, inventory, Vec::new(), 0, &mut order, &mut candidates)?;
    Ok(candidates)
}

fn top_level_runtime_field_candidates(
    info: &RuntimeTypeInfo,
    inventory: &ProgramTypeInventory,
) -> Result<Vec<JsonStructFieldCandidate>, String> {
    let mut candidates = Vec::new();
    let mut order = 0;
    for (index, field) in info.fields.iter().enumerate() {
        if !field.embedded && !is_exported_field(&field.name) {
            continue;
        }
        let Some(tag) = json_tag_info(
            &field.name,
            field.tag.as_deref(),
            JsonQuotedFieldKind::for_runtime_field(field, inventory),
        )?
        else {
            continue;
        };
        if should_promote_embedded_field(field, &tag, inventory) {
            continue;
        }
        candidates.push(JsonStructFieldCandidate {
            path: vec![index],
            key: tag.key,
            omit_empty: tag.omit_empty,
            quoted_kind: tag.quoted_kind,
            tagged: tag.tagged,
            depth: 0,
            order: take_order(&mut order),
        });
    }
    Ok(candidates)
}

fn collect_runtime_field_candidates(
    info: &RuntimeTypeInfo,
    inventory: &ProgramTypeInventory,
    prefix: Vec<usize>,
    depth: usize,
    order: &mut usize,
    candidates: &mut Vec<JsonStructFieldCandidate>,
) -> Result<(), String> {
    for (index, field) in info.fields.iter().enumerate() {
        let mut path = prefix.clone();
        path.push(index);
        if !field.embedded && !is_exported_field(&field.name) {
            continue;
        }
        let Some(tag) = json_tag_info(
            &field.name,
            field.tag.as_deref(),
            JsonQuotedFieldKind::for_runtime_field(field, inventory),
        )?
        else {
            continue;
        };

        if should_promote_embedded_field(field, &tag, inventory) {
            if let Some(embedded) = embedded_struct_info(field, inventory) {
                collect_runtime_field_candidates(
                    &embedded,
                    inventory,
                    path,
                    depth + 1,
                    order,
                    candidates,
                )?;
            }
            continue;
        }

        candidates.push(JsonStructFieldCandidate {
            path,
            key: tag.key,
            omit_empty: tag.omit_empty,
            quoted_kind: tag.quoted_kind,
            tagged: tag.tagged,
            depth,
            order: take_order(order),
        });
    }
    Ok(())
}

fn fallback_field_candidates(
    fields: &[(String, Value)],
) -> Result<Vec<JsonStructFieldCandidate>, String> {
    let mut candidates = Vec::new();
    let mut order = 0;
    for (index, (name, value)) in fields.iter().enumerate() {
        if !is_exported_field(name) {
            continue;
        }
        let Some(tag) = json_tag_info(name, None, JsonQuotedFieldKind::for_value(value))? else {
            continue;
        };
        candidates.push(JsonStructFieldCandidate {
            path: vec![index],
            key: tag.key,
            omit_empty: tag.omit_empty,
            quoted_kind: tag.quoted_kind,
            tagged: tag.tagged,
            depth: 0,
            order: take_order(&mut order),
        });
    }
    Ok(candidates)
}

fn json_tag_info(
    field_name: &str,
    tag: Option<&str>,
    quoted_kind: Option<JsonQuotedFieldKind>,
) -> Result<Option<JsonTagInfo>, String> {
    let Some(tag_value) = tag
        .map(|tag| lookup_tag_value(tag, "json"))
        .transpose()
        .map_err(|reason| json_tag_error(field_name, &reason))?
        .flatten()
    else {
        return Ok(Some(JsonTagInfo {
            key: field_name.to_string(),
            omit_empty: false,
            quoted_kind: None,
            tagged: false,
            explicit_name: false,
        }));
    };
    if tag_value == "-" {
        return Ok(None);
    }

    let mut parts = tag_value.split(',');
    let raw_name = parts.next().unwrap_or_default();
    let mut omit_empty = false;
    let mut field_quoted_kind = None;
    for option in parts {
        match option {
            "" => return Err(json_tag_error(field_name, "empty json tag option")),
            "omitempty" => {
                if omit_empty {
                    return Err(json_tag_error(
                        field_name,
                        "duplicate json tag option \"omitempty\"",
                    ));
                }
                omit_empty = true;
            }
            "string" => {
                if field_quoted_kind.is_some() {
                    return Err(json_tag_error(
                        field_name,
                        "duplicate json tag option \"string\"",
                    ));
                }
                let Some(kind) = quoted_kind else {
                    return Err(json_tag_error(
                        field_name,
                        ",string is only supported on bool, int, float64, and string fields",
                    ));
                };
                field_quoted_kind = Some(kind);
            }
            other => {
                return Err(json_tag_error(
                    field_name,
                    &format!("unsupported json tag option \"{other}\""),
                ));
            }
        }
    }

    Ok(Some(JsonTagInfo {
        key: if raw_name.is_empty() {
            field_name.to_string()
        } else {
            raw_name.to_string()
        },
        omit_empty,
        quoted_kind: field_quoted_kind,
        tagged: true,
        explicit_name: !raw_name.is_empty(),
    }))
}

fn json_tag_error(field_name: &str, reason: &str) -> String {
    format!("json: malformed struct tag for field \"{field_name}\": {reason}")
}

fn lookup_tag_value(tag: &str, key: &str) -> Result<Option<String>, String> {
    let bare_prefix = format!("{key}:");
    let quoted_prefix = format!("{key}:\"");
    let mut found = None;
    for part in tag.split_ascii_whitespace() {
        if !part.starts_with(&bare_prefix) {
            continue;
        }
        let Some(value) = part.strip_prefix(&quoted_prefix) else {
            return Err(format!("expected quoted {key} tag value"));
        };
        let Some(value) = value.strip_suffix('"') else {
            return Err(format!("expected quoted {key} tag value"));
        };
        if found.is_some() {
            return Err(format!("duplicate {key} tag entry"));
        }
        found = Some(value.to_string());
    }
    Ok(found)
}

fn should_promote_embedded_field(
    field: &RuntimeTypeField,
    tag: &JsonTagInfo,
    inventory: &ProgramTypeInventory,
) -> bool {
    field.embedded && !tag.explicit_name && embedded_struct_info(field, inventory).is_some()
}

fn embedded_struct_info(
    field: &RuntimeTypeField,
    inventory: &ProgramTypeInventory,
) -> Option<RuntimeTypeInfo> {
    let info = inventory.resolve_concrete_type(&field.typ)?;
    match info.kind {
        RuntimeTypeKind::Struct => Some(info),
        RuntimeTypeKind::Pointer => info
            .elem
            .as_deref()
            .and_then(|element| inventory.resolve_concrete_type(element))
            .filter(|element| element.kind == RuntimeTypeKind::Struct),
        _ => None,
    }
}

fn is_exported_field(name: &str) -> bool {
    matches!(name.chars().next(), Some(ch) if ch.is_uppercase())
}

fn dominant_specs(
    candidates: Vec<JsonStructFieldCandidate>,
) -> Result<Vec<JsonStructFieldSpec>, String> {
    let mut grouped = HashMap::<String, Vec<JsonStructFieldCandidate>>::new();
    for candidate in candidates {
        grouped
            .entry(candidate.key.clone())
            .or_default()
            .push(candidate);
    }

    let mut winners = Vec::new();
    for mut group in grouped.into_values() {
        let min_depth = group
            .iter()
            .map(|candidate| candidate.depth)
            .min()
            .expect("field candidate group should not be empty");
        group.retain(|candidate| candidate.depth == min_depth);
        let min_path_len = group
            .iter()
            .map(|candidate| candidate.path.len())
            .min()
            .expect("field candidate group should not be empty");
        group.retain(|candidate| candidate.path.len() == min_path_len);
        if group.iter().any(|candidate| candidate.tagged) {
            group.retain(|candidate| candidate.tagged);
        }
        if group.len() == 1 {
            winners.push(group.pop().expect("single winner should exist"));
        }
    }

    winners.sort_by_key(|winner| winner.order);
    Ok(winners
        .into_iter()
        .map(|winner| JsonStructFieldSpec {
            path: winner.path,
            key: winner.key,
            omit_empty: winner.omit_empty,
            quoted_kind: winner.quoted_kind,
        })
        .collect())
}

fn take_order(order: &mut usize) -> usize {
    let next = *order;
    *order += 1;
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConcreteType, ProgramTypeInventory, RuntimeTypeField, RuntimeTypeInfo, TypeId, TYPE_STRING,
    };

    fn scalar(name: &str, kind: RuntimeTypeKind, type_id: TypeId) -> RuntimeTypeInfo {
        RuntimeTypeInfo::scalar(name, kind, Some(type_id))
    }

    #[test]
    fn embedded_precedence_prefers_outer_name_and_tagged_value() {
        let mut inventory = ProgramTypeInventory::default();
        inventory.register(scalar("string", RuntimeTypeKind::String, TYPE_STRING));

        inventory.register(RuntimeTypeInfo {
            display_name: "Tagged".into(),
            package_path: Some("main".into()),
            kind: RuntimeTypeKind::Struct,
            type_id: Some(TypeId(200)),
            fields: vec![
                RuntimeTypeField {
                    name: "Value".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: Some(r#"json:"Value""#.into()),
                },
                RuntimeTypeField {
                    name: "Shared".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
                RuntimeTypeField {
                    name: "Name".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
            ],
            elem: None,
            key: None,
            len: None,
            params: Vec::new(),
            results: Vec::new(),
            underlying: None,
            channel_direction: None,
        });
        inventory.register(RuntimeTypeInfo {
            display_name: "Plain".into(),
            package_path: Some("main".into()),
            kind: RuntimeTypeKind::Struct,
            type_id: Some(TypeId(201)),
            fields: vec![
                RuntimeTypeField {
                    name: "Value".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
                RuntimeTypeField {
                    name: "Shared".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
                RuntimeTypeField {
                    name: "Name".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
            ],
            elem: None,
            key: None,
            len: None,
            params: Vec::new(),
            results: Vec::new(),
            underlying: None,
            channel_direction: None,
        });

        let payload = RuntimeTypeInfo {
            display_name: "Payload".into(),
            package_path: Some("main".into()),
            kind: RuntimeTypeKind::Struct,
            type_id: Some(TypeId(202)),
            fields: vec![
                RuntimeTypeField {
                    name: "Name".into(),
                    typ: ConcreteType::TypeId(TYPE_STRING),
                    embedded: false,
                    tag: None,
                },
                RuntimeTypeField {
                    name: "Tagged".into(),
                    typ: ConcreteType::Pointer {
                        element: Box::new(ConcreteType::TypeId(TypeId(200))),
                    },
                    embedded: true,
                    tag: None,
                },
                RuntimeTypeField {
                    name: "Plain".into(),
                    typ: ConcreteType::TypeId(TypeId(201)),
                    embedded: true,
                    tag: None,
                },
            ],
            elem: None,
            key: None,
            len: None,
            params: Vec::new(),
            results: Vec::new(),
            underlying: None,
            channel_direction: None,
        };

        let specs =
            dominant_specs(runtime_field_candidates(&payload, &inventory).unwrap()).unwrap();
        assert!(
            specs
                .iter()
                .any(|spec| spec.key == "Name" && spec.path == vec![0]),
            "outer Name should remain addressable ahead of promoted embedded Name fields"
        );
        assert!(
            specs
                .iter()
                .any(|spec| spec.key == "Value" && spec.path == vec![1, 0]),
            "tagged embedded Value should win the promoted field slot"
        );
    }
}
