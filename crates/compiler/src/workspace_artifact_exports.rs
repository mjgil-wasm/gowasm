use super::*;
use crate::types::{InstanceKey, TypeConstraint, TypeParamDef};

#[path = "workspace_artifact_constraint_qualifier.rs"]
mod constraint_qualifier;
#[path = "workspace_artifact_visibility.rs"]
mod visibility;

pub(crate) use constraint_qualifier::qualify_type_constraint;
pub(crate) use visibility::{
    dequalify_visible_type, identifier_is_exported, method_key_is_exported, qualify_method_key,
    qualify_visible_type, visible_type_name_is_exported,
};

pub(super) fn package_selector_name(import_path: &str) -> &str {
    import_path.rsplit('/').next().unwrap_or(import_path)
}

pub(super) fn local_named_type_names(package_files: &[ParsedFile]) -> HashSet<String> {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.types.iter().map(|decl| decl.name.clone()))
        .collect()
}

pub(super) fn qualified_function_ids(
    import_path: &str,
    layout: &symbols::SymbolLayout,
) -> HashMap<String, usize> {
    layout
        .function_ids
        .iter()
        .filter(|(name, _)| identifier_is_exported(name))
        .map(|(name, function)| {
            (
                FunctionBuilder::imported_package_symbol_key(import_path, name),
                *function,
            )
        })
        .collect()
}

