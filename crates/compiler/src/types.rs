use std::collections::HashMap;

use gowasm_parser::{
    BinaryOp, InterfaceMethodDecl, Parameter, SourceFile, TypeDeclKind, TypeFieldDecl,
};
use gowasm_vm::{
    CompareOp, TypeId, TYPE_BASE64_ENCODING_PTR, TYPE_FS_FILE_MODE, TYPE_HTTP_CLIENT_PTR,
    TYPE_HTTP_HEADER, TYPE_HTTP_REQUEST_PTR, TYPE_HTTP_RESPONSE_PTR, TYPE_REFLECT_KIND,
    TYPE_REFLECT_STRUCT_TAG, TYPE_REGEXP, TYPE_STRINGS_REPLACER, TYPE_SYNC_MUTEX_PTR,
    TYPE_SYNC_ONCE_PTR, TYPE_SYNC_RW_MUTEX_PTR, TYPE_SYNC_WAIT_GROUP_PTR, TYPE_TIME_DURATION,
    TYPE_TIME_PTR, TYPE_TIME_TIMER_PTR, TYPE_URL_PTR, TYPE_URL_USERINFO_PTR, TYPE_URL_VALUES,
};
use serde::{Deserialize, Serialize};

use crate::CompileError;

#[path = "types_imported.rs"]
mod imported_impl;
#[path = "types_assert.rs"]
mod types_assert;
#[path = "types_generics.rs"]
mod types_generics;
#[path = "types_keys.rs"]
mod types_keys;

