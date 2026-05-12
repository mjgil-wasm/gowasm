use gowasm_parser::{parse_type_repr, TypeChannelDirection, TypeRepr};

use super::{parse_array_type, parse_channel_type, parse_map_type, ChannelDirection};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TypeKey {
    Name(String),
    Pointer(Box<TypeKey>),
    Slice(Box<TypeKey>),
    Array {
        len: usize,
        element: Box<TypeKey>,
    },
    Map {
        key: Box<TypeKey>,
        value: Box<TypeKey>,
    },
    Channel {
        direction: ChannelDirection,
        element: Box<TypeKey>,
    },
    Function {
        params: Vec<TypeKey>,
        results: Vec<TypeKey>,
    },
    Interface,
    GenericInstance {
        base: String,
        type_args: Vec<TypeKey>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionSignatureKey {
    pub(crate) params: Vec<TypeKey>,
    pub(crate) results: Vec<TypeKey>,
}

impl TypeKey {
    pub(crate) fn render(&self) -> String {
        match self {
            Self::Name(name) => name.clone(),
            Self::Pointer(inner) => format!("*{}", inner.render()),
            Self::Slice(inner) => format!("[]{}", inner.render()),
            Self::Array { len, element } => format!("[{len}]{}", element.render()),
            Self::Map { key, value } => format!("map[{}]{}", key.render(), value.render()),
            Self::Channel { direction, element } => match direction {
                ChannelDirection::Bidirectional => format!("chan {}", element.render()),
                ChannelDirection::SendOnly => format!("chan<- {}", element.render()),
                ChannelDirection::ReceiveOnly => format!("<-chan {}", element.render()),
            },
            Self::Function { params, results } => format!(
                "__gowasm_func__({})->({})",
                params
                    .iter()
                    .map(TypeKey::render)
                    .collect::<Vec<_>>()
                    .join(","),
                results
                    .iter()
                    .map(TypeKey::render)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Interface => "interface{}".to_string(),
            Self::GenericInstance { base, type_args } => format!(
                "{base}[{}]",
                type_args
                    .iter()
                    .map(TypeKey::render)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

impl From<TypeRepr> for TypeKey {
    fn from(value: TypeRepr) -> Self {
        match value {
            TypeRepr::Name(name) => Self::Name(name),
            TypeRepr::Pointer(inner) => Self::Pointer(Box::new(Self::from(*inner))),
            TypeRepr::Slice(inner) => Self::Slice(Box::new(Self::from(*inner))),
            TypeRepr::Array { len, element } => Self::Array {
                len,
                element: Box::new(Self::from(*element)),
            },
            TypeRepr::Map { key, value } => Self::Map {
                key: Box::new(Self::from(*key)),
                value: Box::new(Self::from(*value)),
            },
            TypeRepr::Channel { direction, element } => Self::Channel {
                direction: match direction {
                    TypeChannelDirection::Bidirectional => ChannelDirection::Bidirectional,
                    TypeChannelDirection::SendOnly => ChannelDirection::SendOnly,
                    TypeChannelDirection::ReceiveOnly => ChannelDirection::ReceiveOnly,
                },
                element: Box::new(Self::from(*element)),
            },
            TypeRepr::Function { params, results } => Self::Function {
                params: params.into_iter().map(Self::from).collect(),
                results: results.into_iter().map(Self::from).collect(),
            },
            TypeRepr::Struct { fields } => Self::Name(TypeRepr::Struct { fields }.render()),
            TypeRepr::Interface => Self::Interface,
            TypeRepr::GenericInstance { base, type_args } => Self::GenericInstance {
                base,
                type_args: type_args.into_iter().map(Self::from).collect(),
            },
        }
    }
}

pub(crate) fn parse_type_key(typ: &str) -> Option<TypeKey> {
    if let Some(signature) = function_signature_key(typ) {
        return Some(TypeKey::Function {
            params: signature.params,
            results: signature.results,
        });
    }
    if typ == "interface{}" {
        return Some(TypeKey::Interface);
    }
    if let Some(inner) = typ.strip_prefix("[]") {
        return Some(TypeKey::Slice(Box::new(parse_type_key(inner)?)));
    }
    if let Some((len, inner)) = parse_array_type(typ) {
        return Some(TypeKey::Array {
            len,
            element: Box::new(parse_type_key(inner)?),
        });
    }
    if let Some(inner) = typ.strip_prefix('*') {
        return Some(TypeKey::Pointer(Box::new(parse_type_key(inner)?)));
    }
    if let Some((key, value)) = parse_map_type(typ) {
        return Some(TypeKey::Map {
            key: Box::new(parse_type_key(key)?),
            value: Box::new(parse_type_key(value)?),
        });
    }
    if let Some(channel) = parse_channel_type(typ) {
        return Some(TypeKey::Channel {
            direction: channel.direction,
            element: Box::new(parse_type_key(channel.element_type)?),
        });
    }
    if let Some((base, type_args)) = split_generic_type_name_raw(typ) {
        return Some(TypeKey::GenericInstance {
            base,
            type_args: type_args
                .iter()
                .map(|type_arg| parse_type_key(type_arg))
                .collect::<Option<_>>()?,
        });
    }
    parse_type_repr(typ)
        .ok()
        .map(TypeKey::from)
        .or_else(|| Some(TypeKey::Name(typ.to_string())))
}

pub(crate) fn function_signature_key(typ: &str) -> Option<FunctionSignatureKey> {
    let (params, results) = parse_canonical_function_type(typ)?;
    Some(FunctionSignatureKey {
        params: params
            .iter()
            .map(|param| parse_type_key(param))
            .collect::<Option<_>>()?,
        results: results
            .iter()
            .map(|result| parse_type_key(result))
            .collect::<Option<_>>()?,
    })
}

pub(crate) fn parse_function_type(typ: &str) -> Option<(Vec<String>, Vec<String>)> {
    let signature = function_signature_key(typ)?;
    Some((
        signature
            .params
            .into_iter()
            .map(|param| param.render())
            .collect(),
        signature
            .results
            .into_iter()
            .map(|result| result.render())
            .collect(),
    ))
}

pub(crate) fn function_signatures_match(expected: &str, actual: &str) -> bool {
    match (
        function_signature_key(expected),
        function_signature_key(actual),
    ) {
        (Some(expected), Some(actual)) => expected == actual,
        _ => false,
    }
}

fn split_generic_type_name_raw(typ: &str) -> Option<(String, Vec<String>)> {
    let open = typ.find('[')?;
    if !typ.ends_with(']') {
        return None;
    }
    let base = &typ[..open];
    if base.is_empty() {
        return None;
    }
    let inner = &typ[open + 1..typ.len() - 1];
    if inner.is_empty() {
        return None;
    }
    Some((base.to_string(), split_top_level_types(inner)?))
}

fn parse_canonical_function_type(typ: &str) -> Option<(Vec<String>, Vec<String>)> {
    let body = typ.strip_prefix("__gowasm_func__(")?;
    let separator = find_function_type_separator(body)?;
    let params = &body[..separator];
    let results = &body[separator + 4..];
    let results = results.strip_suffix(')')?;
    Some((
        split_top_level_types(params)?,
        split_top_level_types(results)?,
    ))
}

fn split_top_level_types(types: &str) -> Option<Vec<String>> {
    if types.is_empty() {
        return Some(Vec::new());
    }

    let mut values = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    for (index, ch) in types.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let value = types[start..index].trim();
                if value.is_empty() {
                    return None;
                }
                values.push(value.to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    if paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let tail = types[start..].trim();
    if tail.is_empty() {
        return None;
    }
    values.push(tail.to_string());
    Some(values)
}

fn find_function_type_separator(body: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in body.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 && body[index..].starts_with(")->(") => return Some(index),
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}