pub(super) fn qualified_generic_function_instances(
    function_instances: &HashMap<InstanceKey, usize>,
    import_path: &str,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<InstanceKey, usize> {
    function_instances
        .iter()
        .filter(|(key, _)| visible_type_name_is_exported(&key.base_name))
        .map(|(key, function)| {
            let base_name = if key.base_name.contains('/') || key.base_name.contains('.') {
                key.base_name.clone()
            } else {
                FunctionBuilder::imported_package_symbol_key(import_path, &key.base_name)
            };
            (
                InstanceKey {
                    base_name,
                    type_args: key
                        .type_args
                        .iter()
                        .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
                        .collect(),
                },
                *function,
            )
        })
        .collect()
}

pub(super) fn qualified_function_result_types(
    import_path: &str,
    layout: &symbols::SymbolLayout,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    layout
        .function_result_types
        .iter()
        .filter(|(name, _)| identifier_is_exported(name))
        .map(|(name, result_types)| {
            (
                FunctionBuilder::imported_package_symbol_key(import_path, name),
                result_types
                    .iter()
                    .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
                    .collect(),
            )
        })
        .collect()
}

pub(super) fn qualified_function_types(
    import_path: &str,
    layout: &symbols::SymbolLayout,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, String> {
    layout
        .function_types
        .iter()
        .filter(|(name, _)| identifier_is_exported(name))
        .map(|(name, function_type)| {
            (
                FunctionBuilder::imported_package_symbol_key(import_path, name),
                qualify_visible_type(function_type, package_selector, local_named_types),
            )
        })
        .collect()
}

pub(super) fn qualified_variadic_functions(
    import_path: &str,
    layout: &symbols::SymbolLayout,
) -> HashSet<String> {
    layout
        .variadic_functions
        .iter()
        .filter(|name| identifier_is_exported(name))
        .map(|name| FunctionBuilder::imported_package_symbol_key(import_path, name))
        .collect()
}

pub(super) fn qualified_globals(
    import_path: &str,
    globals: &HashMap<String, GlobalBinding>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, GlobalBinding> {
    globals
        .iter()
        .filter(|(name, _)| identifier_is_exported(name))
        .map(|(name, binding)| {
            (
                FunctionBuilder::imported_package_symbol_key(import_path, name),
                GlobalBinding {
                    index: binding.index,
                    typ: binding
                        .typ
                        .as_ref()
                        .map(|typ| qualify_visible_type(typ, package_selector, local_named_types)),
                    is_const: binding.is_const,
                    const_value: binding.const_value.clone().map(|mut value| {
                        if let Some(typ) = value.typ.as_ref() {
                            value.typ = Some(qualify_visible_type(
                                typ,
                                package_selector,
                                local_named_types,
                            ));
                        }
                        value
                    }),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_structs(
    package_files: &[ParsedFile],
    structs: &HashMap<String, StructTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, StructTypeDef> {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.types.iter())
        .filter(|decl| identifier_is_exported(&decl.name))
        .filter_map(|decl| {
            structs.get(&decl.name).map(|struct_type| {
                (
                    format!("{package_selector}.{}", decl.name),
                    StructTypeDef {
                        type_id: struct_type.type_id,
                        fields: struct_type
                            .fields
                            .iter()
                            .map(|field| gowasm_parser::TypeFieldDecl {
                                name: field.name.clone(),
                                typ: qualify_visible_type(
                                    &field.typ,
                                    package_selector,
                                    local_named_types,
                                ),
                                embedded: field.embedded,
                                tag: field.tag.clone(),
                            })
                            .collect(),
                    },
                )
            })
        })
        .collect()
}

pub(super) fn qualified_interfaces(
    package_files: &[ParsedFile],
    interfaces: &HashMap<String, InterfaceTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, InterfaceTypeDef> {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.types.iter())
        .filter(|decl| identifier_is_exported(&decl.name))
        .filter_map(|decl| {
            interfaces.get(&decl.name).map(|interface_type| {
                (
                    format!("{package_selector}.{}", decl.name),
                    InterfaceTypeDef {
                        type_id: interface_type.type_id,
                        methods: interface_type
                            .methods
                            .iter()
                            .map(|method| InterfaceMethodDecl {
                                name: method.name.clone(),
                                params: method
                                    .params
                                    .iter()
                                    .map(|param| gowasm_parser::Parameter {
                                        name: param.name.clone(),
                                        typ: qualify_visible_type(
                                            &param.typ,
                                            package_selector,
                                            local_named_types,
                                        ),
                                        variadic: param.variadic,
                                    })
                                    .collect(),
                                result_types: method
                                    .result_types
                                    .iter()
                                    .map(|typ| {
                                        qualify_visible_type(
                                            typ,
                                            package_selector,
                                            local_named_types,
                                        )
                                    })
                                    .collect(),
                            })
                            .collect(),
                    },
                )
            })
        })
        .collect()
}

pub(super) fn qualified_pointers(
    package_files: &[ParsedFile],
    pointers: &HashMap<String, TypeId>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, TypeId> {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.types.iter())
        .filter(|decl| identifier_is_exported(&decl.name))
        .filter_map(|decl| {
            let pointer_name = format!("*{}", decl.name);
            pointers.get(&pointer_name).map(|type_id| {
                (
                    qualify_visible_type(&pointer_name, package_selector, local_named_types),
                    *type_id,
                )
            })
        })
        .collect()
}

pub(super) fn qualified_aliases(
    package_files: &[ParsedFile],
    aliases: &HashMap<String, AliasTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, AliasTypeDef> {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.types.iter())
        .filter(|decl| identifier_is_exported(&decl.name))
        .filter_map(|decl| {
            aliases.get(&decl.name).map(|alias_type| {
                (
                    format!("{package_selector}.{}", decl.name),
                    AliasTypeDef {
                        type_id: alias_type.type_id,
                        underlying: qualify_visible_type(
                            &alias_type.underlying,
                            package_selector,
                            local_named_types,
                        ),
                    },
                )
            })
        })
        .collect()
}

pub(super) fn qualified_method_function_ids(
    layout: &symbols::SymbolLayout,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, usize> {
    layout
        .method_function_ids
        .iter()
        .filter(|(key, _)| method_key_is_exported(key))
        .map(|(key, function)| {
            (
                qualify_method_key(key, package_selector, local_named_types),
                *function,
            )
        })
        .collect()
}

pub(super) fn qualified_promoted_method_bindings(
    layout: &symbols::SymbolLayout,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, symbols::PromotedMethodBindingInfo> {
    layout
        .promoted_method_bindings
        .iter()
        .filter(|(key, _)| method_key_is_exported(key))
        .map(|(key, binding)| {
            (
                qualify_method_key(key, package_selector, local_named_types),
                symbols::PromotedMethodBindingInfo {
                    path: binding.path.clone(),
                    target_receiver_type: qualify_visible_type(
                        &binding.target_receiver_type,
                        package_selector,
                        local_named_types,
                    ),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_method_sets(
    layout: &symbols::SymbolLayout,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, Vec<InterfaceMethodDecl>> {
    layout
        .method_sets
        .iter()
        .filter(|(receiver_type, _)| visible_type_name_is_exported(receiver_type))
        .map(|(receiver_type, methods)| {
            (
                qualify_visible_type(receiver_type, package_selector, local_named_types),
                methods
                    .iter()
                    .filter(|method| identifier_is_exported(&method.name))
                    .map(|method| InterfaceMethodDecl {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|param| gowasm_parser::Parameter {
                                name: param.name.clone(),
                                typ: qualify_visible_type(
                                    &param.typ,
                                    package_selector,
                                    local_named_types,
                                ),
                                variadic: param.variadic,
                            })
                            .collect(),
                        result_types: method
                            .result_types
                            .iter()
                            .map(|typ| {
                                qualify_visible_type(typ, package_selector, local_named_types)
                            })
                            .collect(),
                    })
                    .collect(),
            )
        })
        .collect()
}

pub(super) fn qualified_generic_functions(
    generic_functions: &HashMap<String, GenericFunctionDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, GenericFunctionDef> {
    generic_functions
        .iter()
        .filter(|(name, _)| identifier_is_exported(name))
        .map(|(name, generic_function)| {
            (
                name.clone(),
                GenericFunctionDef {
                    type_params: generic_function
                        .type_params
                        .iter()
                        .map(|type_param| TypeParamDef {
                            name: type_param.name.clone(),
                            constraint: qualify_type_constraint(
                                &type_param.constraint,
                                package_selector,
                                local_named_types,
                            ),
                        })
                        .collect(),
                    param_types: generic_function
                        .param_types
                        .iter()
                        .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
                        .collect(),
                    result_types: generic_function
                        .result_types
                        .iter()
                        .map(|typ| qualify_visible_type(typ, package_selector, local_named_types))
                        .collect(),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_structs(
    structs: &HashMap<String, StructTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, StructTypeDef> {
    structs
        .iter()
        .filter(|(name, _)| visible_type_name_is_exported(name))
        .map(|(name, struct_type)| {
            (
                qualify_visible_type(name, package_selector, local_named_types),
                StructTypeDef {
                    type_id: struct_type.type_id,
                    fields: struct_type
                        .fields
                        .iter()
                        .map(|field| gowasm_parser::TypeFieldDecl {
                            name: field.name.clone(),
                            typ: qualify_visible_type(
                                &field.typ,
                                package_selector,
                                local_named_types,
                            ),
                            embedded: field.embedded,
                            tag: field.tag.clone(),
                        })
                        .collect(),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_interfaces(
    interfaces: &HashMap<String, InterfaceTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, InterfaceTypeDef> {
    interfaces
        .iter()
        .filter(|(name, _)| visible_type_name_is_exported(name))
        .map(|(name, interface_type)| {
            (
                qualify_visible_type(name, package_selector, local_named_types),
                InterfaceTypeDef {
                    type_id: interface_type.type_id,
                    methods: interface_type
                        .methods
                        .iter()
                        .map(|method| InterfaceMethodDecl {
                            name: method.name.clone(),
                            params: method
                                .params
                                .iter()
                                .map(|param| gowasm_parser::Parameter {
                                    name: param.name.clone(),
                                    typ: qualify_visible_type(
                                        &param.typ,
                                        package_selector,
                                        local_named_types,
                                    ),
                                    variadic: param.variadic,
                                })
                                .collect(),
                            result_types: method
                                .result_types
                                .iter()
                                .map(|typ| {
                                    qualify_visible_type(typ, package_selector, local_named_types)
                                })
                                .collect(),
                        })
                        .collect(),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_pointers(
    pointers: &HashMap<String, TypeId>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, TypeId> {
    pointers
        .iter()
        .filter(|(name, _)| visible_type_name_is_exported(name))
        .map(|(name, type_id)| {
            (
                qualify_visible_type(name, package_selector, local_named_types),
                *type_id,
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_aliases(
    aliases: &HashMap<String, AliasTypeDef>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, AliasTypeDef> {
    aliases
        .iter()
        .filter(|(name, _)| visible_type_name_is_exported(name))
        .map(|(name, alias_type)| {
            (
                qualify_visible_type(name, package_selector, local_named_types),
                AliasTypeDef {
                    type_id: alias_type.type_id,
                    underlying: qualify_visible_type(
                        &alias_type.underlying,
                        package_selector,
                        local_named_types,
                    ),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_method_function_ids(
    method_function_ids: &HashMap<String, usize>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, usize> {
    method_function_ids
        .iter()
        .map(|(key, function)| {
            (
                qualify_method_key(key, package_selector, local_named_types),
                *function,
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_promoted_method_bindings(
    promoted_method_bindings: &HashMap<String, symbols::PromotedMethodBindingInfo>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, symbols::PromotedMethodBindingInfo> {
    promoted_method_bindings
        .iter()
        .map(|(key, binding)| {
            (
                qualify_method_key(key, package_selector, local_named_types),
                symbols::PromotedMethodBindingInfo {
                    path: binding.path.clone(),
                    target_receiver_type: qualify_visible_type(
                        &binding.target_receiver_type,
                        package_selector,
                        local_named_types,
                    ),
                },
            )
        })
        .collect()
}

pub(super) fn qualified_instantiated_method_sets(
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> HashMap<String, Vec<InterfaceMethodDecl>> {
    method_sets
        .iter()
        .map(|(receiver_type, methods)| {
            (
                qualify_visible_type(receiver_type, package_selector, local_named_types),
                methods
                    .iter()
                    .map(|method| InterfaceMethodDecl {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|param| gowasm_parser::Parameter {
                                name: param.name.clone(),
                                typ: qualify_visible_type(
                                    &param.typ,
                                    package_selector,
                                    local_named_types,
                                ),
                                variadic: param.variadic,
                            })
                            .collect(),
                        result_types: method
                            .result_types
                            .iter()
                            .map(|typ| {
                                qualify_visible_type(typ, package_selector, local_named_types)
                            })
                            .collect(),
                    })
                    .collect(),
            )
        })
        .collect()
}
