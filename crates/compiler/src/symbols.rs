use std::collections::{HashMap, HashSet};

use gowasm_parser::{InterfaceMethodDecl, Parameter, SourceFile};
use gowasm_vm::{MethodBinding, PromotedFieldAccess, PromotedFieldStep, TypeId};
use serde::{Deserialize, Serialize};

use crate::{
    types::{
        format_function_type, is_generic_receiver_type, parse_pointer_type, AliasTypeDef,
        GenericTypeDef, StructTypeDef,
    },
    CompileError,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PromotedMethodBindingInfo {
    pub(crate) path: Vec<PromotedFieldStep>,
    pub(crate) target_receiver_type: String,
}

pub(crate) struct SymbolLayout {
    pub(crate) function_ids: HashMap<String, usize>,
    pub(crate) function_result_types: HashMap<String, Vec<String>>,
    pub(crate) function_types: HashMap<String, String>,
    pub(crate) variadic_functions: HashSet<String>,
    pub(crate) method_function_ids: HashMap<String, usize>,
    pub(crate) promoted_method_bindings: HashMap<String, PromotedMethodBindingInfo>,
    pub(crate) method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
    pub(crate) methods: Vec<MethodBinding>,
    pub(crate) init_functions: Vec<usize>,
}

#[derive(Clone)]
struct MethodEntry {
    decl: InterfaceMethodDecl,
    function: usize,
    target_receiver_type: String,
    path: Vec<PromotedFieldStep>,
}

pub(crate) fn collect_symbols<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a SourceFile)>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    alias_types: &HashMap<String, AliasTypeDef>,
    generic_types: &HashMap<String, GenericTypeDef>,
) -> Result<SymbolLayout, CompileError> {
    let mut function_ids = HashMap::new();
    let mut function_result_types = HashMap::new();
    let mut function_types = HashMap::new();
    let mut variadic_functions = HashSet::new();
    let mut methods: Vec<MethodBinding> = Vec::new();
    let mut init_functions = Vec::new();
    let mut declared_methods: HashMap<String, String> = HashMap::new();
    let mut method_entries: HashMap<String, Vec<MethodEntry>> = HashMap::new();
    let mut next_id = 0usize;

    for (_path, file) in files {
        for function in &file.functions {
            if !function.type_params.is_empty() {
                continue;
            }
            if function
                .receiver
                .as_ref()
                .is_some_and(|receiver| is_generic_receiver_type(&receiver.typ, generic_types))
            {
                continue;
            }
            if let Some(receiver) = &function.receiver {
                let (receiver_type, is_pointer_receiver) =
                    parse_method_receiver_type(&receiver.typ, struct_types, alias_types)?;
                let duplicate_key = qualified_method_name(receiver_type, &function.name);
                if let Some(existing) =
                    declared_methods.insert(duplicate_key.clone(), receiver.typ.clone())
                {
                    return Err(CompileError::DuplicateMethod {
                        package: file.package_name.clone(),
                        type_name: existing,
                        name: function.name.clone(),
                    });
                }

                let receiver_type_id = struct_types
                    .get(receiver_type)
                    .map(|s| s.type_id)
                    .or_else(|| alias_types.get(receiver_type).map(|a| a.type_id))
                    .ok_or_else(|| CompileError::UnknownReceiverType {
                        type_name: receiver.typ.clone(),
                    })?;
                let decl = interface_method_decl(function);
                let (param_types, result_types) = method_binding_signature(function);
                if is_pointer_receiver {
                    let pointer_type =
                        pointer_types.get(&receiver.typ).copied().ok_or_else(|| {
                            CompileError::UnknownReceiverType {
                                type_name: receiver.typ.clone(),
                            }
                        })?;
                    insert_method_entry(
                        &mut method_entries,
                        receiver.typ.clone(),
                        MethodEntry {
                            decl: decl.clone(),
                            function: next_id,
                            target_receiver_type: receiver.typ.clone(),
                            path: Vec::new(),
                        },
                    );
                    methods.push(MethodBinding {
                        receiver_type: pointer_type,
                        target_receiver_type: pointer_type,
                        name: function.name.clone(),
                        function: next_id,
                        param_types: param_types.clone(),
                        result_types: result_types.clone(),
                        promoted_fields: Vec::new(),
                    });
                } else {
                    insert_method_entry(
                        &mut method_entries,
                        receiver_type.to_string(),
                        MethodEntry {
                            decl: decl.clone(),
                            function: next_id,
                            target_receiver_type: receiver_type.to_string(),
                            path: Vec::new(),
                        },
                    );
                    methods.push(MethodBinding {
                        receiver_type: receiver_type_id,
                        target_receiver_type: receiver_type_id,
                        name: function.name.clone(),
                        function: next_id,
                        param_types: param_types.clone(),
                        result_types: result_types.clone(),
                        promoted_fields: Vec::new(),
                    });
                    if let Some(pointer_type) = pointer_types.get(&format!("*{receiver_type}")) {
                        insert_method_entry(
                            &mut method_entries,
                            format!("*{receiver_type}"),
                            MethodEntry {
                                decl: decl.clone(),
                                function: next_id,
                                target_receiver_type: receiver_type.to_string(),
                                path: Vec::new(),
                            },
                        );
                        methods.push(MethodBinding {
                            receiver_type: *pointer_type,
                            target_receiver_type: receiver_type_id,
                            name: function.name.clone(),
                            function: next_id,
                            param_types: param_types.clone(),
                            result_types: result_types.clone(),
                            promoted_fields: Vec::new(),
                        });
                    }
                }
            } else if function.name == "init" {
                init_functions.push(next_id);
            } else if function_ids.contains_key(&function.name) {
                return Err(CompileError::DuplicateFunction {
                    package: file.package_name.clone(),
                    name: function.name.clone(),
                });
            } else {
                let param_types = function
                    .params
                    .iter()
                    .map(|parameter| parameter.typ.clone())
                    .collect::<Vec<_>>();
                function_ids.insert(function.name.clone(), next_id);
                function_result_types.insert(function.name.clone(), function.result_types.clone());
                if function.params.last().is_some_and(|p| p.variadic) {
                    variadic_functions.insert(function.name.clone());
                }
                function_types.insert(
                    function.name.clone(),
                    format_function_type(&param_types, &function.result_types),
                );
            }
            next_id += 1;
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (type_name, struct_type) in struct_types {
            changed |= promote_embedded_methods_for_receiver(
                type_name,
                type_name,
                struct_type.type_id,
                false,
                struct_type,
                struct_types,
                pointer_types,
                alias_types,
                &mut method_entries,
                &mut methods,
            )?;
            if let Some(&pointer_type_id) = pointer_types.get(&format!("*{type_name}")) {
                changed |= promote_embedded_methods_for_receiver(
                    type_name,
                    &format!("*{type_name}"),
                    pointer_type_id,
                    true,
                    struct_type,
                    struct_types,
                    pointer_types,
                    alias_types,
                    &mut method_entries,
                    &mut methods,
                )?;
            }
        }
    }

    let mut method_function_ids = HashMap::new();
    let mut promoted_method_bindings = HashMap::new();
    let mut method_sets: HashMap<String, Vec<InterfaceMethodDecl>> = HashMap::new();
    for (receiver_type, entries) in method_entries {
        let receiver_methods = method_sets.entry(receiver_type.clone()).or_default();
        for entry in entries {
            let key = qualified_method_name(&receiver_type, &entry.decl.name);
            method_function_ids.insert(key.clone(), entry.function);
            receiver_methods.push(entry.decl.clone());
            if !entry.path.is_empty() {
                promoted_method_bindings.insert(
                    key,
                    PromotedMethodBindingInfo {
                        path: entry.path.clone(),
                        target_receiver_type: entry.target_receiver_type.clone(),
                    },
                );
            }
        }
    }

    Ok(SymbolLayout {
        function_ids,
        function_result_types,
        function_types,
        variadic_functions,
        method_function_ids,
        promoted_method_bindings,
        method_sets,
        methods,
        init_functions,
    })
}

fn qualified_method_name(receiver_type: &str, method: &str) -> String {
    format!("{receiver_type}.{method}")
}

fn interface_method_decl(function: &gowasm_parser::FunctionDecl) -> InterfaceMethodDecl {
    InterfaceMethodDecl {
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|parameter| Parameter {
                name: parameter.name.clone(),
                typ: parameter.typ.clone(),
                variadic: parameter.variadic,
            })
            .collect(),
        result_types: function.result_types.clone(),
    }
}

