use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::*;
use crate::generics::ResolvedGenericCall;
use crate::types::{build_substitutions, GenericMethodDef, InstanceKey, TypeParamDef};
use gowasm_parser::TypeDeclKind;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ImportedBindingsSnapshot {
    pub(crate) function_ids: HashMap<String, usize>,
    pub(crate) generic_function_instances: HashMap<InstanceKey, usize>,
    pub(crate) function_result_types: HashMap<String, Vec<String>>,
    pub(crate) function_types: HashMap<String, String>,
    pub(crate) variadic_functions: HashSet<String>,
    pub(crate) globals: HashMap<String, GlobalBinding>,
    pub(crate) structs: HashMap<String, StructTypeDef>,
    pub(crate) interfaces: HashMap<String, InterfaceTypeDef>,
    pub(crate) pointers: HashMap<String, TypeId>,
    pub(crate) aliases: HashMap<String, AliasTypeDef>,
    pub(crate) method_function_ids: HashMap<String, usize>,
    pub(crate) promoted_method_bindings: HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(crate) method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
    pub(crate) generic_package_contexts: HashMap<String, Arc<ImportedGenericPackageContext>>,
}

impl ImportedBindingsSnapshot {
    pub(crate) fn as_tables(&self) -> ImportedPackageTables<'_> {
        ImportedPackageTables {
            function_ids: &self.function_ids,
            generic_function_instances: &self.generic_function_instances,
            function_result_types: &self.function_result_types,
            function_types: &self.function_types,
            variadic_functions: &self.variadic_functions,
            globals: &self.globals,
            structs: &self.structs,
            interfaces: &self.interfaces,
            pointers: &self.pointers,
            aliases: &self.aliases,
            method_function_ids: &self.method_function_ids,
            promoted_method_bindings: &self.promoted_method_bindings,
            method_sets: &self.method_sets,
            generic_package_contexts: &self.generic_package_contexts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedGenericPackageContext {
    pub(crate) package_path: String,
    pub(crate) package_selector: String,
    pub(crate) local_named_types: HashSet<String>,
    pub(crate) imported_bindings: ImportedBindingsSnapshot,
    pub(crate) visible_generic_functions: HashMap<String, GenericFunctionDef>,
    pub(crate) generic_functions: HashMap<String, GenericFunctionDef>,
    pub(crate) generic_types: HashMap<String, GenericTypeDef>,
    pub(crate) generic_function_templates: HashMap<String, GenericFunctionTemplate>,
    pub(crate) generic_method_templates:
        HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    pub(crate) instantiation_cache: InstantiationCache,
    pub(crate) function_ids: HashMap<String, usize>,
    pub(crate) function_result_types: HashMap<String, Vec<String>>,
    pub(crate) function_types: HashMap<String, String>,
    pub(crate) variadic_functions: HashSet<String>,
    pub(crate) globals: HashMap<String, GlobalBinding>,
    pub(crate) structs: HashMap<String, StructTypeDef>,
    pub(crate) interfaces: HashMap<String, InterfaceTypeDef>,
    pub(crate) pointers: HashMap<String, TypeId>,
    pub(crate) aliases: HashMap<String, AliasTypeDef>,
    pub(crate) method_function_ids: HashMap<String, usize>,
    pub(crate) promoted_method_bindings: HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(crate) method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
}

impl ImportedGenericPackageContext {
    pub(crate) fn visible_generic_function(&self, symbol: &str) -> Option<&GenericFunctionDef> {
        self.visible_generic_functions.get(symbol)
    }

    pub(crate) fn local_type_args(&self, type_args: &[String]) -> Vec<String> {
        type_args
            .iter()
            .map(|typ| {
                workspace_artifact_exports::dequalify_visible_type(
                    typ,
                    &self.package_selector,
                    &self.local_named_types,
                )
            })
            .collect()
    }
}

pub(crate) fn qualify_generic_type(
    generic_type: &GenericTypeDef,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> GenericTypeDef {
    GenericTypeDef {
        type_params: generic_type
            .type_params
            .iter()
            .map(|type_param| TypeParamDef {
                name: type_param.name.clone(),
                constraint: workspace_artifact_exports::qualify_type_constraint(
                    &type_param.constraint,
                    package_selector,
                    local_named_types,
                ),
            })
            .collect(),
        kind: match &generic_type.kind {
            TypeDeclKind::Struct { fields } => TypeDeclKind::Struct {
                fields: fields
                    .iter()
                    .map(|field| gowasm_parser::TypeFieldDecl {
                        name: field.name.clone(),
                        typ: workspace_artifact_exports::qualify_visible_type(
                            &field.typ,
                            package_selector,
                            local_named_types,
                        ),
                        embedded: field.embedded,
                        tag: field.tag.clone(),
                    })
                    .collect(),
            },
            TypeDeclKind::Alias { underlying } => TypeDeclKind::Alias {
                underlying: workspace_artifact_exports::qualify_visible_type(
                    underlying,
                    package_selector,
                    local_named_types,
                ),
            },
            TypeDeclKind::Interface { methods, embeds } => TypeDeclKind::Interface {
                methods: methods
                    .iter()
                    .map(|method| InterfaceMethodDecl {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|param| gowasm_parser::Parameter {
                                name: param.name.clone(),
                                typ: workspace_artifact_exports::qualify_visible_type(
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
                                workspace_artifact_exports::qualify_visible_type(
                                    typ,
                                    package_selector,
                                    local_named_types,
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                embeds: embeds
                    .iter()
                    .map(|embed| {
                        workspace_artifact_exports::qualify_visible_type(
                            embed,
                            package_selector,
                            local_named_types,
                        )
                    })
                    .collect(),
            },
        },
        methods: generic_type
            .methods
            .iter()
            .map(|method| GenericMethodDef {
                name: method.name.clone(),
                params: method
                    .params
                    .iter()
                    .map(|param| gowasm_parser::Parameter {
                        name: param.name.clone(),
                        typ: workspace_artifact_exports::qualify_visible_type(
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
                        workspace_artifact_exports::qualify_visible_type(
                            typ,
                            package_selector,
                            local_named_types,
                        )
                    })
                    .collect(),
                pointer_receiver: method.pointer_receiver,
            })
            .collect(),
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn imported_generic_function(
        &self,
        package: &str,
        symbol: &str,
    ) -> Option<&GenericFunctionDef> {
        self.env
            .imported_package_tables
            .generic_package_contexts
            .get(package)
            .and_then(|context| context.visible_generic_function(symbol))
    }

    pub(super) fn imported_generic_package_context(
        &self,
        package: &str,
    ) -> Option<Arc<ImportedGenericPackageContext>> {
        self.env
            .imported_package_tables
            .generic_package_contexts
            .get(package)
            .cloned()
    }

    pub(super) fn imported_generic_function_instance(
        &self,
        package_path: &str,
        symbol: &str,
        type_args: &[String],
    ) -> Option<usize> {
        self.env
            .imported_package_tables
            .generic_function_instances
            .get(&InstanceKey {
                base_name: Self::imported_package_symbol_key(package_path, symbol),
                type_args: type_args.to_vec(),
            })
            .copied()
    }

    pub(super) fn namespaced_generic_instance_base_name(&self, name: &str) -> String {
        self.generation
            .generic_instance_namespace
            .as_ref()
            .map(|namespace| Self::imported_package_symbol_key(namespace, name))
            .unwrap_or_else(|| name.to_string())
    }

    pub(super) fn instantiate_imported_generic_function(
        &mut self,
        package_path: &str,
        resolved: &ResolvedGenericCall,
    ) -> Result<usize, CompileError> {
        let Some(context) = self.imported_generic_package_context(package_path) else {
            return Err(CompileError::UnknownFunction {
                name: format!("{package_path}.{}", resolved.name),
            });
        };
        let Some(template) = context
            .generic_function_templates
            .get(&resolved.name)
            .cloned()
        else {
            return Err(CompileError::UnknownFunction {
                name: format!("{package_path}.{}", resolved.name),
            });
        };
        let Some(generic_function) = context.generic_functions.get(&resolved.name).cloned() else {
            return Err(CompileError::UnknownFunction {
                name: format!("{package_path}.{}", resolved.name),
            });
        };

        if let Some(function) = self.imported_generic_function_instance(
            package_path,
            &resolved.name,
            &resolved.type_args,
        ) {
            return Ok(function);
        }

        let key = InstanceKey {
            base_name: Self::imported_package_symbol_key(package_path, &resolved.name),
            type_args: resolved.type_args.clone(),
        };
        let (concrete_name, function, created) = self
            .generation
            .generated_functions
            .reserve_instance(key, self.generation.instantiation_cache);
        if !created {
            return Ok(function);
        }

        let local_type_args = context.local_type_args(&resolved.type_args);
        let substitutions = build_substitutions(&generic_function.type_params, &local_type_args);
        let concrete_decl = crate::generic_substitute::instantiate_function_decl(
            &template.decl,
            &substitutions,
            concrete_name,
        );
        let mut instantiation_cache = context.instantiation_cache.clone();
        let mut instantiated_generics = generic_instances::InstantiatedGenerics::new(
            &context.structs,
            &context.interfaces,
            &context.aliases,
            &context.pointers,
        );
        let compiled = program::compile_function_with_namespace(
            template.imported_packages,
            &template.source_path,
            Some(&template.spans),
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
            &mut instantiation_cache,
            self.generation.generated_functions,
            &mut instantiated_generics,
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
        Ok(function)
    }
}
