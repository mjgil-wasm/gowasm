use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::*;

#[allow(dead_code)]
pub(crate) const PACKAGE_ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SerializedPackageArtifact {
    pub(crate) schema_version: u32,
    pub(crate) import_path: String,
    pub(crate) dependencies: ArtifactDependencyEdgesSchema,
    pub(crate) runtime_metadata: ArtifactRuntimeMetadataSchema,
    pub(crate) diagnostics: ArtifactDiagnosticsSchema,
    pub(crate) exports: ArtifactExportsSchema,
    pub(crate) generics: Option<ArtifactGenericsSchema>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactDependencyEdgesSchema {
    pub(crate) source_paths: Vec<String>,
    pub(crate) direct_dependencies: Vec<String>,
    pub(crate) reverse_dependencies: Vec<String>,
    pub(crate) source_origin: Option<ArtifactSourceOriginSchema>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ArtifactSourceOriginSchema {
    Workspace,
    ModuleCache {
        module_path: String,
        version: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactRuntimeMetadataSchema {
    pub(crate) function_start: usize,
    pub(crate) functions: Vec<Function>,
    pub(crate) methods: Vec<gowasm_vm::MethodBinding>,
    pub(crate) global_start: usize,
    pub(crate) global_count: usize,
    pub(crate) user_type_offset: u32,
    pub(crate) user_type_span: u32,
    pub(crate) type_inventory: Vec<ArtifactRuntimeTypeEntrySchema>,
    pub(crate) entry_function: Option<usize>,
    pub(crate) package_init_function: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactRuntimeTypeEntrySchema {
    pub(crate) type_id: TypeId,
    pub(crate) info: gowasm_vm::RuntimeTypeInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactDiagnosticsSchema {
    pub(crate) debug_infos: Vec<FunctionDebugInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactExportsSchema {
    pub(crate) qualified_function_ids: HashMap<String, usize>,
    pub(crate) qualified_generic_function_instances: Vec<ArtifactGenericFunctionInstanceSchema>,
    pub(crate) qualified_function_result_types: HashMap<String, Vec<String>>,
    pub(crate) qualified_function_types: HashMap<String, String>,
    pub(crate) qualified_variadic_functions: HashSet<String>,
    pub(crate) qualified_globals: HashMap<String, GlobalBinding>,
    pub(crate) qualified_structs: HashMap<String, StructTypeDef>,
    pub(crate) qualified_interfaces: HashMap<String, InterfaceTypeDef>,
    pub(crate) qualified_pointers: HashMap<String, TypeId>,
    pub(crate) qualified_aliases: HashMap<String, AliasTypeDef>,
    pub(crate) qualified_method_function_ids: HashMap<String, usize>,
    pub(crate) qualified_promoted_method_bindings:
        HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(crate) qualified_method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactGenericFunctionInstanceSchema {
    pub(crate) key: InstanceKey,
    pub(crate) function: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactGenericsSchema {
    pub(crate) package_selector: String,
    pub(crate) local_named_types: HashSet<String>,
    pub(crate) visible_generic_functions: HashMap<String, GenericFunctionDef>,
    pub(crate) generic_functions: HashMap<String, GenericFunctionDef>,
    pub(crate) generic_types: HashMap<String, GenericTypeDef>,
    pub(crate) generic_function_templates: HashMap<String, ArtifactGenericTemplateSourceSchema>,
    pub(crate) generic_method_templates: HashMap<String, Vec<ArtifactGenericTemplateSourceSchema>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactGenericTemplateSourceSchema {
    pub(crate) source_path: String,
    pub(crate) source_text: String,
    pub(crate) spans: FunctionSourceSpans,
    pub(crate) imported_packages: HashMap<String, String>,
}

#[allow(dead_code)]
impl SerializedPackageArtifact {
    pub(crate) fn from_artifact(artifact: &CompiledPackageArtifact) -> Self {
        Self {
            schema_version: PACKAGE_ARTIFACT_SCHEMA_VERSION,
            import_path: artifact.import_path.clone(),
            dependencies: ArtifactDependencyEdgesSchema::from_dependency_edges(
                &artifact.dependency_edges,
            ),
            runtime_metadata: ArtifactRuntimeMetadataSchema::from_artifact(artifact),
            diagnostics: ArtifactDiagnosticsSchema {
                debug_infos: artifact.debug_infos.clone(),
            },
            exports: ArtifactExportsSchema::from_artifact(artifact),
            generics: artifact
                .generic_package_context
                .as_ref()
                .map(|context| ArtifactGenericsSchema::from_artifact(artifact, context)),
        }
    }
}

#[allow(dead_code)]
impl ArtifactDependencyEdgesSchema {
    fn from_dependency_edges(edges: &ArtifactDependencyEdges) -> Self {
        Self {
            source_paths: edges.source_paths.clone(),
            direct_dependencies: edges.direct_dependencies.clone(),
            reverse_dependencies: edges.reverse_dependencies.clone(),
            source_origin: edges.source_origin.as_ref().map(|origin| match origin {
                ArtifactSourceOrigin::Workspace => ArtifactSourceOriginSchema::Workspace,
                ArtifactSourceOrigin::ModuleCache {
                    module_path,
                    version,
                } => ArtifactSourceOriginSchema::ModuleCache {
                    module_path: module_path.clone(),
                    version: version.clone(),
                },
            }),
        }
    }
}

#[allow(dead_code)]
impl ArtifactRuntimeMetadataSchema {
    fn from_artifact(artifact: &CompiledPackageArtifact) -> Self {
        Self {
            function_start: artifact.function_start,
            functions: artifact.functions.clone(),
            methods: artifact.methods.clone(),
            global_start: artifact.global_start,
            global_count: artifact.global_count,
            user_type_offset: artifact.user_type_offset,
            user_type_span: artifact.user_type_span,
            type_inventory: artifact
                .type_inventory
                .types_by_id
                .iter()
                .map(|(type_id, info)| ArtifactRuntimeTypeEntrySchema {
                    type_id: *type_id,
                    info: info.clone(),
                })
                .collect(),
            entry_function: artifact.entry_function,
            package_init_function: artifact.package_init_function,
        }
    }
}

#[allow(dead_code)]
impl ArtifactExportsSchema {
    fn from_artifact(artifact: &CompiledPackageArtifact) -> Self {
        Self {
            qualified_function_ids: artifact.qualified_function_ids.clone(),
            qualified_generic_function_instances: artifact
                .qualified_generic_function_instances
                .iter()
                .map(|(key, function)| ArtifactGenericFunctionInstanceSchema {
                    key: key.clone(),
                    function: *function,
                })
                .collect(),
            qualified_function_result_types: artifact.qualified_function_result_types.clone(),
            qualified_function_types: artifact.qualified_function_types.clone(),
            qualified_variadic_functions: artifact.qualified_variadic_functions.clone(),
            qualified_globals: artifact.qualified_globals.clone(),
            qualified_structs: artifact.qualified_structs.clone(),
            qualified_interfaces: artifact.qualified_interfaces.clone(),
            qualified_pointers: artifact.qualified_pointers.clone(),
            qualified_aliases: artifact.qualified_aliases.clone(),
            qualified_method_function_ids: artifact.qualified_method_function_ids.clone(),
            qualified_promoted_method_bindings: artifact.qualified_promoted_method_bindings.clone(),
            qualified_method_sets: artifact.qualified_method_sets.clone(),
        }
    }
}

#[allow(dead_code)]
impl ArtifactGenericsSchema {
    fn from_artifact(
        artifact: &CompiledPackageArtifact,
        context: &std::sync::Arc<imported_generics::ImportedGenericPackageContext>,
    ) -> Self {
        Self {
            package_selector: context.package_selector.clone(),
            local_named_types: context.local_named_types.clone(),
            visible_generic_functions: context.visible_generic_functions.clone(),
            generic_functions: context.generic_functions.clone(),
            generic_types: context.generic_types.clone(),
            generic_function_templates: artifact
                .generic_function_template_sources
                .iter()
                .map(|(name, template)| {
                    (
                        name.clone(),
                        ArtifactGenericTemplateSourceSchema::from_template_source(template),
                    )
                })
                .collect(),
            generic_method_templates: artifact
                .generic_method_template_sources
                .iter()
                .map(|(name, templates)| {
                    (
                        name.clone(),
                        templates
                            .iter()
                            .map(ArtifactGenericTemplateSourceSchema::from_template_source)
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

#[allow(dead_code)]
impl ArtifactGenericTemplateSourceSchema {
    fn from_template_source(template: &ArtifactGenericTemplateSource) -> Self {
        Self {
            source_path: template.source_path.clone(),
            source_text: template.source_text.clone(),
            spans: template.spans.clone(),
            imported_packages: template.imported_packages.clone(),
        }
    }
}
