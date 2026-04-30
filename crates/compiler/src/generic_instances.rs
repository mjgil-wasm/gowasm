use std::collections::HashMap;

use gowasm_parser::{FunctionDecl, FunctionSourceSpans, InterfaceMethodDecl, TypeDeclKind};
use gowasm_vm::{MethodBinding, PromotedFieldAccess, PromotedFieldStep, TypeId};

use super::*;
use crate::program::compile_function_with_imports;
use crate::types::{
    build_substitutions, is_named_type, substitute_type_params, underlying_type_name, AliasTypeDef,
    GenericTypeDef, InstanceKey, InterfaceTypeDef, StructTypeDef,
};

#[path = "generic_instances_imported.rs"]
mod imported_impl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GenericMethodTemplate {
    pub(crate) decl: FunctionDecl,
    pub(crate) source_path: String,
    pub(crate) spans: FunctionSourceSpans,
    pub(crate) imported_packages: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub(crate) struct InstantiatedGenerics {
    next_type_id: u32,
    structs: HashMap<String, StructTypeDef>,
    interfaces: HashMap<String, InterfaceTypeDef>,
    aliases: HashMap<String, AliasTypeDef>,
    pointers: HashMap<String, TypeId>,
    method_function_ids: HashMap<String, usize>,
    promoted_method_bindings: HashMap<String, symbols::PromotedMethodBindingInfo>,
    method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
    methods: Vec<MethodBinding>,
}

#[derive(Debug, Clone)]
struct ReceiverMethodEntry {
    decl: InterfaceMethodDecl,
    function: usize,
    target_receiver_type: String,
    path: Vec<PromotedFieldStep>,
}

impl InstantiatedGenerics {
    pub(crate) fn new(
        struct_types: &HashMap<String, StructTypeDef>,
        interface_types: &HashMap<String, InterfaceTypeDef>,
        alias_types: &HashMap<String, AliasTypeDef>,
        pointer_types: &HashMap<String, TypeId>,
    ) -> Self {
        let next_type_id = struct_types
            .values()
            .map(|typ| typ.type_id.0)
            .chain(interface_types.values().map(|typ| typ.type_id.0))
            .chain(alias_types.values().map(|typ| typ.type_id.0))
            .chain(pointer_types.values().map(|typ| typ.0))
            .max()
            .unwrap_or(0)
            + 1;
        Self {
            next_type_id,
            ..Self::default()
        }
    }

    pub(crate) fn struct_type(&self, name: &str) -> Option<&StructTypeDef> {
        self.structs.get(name)
    }

    pub(crate) fn struct_types(&self) -> &HashMap<String, StructTypeDef> {
        &self.structs
    }

    pub(crate) fn interface_type(&self, name: &str) -> Option<&InterfaceTypeDef> {
        self.interfaces.get(name)
    }

    pub(crate) fn interface_types(&self) -> &HashMap<String, InterfaceTypeDef> {
        &self.interfaces
    }

    pub(crate) fn alias_type(&self, name: &str) -> Option<&AliasTypeDef> {
        self.aliases.get(name)
    }

    pub(crate) fn alias_types(&self) -> &HashMap<String, AliasTypeDef> {
        &self.aliases
    }

    pub(crate) fn pointer_type(&self, name: &str) -> Option<TypeId> {
        self.pointers.get(name).copied()
    }

    pub(crate) fn pointer_types(&self) -> &HashMap<String, TypeId> {
        &self.pointers
    }

    pub(crate) fn method_sets(&self, receiver_type: &str) -> Option<&Vec<InterfaceMethodDecl>> {
        self.method_sets.get(receiver_type)
    }

    pub(crate) fn method_function_id(&self, key: &str) -> Option<usize> {
        self.method_function_ids.get(key).copied()
    }

    pub(crate) fn method_function_ids(&self) -> &HashMap<String, usize> {
        &self.method_function_ids
    }

    pub(crate) fn promoted_method_binding(
        &self,
        key: &str,
    ) -> Option<&symbols::PromotedMethodBindingInfo> {
        self.promoted_method_bindings.get(key)
    }

    pub(crate) fn promoted_method_bindings(
        &self,
    ) -> &HashMap<String, symbols::PromotedMethodBindingInfo> {
        &self.promoted_method_bindings
    }

    pub(crate) fn all_method_sets(&self) -> &HashMap<String, Vec<InterfaceMethodDecl>> {
        &self.method_sets
    }