fn method_binding_signature(function: &gowasm_parser::FunctionDecl) -> (Vec<String>, Vec<String>) {
    (
        function
            .params
            .iter()
            .map(|parameter| parameter.typ.clone())
            .collect(),
        function.result_types.clone(),
    )
}

fn insert_method_entry(
    method_entries: &mut HashMap<String, Vec<MethodEntry>>,
    receiver_type: String,
    entry: MethodEntry,
) -> bool {
    let methods = method_entries.entry(receiver_type).or_default();
    if methods
        .iter()
        .any(|candidate| candidate.decl.name == entry.decl.name)
    {
        return false;
    }
    methods.push(entry);
    true
}

#[allow(clippy::too_many_arguments)]
fn promote_embedded_methods_for_receiver(
    type_name: &str,
    receiver_key: &str,
    receiver_type_id: TypeId,
    pointer_receiver: bool,
    struct_type: &StructTypeDef,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_entries: &mut HashMap<String, Vec<MethodEntry>>,
    methods: &mut Vec<MethodBinding>,
) -> Result<bool, CompileError> {
    let mut changed = false;
    for field in &struct_type.fields {
        if !field.embedded {
            continue;
        }
        for source_receiver in promoted_source_receivers(&field.typ, pointer_receiver) {
            let source_methods = method_entries
                .get(&source_receiver)
                .cloned()
                .unwrap_or_default();
            let first_step = promoted_field_step(&field.typ, &field.name, &source_receiver);
            for entry in source_methods {
                let mut path = vec![first_step.clone()];
                path.extend(entry.path.clone());
                let new_entry = MethodEntry {
                    decl: entry.decl.clone(),
                    function: entry.function,
                    target_receiver_type: entry.target_receiver_type.clone(),
                    path: path.clone(),
                };
                if !insert_method_entry(method_entries, receiver_key.to_string(), new_entry) {
                    continue;
                }
                let target_receiver_type = resolve_runtime_type_id(
                    &entry.target_receiver_type,
                    struct_types,
                    pointer_types,
                    alias_types,
                )
                .ok_or_else(|| CompileError::UnknownReceiverType {
                    type_name: entry.target_receiver_type.clone(),
                })?;
                methods.push(MethodBinding {
                    receiver_type: receiver_type_id,
                    target_receiver_type,
                    name: entry.decl.name.clone(),
                    function: entry.function,
                    param_types: entry
                        .decl
                        .params
                        .iter()
                        .map(|parameter| parameter.typ.clone())
                        .collect(),
                    result_types: entry.decl.result_types.clone(),
                    promoted_fields: path,
                });
                changed = true;
            }
        }
    }
    let _ = type_name;
    Ok(changed)
}