pub(crate) use imported_impl::is_imported_type_only_package;
#[allow(unused_imports)]
pub(crate) use types_generics::{
    build_substitutions, check_type_constraint, infer_type_args, lower_type_param,
    substitute_type_params, validate_type_args,
};
pub(crate) use types_keys::{
    function_signatures_match, parse_function_type, parse_type_key, TypeKey,
};
pub(crate) const FIRST_USER_TYPE_ID: u32 = imported_impl::FIRST_USER_TYPE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StructTypeDef {
    pub(crate) type_id: TypeId,
    pub(crate) fields: Vec<TypeFieldDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InterfaceTypeDef {
    pub(crate) type_id: TypeId,
    pub(crate) methods: Vec<InterfaceMethodDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AliasTypeDef {
    pub(crate) type_id: TypeId,
    pub(crate) underlying: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TypeConstraint {
    Any,
    Comparable,
    Interface(String),
    InterfaceLiteral(ConstraintInterface),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConstraintInterface {
    pub(crate) methods: Vec<InterfaceMethodDecl>,
    pub(crate) embeds: Vec<TypeConstraint>,
    pub(crate) type_sets: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TypeParamDef {
    pub(crate) name: String,
    pub(crate) constraint: TypeConstraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GenericFunctionDef {
    pub(crate) type_params: Vec<TypeParamDef>,
    pub(crate) param_types: Vec<String>,
    pub(crate) result_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GenericTypeDef {
    pub(crate) type_params: Vec<TypeParamDef>,
    pub(crate) kind: TypeDeclKind,
    pub(crate) methods: Vec<GenericMethodDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GenericMethodDef {
    pub(crate) name: String,
    pub(crate) params: Vec<Parameter>,
    pub(crate) result_types: Vec<String>,
    pub(crate) pointer_receiver: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct InstanceKey {
    pub(crate) base_name: String,
    pub(crate) type_args: Vec<String>,
}

impl InstanceKey {
    pub(crate) fn mangled_name(&self) -> String {
        format!("{}[{}]", self.base_name, self.type_args.join(","))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct InstantiationCache {
    pub(crate) function_instances: HashMap<InstanceKey, String>,
    pub(crate) type_instances: HashMap<InstanceKey, TypeId>,
}

impl InstantiationCache {
    pub(crate) fn function_name(&mut self, key: &InstanceKey) -> String {
        self.function_instances
            .entry(key.clone())
            .or_insert_with(|| key.mangled_name())
            .clone()
    }

    pub(crate) fn type_id(&self, key: &InstanceKey) -> Option<TypeId> {
        self.type_instances.get(key).copied()
    }

    pub(crate) fn record_type(&mut self, key: InstanceKey, type_id: TypeId) {
        self.type_instances.entry(key).or_insert(type_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeTables {
    pub(crate) structs: HashMap<String, StructTypeDef>,
    pub(crate) interfaces: HashMap<String, InterfaceTypeDef>,
    pub(crate) pointers: HashMap<String, TypeId>,
    pub(crate) aliases: HashMap<String, AliasTypeDef>,
    pub(crate) generic_functions: HashMap<String, GenericFunctionDef>,
    pub(crate) generic_types: HashMap<String, GenericTypeDef>,
    pub(crate) instantiation_cache: InstantiationCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ChannelDirection {
    Bidirectional,
    SendOnly,
    ReceiveOnly,
}

impl ChannelDirection {
    pub(crate) fn accepts_recv(self) -> bool {
        matches!(self, Self::Bidirectional | Self::ReceiveOnly)
    }

    pub(crate) fn accepts_send(self) -> bool {
        matches!(self, Self::Bidirectional | Self::SendOnly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChannelType<'a> {
    pub(crate) direction: ChannelDirection,
    pub(crate) element_type: &'a str,
}

pub(crate) fn split_generic_type_name(typ: &str) -> Option<(String, Vec<String>)> {
    match parse_type_key(typ)? {
        TypeKey::GenericInstance { base, type_args } => Some((
            base,
            type_args
                .into_iter()
                .map(|type_arg| type_arg.render())
                .collect(),
        )),
        _ => None,
    }
}

pub(crate) fn parse_generic_receiver_type(
    receiver_type: &str,
) -> Option<(bool, String, Vec<String>)> {
    let (pointer_receiver, base_type) = if let Some(inner) = receiver_type.strip_prefix('*') {
        (true, inner)
    } else {
        (false, receiver_type)
    };
    split_generic_type_name(base_type).map(|(name, type_args)| (pointer_receiver, name, type_args))
}

pub(crate) fn is_generic_receiver_type(
    receiver_type: &str,
    generic_types: &HashMap<String, GenericTypeDef>,
) -> bool {
    parse_generic_receiver_type(receiver_type)
        .is_some_and(|(_, base_name, _)| generic_types.contains_key(&base_name))
}

pub(crate) fn collect_type_tables<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a SourceFile)>,
) -> Result<TypeTables, CompileError> {
    let files: Vec<_> = files.into_iter().collect();
    let mut structs = HashMap::new();
    imported_impl::seed_imported_structs(&mut structs);
    let mut interfaces = HashMap::new();
    imported_impl::seed_imported_interfaces(&mut interfaces);
    let mut pointers = HashMap::from([
        ("*base64.Encoding".into(), TYPE_BASE64_ENCODING_PTR),
        ("*regexp.Regexp".into(), TYPE_REGEXP),
        ("*strings.Replacer".into(), TYPE_STRINGS_REPLACER),
        ("*sync.WaitGroup".into(), TYPE_SYNC_WAIT_GROUP_PTR),
        ("*sync.Once".into(), TYPE_SYNC_ONCE_PTR),
        ("*sync.Mutex".into(), TYPE_SYNC_MUTEX_PTR),
        ("*sync.RWMutex".into(), TYPE_SYNC_RW_MUTEX_PTR),
        ("*http.Client".into(), TYPE_HTTP_CLIENT_PTR),
        ("*http.Request".into(), TYPE_HTTP_REQUEST_PTR),
        ("*http.Response".into(), TYPE_HTTP_RESPONSE_PTR),
        ("*url.URL".into(), TYPE_URL_PTR),
        ("*url.Userinfo".into(), TYPE_URL_USERINFO_PTR),
        ("*time.Time".into(), TYPE_TIME_PTR),
        ("*time.Timer".into(), TYPE_TIME_TIMER_PTR),
    ]);
    let mut aliases = HashMap::from([(
        "time.Duration".into(),
        AliasTypeDef {
            type_id: TYPE_TIME_DURATION,
            underlying: "int".into(),
        },
    )]);
    aliases.insert(
        "fs.FileMode".into(),
        AliasTypeDef {
            type_id: TYPE_FS_FILE_MODE,
            underlying: "int".into(),
        },
    );
    aliases.insert(
        "context.CancelFunc".into(),
        AliasTypeDef {
            type_id: TypeId(101),
            underlying: "__gowasm_func__()->()".into(),
        },
    );
    aliases.insert(
        "reflect.Kind".into(),
        AliasTypeDef {
            type_id: TYPE_REFLECT_KIND,
            underlying: "int".into(),
        },
    );
    aliases.insert(
        "reflect.StructTag".into(),
        AliasTypeDef {
            type_id: TYPE_REFLECT_STRUCT_TAG,
            underlying: "string".into(),
        },
    );
    aliases.insert(
        "http.Header".into(),
        AliasTypeDef {
            type_id: TYPE_HTTP_HEADER,
            underlying: "map[string][]string".into(),
        },
    );
    aliases.insert(
        "url.Values".into(),
        AliasTypeDef {
            type_id: TYPE_URL_VALUES,
            underlying: "map[string][]string".into(),
        },
    );
    let mut next_type_id = imported_impl::FIRST_USER_TYPE_ID;
    let mut pending_embeds: Vec<(String, Vec<String>)> = Vec::new();
    let mut generic_types = HashMap::new();
    let mut generic_functions = HashMap::new();

    for (_path, file) in &files {
        for type_decl in &file.types {
            if structs.contains_key(&type_decl.name)
                || interfaces.contains_key(&type_decl.name)
                || aliases.contains_key(&type_decl.name)
                || generic_types.contains_key(&type_decl.name)
            {
                return Err(CompileError::DuplicateType {
                    package: file.package_name.clone(),
                    name: type_decl.name.clone(),
                });
            }
            if !type_decl.type_params.is_empty() {
                generic_types.insert(
                    type_decl.name.clone(),
                    GenericTypeDef {
                        type_params: type_decl.type_params.iter().map(lower_type_param).collect(),
                        kind: type_decl.kind.clone(),
                        methods: Vec::new(),
                    },
                );
                continue;
            }

            match &type_decl.kind {
                TypeDeclKind::Struct { fields } => {
                    let struct_type_id = TypeId(next_type_id);
                    next_type_id += 1;
                    structs.insert(
                        type_decl.name.clone(),
                        StructTypeDef {
                            type_id: struct_type_id,
                            fields: fields.clone(),
                        },
                    );
                    pointers.insert(format!("*{}", type_decl.name), TypeId(next_type_id));
                }
                TypeDeclKind::Alias { underlying } => {
                    let alias_type_id = TypeId(next_type_id);
                    next_type_id += 1;
                    aliases.insert(
                        type_decl.name.clone(),
                        AliasTypeDef {
                            type_id: alias_type_id,
                            underlying: underlying.clone(),
                        },
                    );
                    pointers.insert(format!("*{}", type_decl.name), TypeId(next_type_id));
                }
                TypeDeclKind::Interface { methods, embeds } => {
                    pending_embeds.push((type_decl.name.clone(), embeds.clone()));
                    interfaces.insert(
                        type_decl.name.clone(),
                        InterfaceTypeDef {
                            type_id: TypeId(next_type_id),
                            methods: methods.clone(),
                        },
                    );
                }
            }
            next_type_id += 1;
        }
    }

    for (_path, file) in &files {
        for function in &file.functions {
            if function.receiver.is_some() && !function.type_params.is_empty() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "methods cannot declare their own type parameters in the current subset: `{}`",
                        function.name
                    ),
                });
            }
            if !function.type_params.is_empty() {
                generic_functions.insert(
                    function.name.clone(),
                    GenericFunctionDef {
                        type_params: function.type_params.iter().map(lower_type_param).collect(),
                        param_types: function.params.iter().map(|p| p.typ.clone()).collect(),
                        result_types: function.result_types.clone(),
                    },
                );
                continue;
            }
            let Some(receiver) = &function.receiver else {
                continue;
            };
            let Some((pointer_receiver, base_name, type_args)) =
                parse_generic_receiver_type(&receiver.typ)
            else {
                continue;
            };
            let generic_type = generic_types.get_mut(&base_name).ok_or_else(|| {
                CompileError::UnknownReceiverType {
                    type_name: receiver.typ.clone(),
                }
            })?;
            if type_args.len() != generic_type.type_params.len() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "generic method receiver `{}` uses {} type argument(s), but `{}` declares {}",
                        receiver.typ,
                        type_args.len(),
                        base_name,
                        generic_type.type_params.len()
                    ),
                });
            }
            generic_type.methods.push(GenericMethodDef {
                name: function.name.clone(),
                params: function.params.clone(),
                result_types: function.result_types.clone(),
                pointer_receiver,
            });
        }
    }

    interfaces
        .entry("error".into())
        .or_insert_with(|| InterfaceTypeDef {
            type_id: TypeId(next_type_id),
            methods: vec![InterfaceMethodDecl {
                name: "Error".into(),
                params: Vec::<Parameter>::new(),
                result_types: vec!["string".into()],
            }],
        });

    for (name, embeds) in pending_embeds {
        let mut merged = Vec::new();
        for embed in &embeds {
            let Some(embedded) = interfaces.get(embed) else {
                return Err(CompileError::Unsupported {
                    detail: format!("interface `{name}` embeds unknown interface `{embed}`"),
                });
            };
            merged.extend(embedded.methods.clone());
        }
        if let Some(iface) = interfaces.get_mut(&name) {
            iface.methods.extend(merged);
        }
    }

    Ok(TypeTables {
        structs,
        interfaces,
        pointers,
        aliases,
        generic_functions,
        generic_types,
        instantiation_cache: InstantiationCache::default(),
    })
}

pub(crate) fn user_type_id_span(type_tables: &TypeTables) -> u32 {
    let max_type_id = type_tables
        .structs
        .values()
        .map(|typ| typ.type_id.0)
        .chain(type_tables.interfaces.values().map(|typ| typ.type_id.0))
        .chain(type_tables.aliases.values().map(|typ| typ.type_id.0))
        .chain(type_tables.pointers.values().map(|typ| typ.0))
        .filter(|type_id| *type_id >= FIRST_USER_TYPE_ID)
        .max();
    max_type_id
        .map(|type_id| type_id - FIRST_USER_TYPE_ID + 1)
        .unwrap_or(0)
}

pub(crate) fn offset_user_type_ids(type_tables: &mut TypeTables, offset: u32) {
    if offset == 0 {
        return;
    }
    for struct_type in type_tables.structs.values_mut() {
        offset_type_id(&mut struct_type.type_id, offset);
    }
    for interface_type in type_tables.interfaces.values_mut() {
        offset_type_id(&mut interface_type.type_id, offset);
    }
    for alias_type in type_tables.aliases.values_mut() {
        offset_type_id(&mut alias_type.type_id, offset);
    }
    for pointer_type in type_tables.pointers.values_mut() {
        offset_type_id(pointer_type, offset);
    }
}

fn offset_type_id(type_id: &mut TypeId, offset: u32) {
    if type_id.0 >= FIRST_USER_TYPE_ID {
        type_id.0 += offset;
    }
}

pub(crate) fn parse_array_type(typ: &str) -> Option<(usize, &str)> {
    if !typ.starts_with('[') || typ.starts_with("[]") {
        return None;
    }

    let end = typ.find(']')?;
    let len = typ[1..end].parse().ok()?;
    Some((len, &typ[end + 1..]))
}

pub(crate) fn parse_map_type(typ: &str) -> Option<(&str, &str)> {
    if !typ.starts_with("map[") {
        return None;
    }

    let mut depth = 1usize;
    let start = 4usize;
    for (offset, ch) in typ[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + offset;
                    return Some((&typ[start..end], &typ[end + 1..]));
                }
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn parse_pointer_type(typ: &str) -> Option<&str> {
    typ.strip_prefix('*')
}

pub(crate) fn parse_channel_type(typ: &str) -> Option<ChannelType<'_>> {
    if let Some(element_type) = typ.strip_prefix("<-chan ") {
        return Some(ChannelType {
            direction: ChannelDirection::ReceiveOnly,
            element_type,
        });
    }

    if let Some(element_type) = typ.strip_prefix("chan<- ") {
        return Some(ChannelType {
            direction: ChannelDirection::SendOnly,
            element_type,
        });
    }

    typ.strip_prefix("chan ").map(|element_type| ChannelType {
        direction: ChannelDirection::Bidirectional,
        element_type,
    })
}

pub(crate) fn channel_types_assignable(expected: &str, actual: &str) -> bool {
    let Some(expected) = parse_channel_type(expected) else {
        return false;
    };
    let Some(actual) = parse_channel_type(actual) else {
        return false;
    };
    if expected.element_type != actual.element_type {
        return false;
    }

    match expected.direction {
        ChannelDirection::Bidirectional => actual.direction == ChannelDirection::Bidirectional,
        ChannelDirection::ReceiveOnly => actual.direction != ChannelDirection::SendOnly,
        ChannelDirection::SendOnly => actual.direction != ChannelDirection::ReceiveOnly,
    }
}

pub(crate) fn underlying_type_name(
    typ: &str,
    alias_types: &HashMap<String, AliasTypeDef>,
) -> String {
    if matches!(typ, "byte" | "rune") {
        return "int".into();
    }
    let mut current = typ;
    while let Some(alias) = alias_types.get(current) {
        current = &alias.underlying;
    }
    current.to_string()
}

pub(crate) fn is_named_type(
    typ: &str,
    struct_types: &HashMap<String, StructTypeDef>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
) -> bool {
    matches!(typ, "int" | "float64" | "string" | "bool" | "error" | "any")
        || struct_types.contains_key(typ)
        || interface_types.contains_key(typ)
        || alias_types.contains_key(typ)
}

pub(crate) fn format_function_type(param_types: &[String], result_types: &[String]) -> String {
    format!(
        "__gowasm_func__({})->({})",
        param_types.join(","),
        result_types.join(",")
    )
}

pub(crate) fn display_type(typ: &str) -> String {
    if let Some((params, results)) = parse_function_type(typ) {
        let params_str = params.join(", ");
        match results.len() {
            0 => format!("func({params_str})"),
            1 => format!("func({params_str}) {}", results[0]),
            _ => format!("func({params_str}) ({})", results.join(", ")),
        }
    } else {
        typ.to_string()
    }
}

pub(crate) fn lower_compare_op(op: BinaryOp) -> CompareOp {
    match op {
        BinaryOp::Add => unreachable!("add is handled separately"),
        BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
            unreachable!("arithmetic ops are handled separately")
        }
        BinaryOp::BitOr
        | BinaryOp::BitXor
        | BinaryOp::BitAnd
        | BinaryOp::BitClear
        | BinaryOp::ShiftLeft
        | BinaryOp::ShiftRight => unreachable!("bitwise ops are handled separately"),
        BinaryOp::And | BinaryOp::Or => unreachable!("logical ops are handled separately"),
        BinaryOp::Equal => CompareOp::Equal,
        BinaryOp::NotEqual => CompareOp::NotEqual,
        BinaryOp::Less => CompareOp::Less,
        BinaryOp::LessEqual => CompareOp::LessEqual,
        BinaryOp::Greater => CompareOp::Greater,
        BinaryOp::GreaterEqual => CompareOp::GreaterEqual,
    }
}

fn channel_direction_matches(expected: ChannelDirection, actual: ChannelDirection) -> bool {
    match expected {
        ChannelDirection::Bidirectional => actual == ChannelDirection::Bidirectional,
        ChannelDirection::ReceiveOnly => actual != ChannelDirection::SendOnly,
        ChannelDirection::SendOnly => actual != ChannelDirection::ReceiveOnly,
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