    pub(crate) fn methods(&self) -> &[MethodBinding] {
        &self.methods
    }

    fn allocate_type_id(&mut self) -> TypeId {
        let type_id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        type_id
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn instantiated_struct_type(&self, name: &str) -> Option<StructTypeDef> {
        self.generation
            .instantiated_generics
            .struct_type(name)
            .cloned()
            .or_else(|| self.env.struct_types.get(name).cloned())
            .or_else(|| self.env.imported_package_tables.structs.get(name).cloned())
    }

    pub(super) fn instantiated_interface_type(&self, name: &str) -> Option<InterfaceTypeDef> {
        self.generation
            .instantiated_generics
            .interface_type(name)
            .cloned()
            .or_else(|| self.env.interface_types.get(name).cloned())
            .or_else(|| {
                self.env
                    .imported_package_tables
                    .interfaces
                    .get(name)
                    .cloned()
            })
    }

    pub(super) fn instantiated_alias_type(&self, name: &str) -> Option<AliasTypeDef> {
        self.generation
            .instantiated_generics
            .alias_type(name)
            .cloned()
            .or_else(|| self.env.alias_types.get(name).cloned())
            .or_else(|| self.env.imported_package_tables.aliases.get(name).cloned())
    }

    pub(super) fn instantiated_pointer_type(&self, name: &str) -> Option<TypeId> {
        self.generation
            .instantiated_generics
            .pointer_type(name)
            .or_else(|| self.env.pointer_types.get(name).copied())
            .or_else(|| self.env.imported_package_tables.pointers.get(name).copied())
    }

    pub(super) fn instantiated_method_set(
        &self,
        receiver_type: &str,
    ) -> Option<Vec<InterfaceMethodDecl>> {
        self.generation
            .instantiated_generics
            .method_sets(receiver_type)
            .cloned()
            .or_else(|| self.env.method_sets.get(receiver_type).cloned())
            .or_else(|| {
                self.env
                    .imported_package_tables
                    .method_sets
                    .get(receiver_type)
                    .cloned()
            })
    }

    pub(super) fn instantiated_method_function_id(&self, key: &str) -> Option<usize> {
        self.generation
            .instantiated_generics
            .method_function_id(key)
            .or_else(|| self.env.method_function_ids.get(key).copied())
            .or_else(|| {
                self.env
                    .imported_package_tables
                    .method_function_ids
                    .get(key)
                    .copied()
            })
    }

    pub(super) fn instantiated_promoted_method_binding(
        &self,
        key: &str,
    ) -> Option<symbols::PromotedMethodBindingInfo> {
        self.generation
            .instantiated_generics
            .promoted_method_binding(key)
            .cloned()
            .or_else(|| self.env.promoted_method_bindings.get(key).cloned())
            .or_else(|| {
                self.env
                    .imported_package_tables
                    .promoted_method_bindings
                    .get(key)
                    .cloned()
            })
    }

    pub(super) fn instantiated_underlying_type_name(&self, typ: &str) -> String {
        let mut current = underlying_type_name(typ, self.env.alias_types);
        loop {
            let Some(alias) = self.instantiated_alias_type(&current) else {
                return current;
            };
            current = alias.underlying;
        }
    }

    pub(super) fn instantiated_named_type(&self, typ: &str) -> bool {
        is_named_type(
            typ,
            self.env.struct_types,
            self.env.interface_types,
            self.env.alias_types,
        ) || self
            .generation
            .instantiated_generics
            .struct_type(typ)
            .is_some()
            || self
                .generation
                .instantiated_generics
                .interface_type(typ)
                .is_some()
            || self
                .generation
                .instantiated_generics
                .alias_type(typ)
                .is_some()
            || self.env.imported_package_tables.structs.contains_key(typ)
            || self
                .env
                .imported_package_tables
                .interfaces
                .contains_key(typ)
            || self.env.imported_package_tables.aliases.contains_key(typ)
    }

    pub(super) fn instantiate_generic_named_type(
        &mut self,
        base_name: &str,
        type_args: &[String],
    ) -> Result<(), CompileError> {
        let key = InstanceKey {
            base_name: base_name.to_string(),
            type_args: type_args.to_vec(),
        };
        let concrete_name = key.mangled_name();
        if self.generation.instantiation_cache.type_id(&key).is_some()
            || self.instantiated_named_type(&concrete_name)
        {
            return Ok(());
        }

        let generic_type = self
            .env
            .generic_types
            .get(base_name)
            .cloned()
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!("unknown generic type `{base_name}`"),
            })?;
        self.validate_type_args_in_context(&generic_type.type_params, type_args)?;
        let substitutions = build_substitutions(&generic_type.type_params, type_args);