fn promoted_source_receivers(field_type: &str, pointer_receiver: bool) -> Vec<String> {
    let embedded_type = field_type.strip_prefix('*').unwrap_or(field_type);
    if field_type.starts_with('*') {
        vec![embedded_type.to_string(), field_type.to_string()]
    } else if pointer_receiver {
        vec![embedded_type.to_string(), format!("*{embedded_type}")]
    } else {
        vec![embedded_type.to_string()]
    }
}

fn promoted_field_step(
    field_type: &str,
    field_name: &str,
    source_receiver: &str,
) -> PromotedFieldStep {
    let embedded_type = field_type.strip_prefix('*').unwrap_or(field_type);
    let access = if !field_type.starts_with('*') && source_receiver == format!("*{embedded_type}") {
        PromotedFieldAccess::Pointer
    } else {
        PromotedFieldAccess::Value
    };
    PromotedFieldStep {
        field: field_name.to_string(),
        access,
    }
}

fn resolve_runtime_type_id(
    type_name: &str,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    alias_types: &HashMap<String, AliasTypeDef>,
) -> Option<TypeId> {
    pointer_types
        .get(type_name)
        .copied()
        .or_else(|| struct_types.get(type_name).map(|typ| typ.type_id))
        .or_else(|| alias_types.get(type_name).map(|typ| typ.type_id))
}

fn parse_method_receiver_type<'a>(
    receiver_type: &'a str,
    struct_types: &HashMap<String, StructTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
) -> Result<(&'a str, bool), CompileError> {
    if struct_types.contains_key(receiver_type) || alias_types.contains_key(receiver_type) {
        return Ok((receiver_type, false));
    }
    if let Some(inner) = parse_pointer_type(receiver_type) {
        if struct_types.contains_key(inner) || alias_types.contains_key(inner) {
            return Ok((inner, true));
        }
    }
    Err(CompileError::UnknownReceiverType {
        type_name: receiver_type.to_string(),
    })
}
