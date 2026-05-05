use super::*;
use std::collections::{HashMap, HashSet};

impl FunctionBuilder<'_> {
    pub(crate) fn instantiate_visible_generic_named_type(
        &mut self,
        base_name: &str,
        type_args: &[String],
    ) -> Result<bool, CompileError> {
        if self.env.generic_types.contains_key(base_name) {
            self.instantiate_generic_named_type(base_name, type_args)?;
            return Ok(true);
        }
        let Some((package_selector, symbol)) = base_name.rsplit_once('.') else {
            return Ok(false);
        };
        let Some(package_path) = self.env.imported_packages.get(package_selector).cloned() else {
            return Ok(false);
        };
        let Some(context) = self.imported_generic_package_context(&package_path) else {
            return Ok(false);
        };
        let Some(local_generic_type) = context.generic_types.get(symbol).cloned() else {
            return Ok(false);
        };
        let visible_generic_type = imported_generics::qualify_generic_type(
            &local_generic_type,
            &context.package_selector,
            &context.local_named_types,
        );
        self.validate_type_args_in_context(&visible_generic_type.type_params, type_args)?;

        let key = InstanceKey {
            base_name: base_name.to_string(),
            type_args: type_args.to_vec(),
        };
        let concrete_name = key.mangled_name();
        if self.generation.instantiation_cache.type_id(&key).is_some()
            || self.instantiated_named_type(&concrete_name)
        {
            return Ok(true);
        }

        let local_type_args = context.local_type_args(type_args);
        let local_substitutions =
            build_substitutions(&local_generic_type.type_params, &local_type_args);
        let visible_substitutions =
            build_substitutions(&visible_generic_type.type_params, type_args);
        let local_concrete_name = InstanceKey {
            base_name: symbol.to_string(),
            type_args: local_type_args,
        }
        .mangled_name();

        match &visible_generic_type.kind {
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
                        let field_type = substitute_type_params(&field.typ, &visible_substitutions);
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
                    .expect("placeholder imported generic struct should exist")
                    .fields = concrete_fields;
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
                let concrete_underlying =
                    substitute_type_params(underlying, &visible_substitutions);
                self.ensure_runtime_visible_type(&concrete_underlying)?;
                self.generation
                    .instantiated_generics
                    .aliases
                    .get_mut(&concrete_name)
                    .expect("placeholder imported generic alias should exist")
                    .underlying = concrete_underlying;
            }
            TypeDeclKind::Interface { methods, embeds } => {
                let mut concrete_methods = methods
                    .iter()
                    .map(|method| {
                        let params = method
                            .params
                            .iter()
                            .map(|param| {
                                let typ =
                                    substitute_type_params(&param.typ, &visible_substitutions);
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
                                let typ = substitute_type_params(typ, &visible_substitutions);
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
                    let embed = substitute_type_params(embed, &visible_substitutions);
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
                    .record_type(key.clone(), type_id);
                self.generation.instantiated_generics.interfaces.insert(
                    concrete_name.clone(),
                    InterfaceTypeDef {
                        type_id,
                        methods: concrete_methods,
                    },
                );
            }
        }

        let Some(templates) = context.generic_method_templates.get(symbol).cloned() else {
            return Ok(true);
        };
        let mut local_instantiation_cache = context.instantiation_cache.clone();
        let mut local_instantiated_generics = InstantiatedGenerics::new(
            &context.structs,
            &context.interfaces,
            &context.aliases,
            &context.pointers,
        );
        match &local_generic_type.kind {
            TypeDeclKind::Struct { fields } => {
                let type_id = local_instantiated_generics.allocate_type_id();
                let pointer_type = local_instantiated_generics.allocate_type_id();
                local_instantiated_generics.structs.insert(
                    local_concrete_name.clone(),
                    StructTypeDef {
                        type_id,
                        fields: fields
                            .iter()
                            .map(|field| gowasm_parser::TypeFieldDecl {
                                name: field.name.clone(),
                                typ: substitute_type_params(&field.typ, &local_substitutions),
                                embedded: field.embedded,
                                tag: field.tag.clone(),
                            })
                            .collect(),
                    },
                );
                local_instantiated_generics
                    .pointers
                    .insert(format!("*{local_concrete_name}"), pointer_type);
            }
            TypeDeclKind::Alias { underlying } => {
                let type_id = local_instantiated_generics.allocate_type_id();
                let pointer_type = local_instantiated_generics.allocate_type_id();
                local_instantiated_generics.aliases.insert(
                    local_concrete_name.clone(),
                    AliasTypeDef {
                        type_id,
                        underlying: substitute_type_params(underlying, &local_substitutions),
                    },
                );
                local_instantiated_generics
                    .pointers
                    .insert(format!("*{local_concrete_name}"), pointer_type);
            }
            TypeDeclKind::Interface { methods, embeds } => {
                let mut concrete_methods = methods
                    .iter()
                    .map(|method| InterfaceMethodDecl {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|param| gowasm_parser::Parameter {
                                name: param.name.clone(),
                                typ: substitute_type_params(&param.typ, &local_substitutions),
                                variadic: param.variadic,
                            })
                            .collect(),
                        result_types: method
                            .result_types
                            .iter()
                            .map(|typ| substitute_type_params(typ, &local_substitutions))
                            .collect(),
                    })
                    .collect::<Vec<_>>();
                for embed in embeds {
                    if let Some(embedded) = local_instantiated_generics
                        .interface_type(&substitute_type_params(embed, &local_substitutions))
                        .cloned()
                        .or_else(|| {
                            context
                                .interfaces
                                .get(&substitute_type_params(embed, &local_substitutions))
                                .cloned()
                        })
                    {
                        concrete_methods.extend(embedded.methods);
                    }
                }
                let type_id = local_instantiated_generics.allocate_type_id();
                local_instantiated_generics.interfaces.insert(
                    local_concrete_name.clone(),
                    InterfaceTypeDef {
                        type_id,
                        methods: concrete_methods,
                    },
                );
            }
        }

        let mut concrete_methods = Vec::with_capacity(templates.len());
        let mut local_method_bindings = Vec::with_capacity(templates.len());
        for template in templates {
            let concrete_decl = crate::generic_substitute::instantiate_function_decl(
                &template.decl,
                &local_substitutions,
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
            let cache_name = if receiver_type.starts_with('*') {
                format!(
                    "*{}.{}",
                    Self::imported_package_symbol_key(&context.package_path, symbol),
                    concrete_decl.name
                )
            } else {
                format!(
                    "{}.{}",
                    Self::imported_package_symbol_key(&context.package_path, symbol),
                    concrete_decl.name
                )
            };
            let (_cache_name, function, _created) =
                self.generation.generated_functions.reserve_instance(
                    InstanceKey {
                        base_name: cache_name,
                        type_args: type_args.to_vec(),
                    },
                    self.generation.instantiation_cache,
                );
            let visible_receiver = workspace_artifact_exports::qualify_visible_type(
                &receiver_type,
                &context.package_selector,
                &context.local_named_types,
            );
            let visible_method = InterfaceMethodDecl {
                name: concrete_decl.name.clone(),
                params: concrete_decl
                    .params
                    .iter()
                    .map(|param| gowasm_parser::Parameter {
                        name: param.name.clone(),
                        typ: workspace_artifact_exports::qualify_visible_type(
                            &param.typ,
                            &context.package_selector,
                            &context.local_named_types,
                        ),
                        variadic: param.variadic,
                    })
                    .collect(),
                result_types: concrete_decl
                    .result_types
                    .iter()
                    .map(|typ| {
                        workspace_artifact_exports::qualify_visible_type(
                            typ,
                            &context.package_selector,
                            &context.local_named_types,
                        )
                    })
                    .collect(),
            };
            self.register_direct_method_binding(&visible_receiver, function, &visible_method)?;
            local_method_bindings.push((
                receiver_type,
                InterfaceMethodDecl {
                    name: concrete_decl.name.clone(),
                    params: concrete_decl.params.clone(),
                    result_types: concrete_decl.result_types.clone(),
                },
                function,
            ));
            concrete_methods.push((
                template.imported_packages,
                template.source_path,
                template.spans,
                concrete_decl,
                function,
            ));
        }

        if let Some(struct_type) = self.instantiated_struct_type(&concrete_name) {
            self.promote_embedded_methods_for_instantiated_receiver(
                &concrete_name,
                &concrete_name,
                struct_type.type_id,
                false,
                &struct_type,
            )?;
            if let Some(pointer_type) = self.instantiated_pointer_type(&format!("*{concrete_name}"))
            {
                self.promote_embedded_methods_for_instantiated_receiver(
                    &concrete_name,
                    &format!("*{concrete_name}"),
                    pointer_type,
                    true,
                    &struct_type,
                )?;
            }
        }

        {
            let mut local_builder = FunctionBuilder {
                emitter: EmitterState::default(),
                env: CompilerEnvironment::new(
                    ImportContext {
                        imported_packages: HashMap::new(),
                        imported_package_tables: context.imported_bindings.as_tables(),
                    },
                    SymbolTables {
                        function_ids: &context.function_ids,
                        function_result_types: &context.function_result_types,
                        function_types: &context.function_types,
                        variadic_functions: &context.variadic_functions,
                        method_function_ids: &context.method_function_ids,
                        promoted_method_bindings: &context.promoted_method_bindings,
                        globals: &context.globals,
                        method_sets: &context.method_sets,
                    },
                    TypeContext {
                        generic_functions: &context.generic_functions,
                        generic_types: &context.generic_types,
                        generic_function_templates: &context.generic_function_templates,
                        generic_method_templates: &context.generic_method_templates,
                    },
                    RuntimeMetadataContext {
                        struct_types: &context.structs,
                        pointer_types: &context.pointers,
                        interface_types: &context.interfaces,
                        alias_types: &context.aliases,
                    },
                ),
                generation: GenerationState {
                    instantiation_cache: &mut local_instantiation_cache,
                    generated_functions: self.generation.generated_functions,
                    instantiated_generics: &mut local_instantiated_generics,
                    generic_instance_namespace: Some(context.package_path.clone()),
                },
                scopes: ScopeStack {
                    scopes: vec![HashMap::new()],
                    captured_by_ref: HashSet::new(),
                    const_scopes: vec![HashSet::new()],
                    const_value_scopes: vec![HashMap::new()],
                    type_scopes: vec![HashMap::new()],
                },
                control: ControlFlowContext {
                    in_package_init: false,
                    current_result_types: Vec::new(),
                    current_result_names: Vec::new(),
                    break_scopes: Vec::new(),
                    loops: Vec::new(),
                    pending_label: None,
                },
            };
            for (receiver_type, method, function) in &local_method_bindings {
                local_builder.register_direct_method_binding(receiver_type, *function, method)?;
            }
            if let Some(struct_type) = local_builder.instantiated_struct_type(&local_concrete_name)
            {
                local_builder.promote_embedded_methods_for_instantiated_receiver(
                    &local_concrete_name,
                    &local_concrete_name,
                    struct_type.type_id,
                    false,
                    &struct_type,
                )?;
                if let Some(pointer_type) =
                    local_builder.instantiated_pointer_type(&format!("*{local_concrete_name}"))
                {
                    local_builder.promote_embedded_methods_for_instantiated_receiver(
                        &local_concrete_name,
                        &format!("*{local_concrete_name}"),
                        pointer_type,
                        true,
                        &struct_type,
                    )?;
                }
            }
        }

        for (imported_packages, source_path, spans, concrete_decl, function) in concrete_methods {
            let compiled = program::compile_function_with_namespace(
                imported_packages,
                &source_path,
                Some(&spans),
                &concrete_decl,
                context.imported_bindings.as_tables(),
                &context.function_ids,
                &context.function_result_types,
                &context.function_types,
                &context.variadic_functions,
                &context.generic_functions,
                &context.generic_types,
                &context.generic_function_templates,
                &context.generic_method_templates,
                &mut local_instantiation_cache,
                self.generation.generated_functions,
                &mut local_instantiated_generics,
                &context.method_function_ids,
                &context.promoted_method_bindings,
                &context.structs,
                &context.pointers,
                &context.interfaces,
                &context.aliases,
                &context.method_sets,
                &context.globals,
                Some(context.package_path.clone()),
            )?;
            self.generation.generated_functions.fill(function, compiled);
        }
        Ok(true)
    }
}