        match &generic_type.kind {
            TypeDeclKind::Struct { fields } => {
                let type_id = self.generation.instantiated_generics.allocate_type_id();
                let pointer_type = self.generation.instantiated_generics.allocate_type_id();
                self.generation.instantiated_generics.structs.insert(
                    concrete_name.clone(),
                    StructTypeDef {
                        type_id,
                        fields: Vec::new(),
                    },
                );
                self.generation
                    .instantiated_generics
                    .pointers
                    .insert(format!("*{concrete_name}"), pointer_type);
                self.generation
                    .instantiation_cache
                    .record_type(key.clone(), type_id);

                let concrete_fields = fields
                    .iter()
                    .map(|field| {
                        let field_type = substitute_type_params(&field.typ, &substitutions);
                        self.ensure_runtime_visible_type(&field_type)?;
                        Ok(gowasm_parser::TypeFieldDecl {
                            name: field.name.clone(),
                            typ: field_type,
                            embedded: field.embedded,
                            tag: field.tag.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?;
                self.generation
                    .instantiated_generics
                    .structs
                    .get_mut(&concrete_name)
                    .expect("placeholder generic struct should exist")
                    .fields = concrete_fields;
                self.instantiate_generic_methods(
                    base_name,
                    &concrete_name,
                    type_args,
                    &generic_type,
                )?;
            }
            TypeDeclKind::Alias { underlying } => {
                let type_id = self.generation.instantiated_generics.allocate_type_id();
                let pointer_type = self.generation.instantiated_generics.allocate_type_id();
                self.generation.instantiated_generics.aliases.insert(
                    concrete_name.clone(),
                    AliasTypeDef {
                        type_id,
                        underlying: String::new(),
                    },
                );
                self.generation
                    .instantiated_generics
                    .pointers
                    .insert(format!("*{concrete_name}"), pointer_type);
                self.generation
                    .instantiation_cache
                    .record_type(key.clone(), type_id);

                let concrete_underlying = substitute_type_params(underlying, &substitutions);
                self.ensure_runtime_visible_type(&concrete_underlying)?;
                self.generation
                    .instantiated_generics
                    .aliases
                    .get_mut(&concrete_name)
                    .expect("placeholder generic alias should exist")
                    .underlying = concrete_underlying;
                self.instantiate_generic_methods(
                    base_name,
                    &concrete_name,
                    type_args,
                    &generic_type,
                )?;
            }
            TypeDeclKind::Interface { methods, embeds } => {
                let mut concrete_methods = methods
                    .iter()
                    .map(|method| {
                        let params = method
                            .params
                            .iter()
                            .map(|param| {
                                let typ = substitute_type_params(&param.typ, &substitutions);
                                self.ensure_runtime_visible_type(&typ)?;
                                Ok(gowasm_parser::Parameter {
                                    name: param.name.clone(),
                                    typ,
                                    variadic: param.variadic,
                                })
                            })
                            .collect::<Result<Vec<_>, CompileError>>()?;
                        let result_types = method
                            .result_types
                            .iter()
                            .map(|typ| {
                                let typ = substitute_type_params(typ, &substitutions);
                                self.ensure_runtime_visible_type(&typ)?;
                                Ok(typ)
                            })
                            .collect::<Result<Vec<_>, CompileError>>()?;
                        Ok(InterfaceMethodDecl {
                            name: method.name.clone(),
                            params,
                            result_types,
                        })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?;
                for embed in embeds {
                    let embed = substitute_type_params(embed, &substitutions);
                    self.ensure_runtime_visible_type(&embed)?;
                    let embedded = self.instantiated_interface_type(&embed).ok_or_else(|| {
                        CompileError::Unsupported {
                            detail: format!(
                                "interface `{concrete_name}` embeds unknown interface `{embed}`"
                            ),
                        }
                    })?;
                    concrete_methods.extend(embedded.methods);
                }
                let type_id = self.generation.instantiated_generics.allocate_type_id();
                self.generation
                    .instantiation_cache
                    .record_type(key, type_id);
                self.generation.instantiated_generics.interfaces.insert(
                    concrete_name,
                    InterfaceTypeDef {
                        type_id,
                        methods: concrete_methods,
                    },
                );
            }
        }

        Ok(())
    }

    fn instantiate_generic_methods(
        &mut self,
        base_name: &str,
        concrete_name: &str,
        type_args: &[String],
        generic_type: &GenericTypeDef,
    ) -> Result<(), CompileError> {
        let Some(templates) = self.env.generic_method_templates.get(base_name).cloned() else {
            return Ok(());
        };
        let substitutions = build_substitutions(&generic_type.type_params, type_args);
        let mut concrete_methods = Vec::with_capacity(templates.len());
        for template in templates {
            let concrete_decl = crate::generic_substitute::instantiate_function_decl(
                &template.decl,
                &substitutions,
                template.decl.name.clone(),
            );
            let receiver_type = concrete_decl
                .receiver
                .as_ref()
                .map(|receiver| receiver.typ.clone())
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!(
                        "generic method `{}` must have a receiver",
                        concrete_decl.name
                    ),
                })?;
            for param in &concrete_decl.params {
                self.ensure_runtime_visible_type(&param.typ)?;
            }
            for result in &concrete_decl.result_types {
                self.ensure_runtime_visible_type(result)?;
            }
            let cache_name = if receiver_type.starts_with('*') {
                format!("*{base_name}.{}", concrete_decl.name)
            } else {
                format!("{base_name}.{}", concrete_decl.name)
            };
            let (_cache_name, function, _created) =
                self.generation.generated_functions.reserve_instance(
                    InstanceKey {
                        base_name: cache_name,
                        type_args: type_args.to_vec(),
                    },
                    self.generation.instantiation_cache,
                );
            let method_decl = InterfaceMethodDecl {
                name: concrete_decl.name.clone(),
                params: concrete_decl.params.clone(),
                result_types: concrete_decl.result_types.clone(),
            };
            self.register_direct_method_binding(&receiver_type, function, &method_decl)?;
            concrete_methods.push((
                template.imported_packages,
                template.source_path,
                template.spans,
                concrete_decl,
                function,
            ));
        }

        if let Some(struct_type) = self.instantiated_struct_type(concrete_name) {
            self.promote_embedded_methods_for_instantiated_receiver(
                concrete_name,
                concrete_name,
                struct_type.type_id,
                false,
                &struct_type,
            )?;
            if let Some(pointer_type) = self.instantiated_pointer_type(&format!("*{concrete_name}"))
            {
                self.promote_embedded_methods_for_instantiated_receiver(
                    concrete_name,
                    &format!("*{concrete_name}"),
                    pointer_type,
                    true,
                    &struct_type,
                )?;
            }
        }

        for (imported_packages, source_path, spans, concrete_decl, function) in concrete_methods {
            let compiled = compile_function_with_imports(
                imported_packages,
                &source_path,
                Some(&spans),
                &concrete_decl,
                self.env.imported_package_tables,
                self.env.function_ids,
                self.env.function_result_types,
                self.env.function_types,
                self.env.variadic_functions,
                self.env.generic_functions,
                self.env.generic_types,
                self.env.generic_function_templates,
                self.env.generic_method_templates,
                self.generation.instantiation_cache,
                self.generation.generated_functions,
                self.generation.instantiated_generics,
                self.env.method_function_ids,
                self.env.promoted_method_bindings,
                self.env.struct_types,
                self.env.pointer_types,
                self.env.interface_types,
                self.env.alias_types,
                self.env.method_sets,
                self.env.globals,
            )?;
            self.generation.generated_functions.fill(function, compiled);
        }
        Ok(())
    }

    fn register_direct_method_binding(
        &mut self,
        receiver_type: &str,
        function: usize,
        method: &InterfaceMethodDecl,
    ) -> Result<(), CompileError> {
        let base_receiver = receiver_type.strip_prefix('*').unwrap_or(receiver_type);
        let pointer_receiver = receiver_type.starts_with('*');
        if pointer_receiver {
            self.register_method_binding(
                receiver_type,
                function,
                method,
                receiver_type,
                Vec::new(),
            )?;
            return Ok(());
        }
        self.register_method_binding(base_receiver, function, method, base_receiver, Vec::new())?;
        if self
            .instantiated_pointer_type(&format!("*{base_receiver}"))
            .is_some()
        {
            self.register_method_binding(
                &format!("*{base_receiver}"),
                function,
                method,
                base_receiver,
                Vec::new(),
            )?;
        }
        Ok(())
    }

    fn register_method_binding(
        &mut self,
        receiver_type: &str,
        function: usize,
        method: &InterfaceMethodDecl,
        target_receiver_type: &str,
        path: Vec<PromotedFieldStep>,
    ) -> Result<bool, CompileError> {
        let key = format!("{receiver_type}.{}", method.name);
        if self.instantiated_method_function_id(&key).is_some() {
            return Ok(false);
        }
        let receiver_type_id = self
            .instantiated_pointer_type(receiver_type)
            .or_else(|| {
                self.instantiated_struct_type(receiver_type)
                    .map(|typ| typ.type_id)
            })
            .or_else(|| {
                self.instantiated_alias_type(receiver_type)
                    .map(|typ| typ.type_id)
            })
            .ok_or_else(|| CompileError::UnknownReceiverType {
                type_name: receiver_type.to_string(),
            })?;
        let target_receiver_type_id = self
            .instantiated_pointer_type(target_receiver_type)
            .or_else(|| {
                self.instantiated_struct_type(target_receiver_type)
                    .map(|typ| typ.type_id)
            })
            .or_else(|| {
                self.instantiated_alias_type(target_receiver_type)
                    .map(|typ| typ.type_id)
            })
            .ok_or_else(|| CompileError::UnknownReceiverType {
                type_name: target_receiver_type.to_string(),
            })?;
        self.generation
            .instantiated_generics
            .method_function_ids
            .insert(key.clone(), function);
        self.generation
            .instantiated_generics
            .method_sets
            .entry(receiver_type.to_string())
            .or_default()
            .push(method.clone());
        if !path.is_empty() {
            self.generation
                .instantiated_generics
                .promoted_method_bindings
                .insert(
                    key,
                    symbols::PromotedMethodBindingInfo {
                        path: path.clone(),
                        target_receiver_type: target_receiver_type.to_string(),
                    },
                );
        }
        self.generation
            .instantiated_generics
            .methods
            .push(MethodBinding {
                receiver_type: receiver_type_id,
                target_receiver_type: target_receiver_type_id,
                name: method.name.clone(),
                function,
                param_types: method
                    .params
                    .iter()
                    .map(|parameter| parameter.typ.clone())
                    .collect(),
                result_types: method.result_types.clone(),
                promoted_fields: path,
            });
        Ok(true)
    }

    fn promote_embedded_methods_for_instantiated_receiver(
        &mut self,
        type_name: &str,
        receiver_key: &str,
        _receiver_type_id: TypeId,
        pointer_receiver: bool,
        struct_type: &StructTypeDef,
    ) -> Result<(), CompileError> {
        let mut changed = true;
        while changed {
            changed = false;
            for field in &struct_type.fields {
                if !field.embedded {
                    continue;
                }
                self.ensure_runtime_visible_type(&field.typ)?;
                for source_receiver in promoted_source_receivers(&field.typ, pointer_receiver) {
                    for entry in self.receiver_method_entries(&source_receiver) {
                        let mut path = vec![promoted_field_step(
                            &field.typ,
                            &field.name,
                            &source_receiver,
                        )];
                        path.extend(entry.path.clone());
                        changed |= self.register_method_binding(
                            receiver_key,
                            entry.function,
                            &entry.decl,
                            &entry.target_receiver_type,
                            path,
                        )?;
                    }
                }
            }
        }
        let _ = type_name;
        Ok(())
    }

    fn receiver_method_entries(&self, receiver_type: &str) -> Vec<ReceiverMethodEntry> {
        let Some(methods) = self.instantiated_method_set(receiver_type) else {
            return Vec::new();
        };
        let mut entries = Vec::with_capacity(methods.len());
        for method in methods {
            let key = format!("{receiver_type}.{}", method.name);
            let Some(function) = self.instantiated_method_function_id(&key) else {
                continue;
            };
            let (target_receiver_type, path) = self
                .instantiated_promoted_method_binding(&key)
                .map(|binding| (binding.target_receiver_type, binding.path))
                .unwrap_or_else(|| (receiver_type.to_string(), Vec::new()));
            entries.push(ReceiverMethodEntry {
                decl: method,
                function,
                target_receiver_type,
                path,
            });
        }
        entries
    }
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
