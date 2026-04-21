use super::*;
use crate::types::split_generic_type_name;

pub(crate) fn qualify_visible_type(
    typ: &str,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> String {
    if typ.is_empty()
        || matches!(
            typ,
            "int"
                | "byte"
                | "rune"
                | "float64"
                | "string"
                | "bool"
                | "interface{}"
                | "error"
                | "any"
        )
    {
        return typ.to_string();
    }
    if let Some(inner) = parse_pointer_type(typ) {
        return format!(
            "*{}",
            qualify_visible_type(inner, package_selector, local_named_types)
        );
    }
    if let Some((len, element_type)) = parse_array_type(typ) {
        return format!(
            "[{len}]{}",
            qualify_visible_type(element_type, package_selector, local_named_types)
        );
    }
    if let Some(element_type) = typ.strip_prefix("[]") {
        return format!(
            "[]{}",
            qualify_visible_type(element_type, package_selector, local_named_types)
        );
    }
    if let Some((key_type, value_type)) = parse_map_type(typ) {
        return format!(
            "map[{}]{}",
            qualify_visible_type(key_type, package_selector, local_named_types),
            qualify_visible_type(value_type, package_selector, local_named_types),
        );
    }
    if let Some(channel_type) = parse_channel_type(typ) {
        let element_type = qualify_visible_type(
            channel_type.element_type,
            package_selector,
            local_named_types,
        );
        return match channel_type.direction {
            types::ChannelDirection::Bidirectional => format!("chan {element_type}"),
            types::ChannelDirection::SendOnly => format!("chan<- {element_type}"),
            types::ChannelDirection::ReceiveOnly => format!("<-chan {element_type}"),
        };
    }
    if let Some((params, results)) = parse_function_type(typ) {
        let params = params
            .iter()
            .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        let results = results
            .iter()
            .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        return types::format_function_type(&params, &results);
    }
    if let Some((base_name, type_args)) = split_generic_type_name(typ) {
        let base_name = qualify_visible_type(&base_name, package_selector, local_named_types);
        let type_args = type_args
            .iter()
            .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        return format!("{base_name}[{}]", type_args.join(","));
    }
    if local_named_types.contains(typ) {
        format!("{package_selector}.{typ}")
    } else {
        typ.to_string()
    }
}

pub(crate) fn dequalify_visible_type(
    typ: &str,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> String {
    if typ.is_empty()
        || matches!(
            typ,
            "int"
                | "byte"
                | "rune"
                | "float64"
                | "string"
                | "bool"
                | "interface{}"
                | "error"
                | "any"
        )
    {
        return typ.to_string();
    }
    if let Some(inner) = parse_pointer_type(typ) {
        return format!(
            "*{}",
            dequalify_visible_type(inner, package_selector, local_named_types)
        );
    }
    if let Some((len, element_type)) = parse_array_type(typ) {
        return format!(
            "[{len}]{}",
            dequalify_visible_type(element_type, package_selector, local_named_types)
        );
    }
    if let Some(element_type) = typ.strip_prefix("[]") {
        return format!(
            "[]{}",
            dequalify_visible_type(element_type, package_selector, local_named_types)
        );
    }
    if let Some((key_type, value_type)) = parse_map_type(typ) {
        return format!(
            "map[{}]{}",
            dequalify_visible_type(key_type, package_selector, local_named_types),
            dequalify_visible_type(value_type, package_selector, local_named_types),
        );
    }
    if let Some(channel_type) = parse_channel_type(typ) {
        let element_type = dequalify_visible_type(
            channel_type.element_type,
            package_selector,
            local_named_types,
        );
        return match channel_type.direction {
            types::ChannelDirection::Bidirectional => format!("chan {element_type}"),
            types::ChannelDirection::SendOnly => format!("chan<- {element_type}"),
            types::ChannelDirection::ReceiveOnly => format!("<-chan {element_type}"),
        };
    }
    if let Some((params, results)) = parse_function_type(typ) {
        let params = params
            .iter()
            .map(|typ| dequalify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        let results = results
            .iter()
            .map(|typ| dequalify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        return types::format_function_type(&params, &results);
    }
    if let Some((base_name, type_args)) = split_generic_type_name(typ) {
        let base_name = dequalify_visible_type(&base_name, package_selector, local_named_types);
        let type_args = type_args
            .iter()
            .map(|typ| dequalify_visible_type(typ, package_selector, local_named_types))
            .collect::<Vec<_>>();
        return format!("{base_name}[{}]", type_args.join(","));
    }
    typ.strip_prefix(&format!("{package_selector}."))
        .filter(|name| local_named_types.contains(*name))
        .map(str::to_string)
        .unwrap_or_else(|| typ.to_string())
}

pub(crate) fn qualify_method_key(
    key: &str,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> String {
    let Some((receiver_type, method)) = key.rsplit_once('.') else {
        return key.to_string();
    };
    format!(
        "{}.{method}",
        qualify_visible_type(receiver_type, package_selector, local_named_types)
    )
}

pub(crate) fn identifier_is_exported(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

pub(crate) fn visible_type_name_is_exported(name: &str) -> bool {
    let head = name.strip_prefix('*').unwrap_or(name);
    let head = head.split('[').next().unwrap_or(head);
    let head = head.rsplit_once('.').map(|(_, tail)| tail).unwrap_or(head);
    identifier_is_exported(head)
}

pub(crate) fn method_key_is_exported(key: &str) -> bool {
    let Some((receiver_type, method)) = key.rsplit_once('.') else {
        return false;
    };
    visible_type_name_is_exported(receiver_type) && identifier_is_exported(method)
}
