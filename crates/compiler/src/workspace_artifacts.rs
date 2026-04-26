use super::*;
use crate::types::InstanceKey;

#[path = "workspace_artifact_compile_phases.rs"]
mod compile_phases;
#[path = "workspace_artifact_metadata.rs"]
mod metadata;
#[path = "package_artifact_schema.rs"]
pub(crate) mod package_artifact_schema;

use metadata::{ArtifactDependencyEdges, ArtifactGenericTemplateSource, ArtifactSourceOrigin};

#[derive(Debug, Clone, Default)]
struct QualifiedPackageBindings {
    function_ids: HashMap<String, usize>,
    generic_function_instances: HashMap<InstanceKey, usize>,
    function_result_types: HashMap<String, Vec<String>>,
    function_types: HashMap<String, String>,
    variadic_functions: HashSet<String>,
    globals: HashMap<String, GlobalBinding>,
    structs: HashMap<String, StructTypeDef>,
    interfaces: HashMap<String, InterfaceTypeDef>,
    pointers: HashMap<String, TypeId>,
    aliases: HashMap<String, AliasTypeDef>,
    method_function_ids: HashMap<String, usize>,
    promoted_method_bindings: HashMap<String, symbols::PromotedMethodBindingInfo>,
    method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
    generic_package_contexts:
        HashMap<String, std::sync::Arc<imported_generics::ImportedGenericPackageContext>>,
}

impl QualifiedPackageBindings {
    fn snapshot(&self) -> imported_generics::ImportedBindingsSnapshot {
        imported_generics::ImportedBindingsSnapshot {
            function_ids: self.function_ids.clone(),
            generic_function_instances: self.generic_function_instances.clone(),
            function_result_types: self.function_result_types.clone(),
            function_types: self.function_types.clone(),
            variadic_functions: self.variadic_functions.clone(),
            globals: self.globals.clone(),
            structs: self.structs.clone(),
            interfaces: self.interfaces.clone(),
            pointers: self.pointers.clone(),
            aliases: self.aliases.clone(),
            method_function_ids: self.method_function_ids.clone(),
            promoted_method_bindings: self.promoted_method_bindings.clone(),
            method_sets: self.method_sets.clone(),
            generic_package_contexts: self.generic_package_contexts.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledPackageArtifact {
    import_path: String,
    function_start: usize,
    functions: Vec<Function>,
    debug_infos: Vec<FunctionDebugInfo>,
    methods: Vec<gowasm_vm::MethodBinding>,
    global_start: usize,
    global_count: usize,
    user_type_offset: u32,
    user_type_span: u32,
    type_inventory: gowasm_vm::ProgramTypeInventory,
    entry_function: Option<usize>,
    package_init_function: Option<usize>,
    qualified_function_ids: HashMap<String, usize>,
    qualified_generic_function_instances: HashMap<InstanceKey, usize>,
    qualified_function_result_types: HashMap<String, Vec<String>>,
    qualified_function_types: HashMap<String, String>,
    qualified_variadic_functions: HashSet<String>,
    qualified_globals: HashMap<String, GlobalBinding>,
    qualified_structs: HashMap<String, StructTypeDef>,
    qualified_interfaces: HashMap<String, InterfaceTypeDef>,
    qualified_pointers: HashMap<String, TypeId>,
    qualified_aliases: HashMap<String, AliasTypeDef>,
    qualified_method_function_ids: HashMap<String, usize>,
    qualified_promoted_method_bindings: HashMap<String, symbols::PromotedMethodBindingInfo>,
    qualified_method_sets: HashMap<String, Vec<InterfaceMethodDecl>>,
    dependency_edges: ArtifactDependencyEdges,
    generic_function_template_sources: HashMap<String, ArtifactGenericTemplateSource>,
    generic_method_template_sources: HashMap<String, Vec<ArtifactGenericTemplateSource>>,
    generic_package_context:
        Option<std::sync::Arc<imported_generics::ImportedGenericPackageContext>>,
}

#[path = "workspace_artifact_rebase.rs"]
mod rebase;

use rebase::ArtifactRebaseMap;

impl CompiledPackageArtifact {
    #[allow(dead_code)]
    pub(crate) fn serialized_schema(&self) -> package_artifact_schema::SerializedPackageArtifact {
        package_artifact_schema::SerializedPackageArtifact::from_artifact(self)
    }

    fn export_surface_matches(&self, previous: &Self) -> bool {
        map_preserves_keys(
            &self.qualified_function_ids,
            &previous.qualified_function_ids,
        ) && map_preserves_keys(
            &self.qualified_generic_function_instances,
            &previous.qualified_generic_function_instances,
        ) && map_preserves_values(
            &self.qualified_function_result_types,
            &previous.qualified_function_result_types,
            PartialEq::eq,
        ) && map_preserves_values(
            &self.qualified_function_types,
            &previous.qualified_function_types,
            PartialEq::eq,
        ) && set_preserves_values(
            &self.qualified_variadic_functions,
            &previous.qualified_variadic_functions,
        ) && map_preserves_values(&self.qualified_globals, &previous.qualified_globals, {
            |current, previous| current.global_shape_matches(previous)
        }) && map_preserves_values(&self.qualified_structs, &previous.qualified_structs, {
            |current, previous| current.struct_shape_matches(previous)
        }) && map_preserves_values(
            &self.qualified_interfaces,
            &previous.qualified_interfaces,
            |current, previous| current.interface_shape_matches(previous),
        ) && map_preserves_keys(&self.qualified_pointers, &previous.qualified_pointers)
            && map_preserves_values(&self.qualified_aliases, &previous.qualified_aliases, {
                |current, previous| current.alias_shape_matches(previous)
            })
            && map_preserves_keys(
                &self.qualified_method_function_ids,
                &previous.qualified_method_function_ids,
            )
            && map_preserves_values(
                &self.qualified_promoted_method_bindings,
                &previous.qualified_promoted_method_bindings,
                PartialEq::eq,
            )
            && map_preserves_values(
                &self.qualified_method_sets,
                &previous.qualified_method_sets,
                PartialEq::eq,
            )
            && option_preserves_value(
                &self.generic_package_context,
                &previous.generic_package_context,
                |current, previous| current.context_shape_matches(previous),
            )
    }

    fn extend_imported_bindings(&self, bindings: &mut QualifiedPackageBindings) {
        bindings
            .function_ids
            .extend(self.qualified_function_ids.clone());
        bindings
            .generic_function_instances
            .extend(self.qualified_generic_function_instances.clone());
        bindings
            .function_result_types
            .extend(self.qualified_function_result_types.clone());
        bindings
            .function_types
            .extend(self.qualified_function_types.clone());
        bindings
            .variadic_functions
            .extend(self.qualified_variadic_functions.clone());
        bindings.globals.extend(self.qualified_globals.clone());
        bindings.structs.extend(self.qualified_structs.clone());
        bindings
            .interfaces
            .extend(self.qualified_interfaces.clone());
        bindings.pointers.extend(self.qualified_pointers.clone());
        bindings.aliases.extend(self.qualified_aliases.clone());
        bindings
            .method_function_ids
            .extend(self.qualified_method_function_ids.clone());
        bindings
            .promoted_method_bindings
            .extend(self.qualified_promoted_method_bindings.clone());
        bindings
            .method_sets
            .extend(self.qualified_method_sets.clone());
        if let Some(context) = &self.generic_package_context {
            bindings
                .generic_package_contexts
                .insert(self.import_path.clone(), context.clone());
        }
    }

    fn refresh_reused_imported_contexts(&mut self, bindings: &QualifiedPackageBindings) {
        if let Some(context) = &mut self.generic_package_context {
            std::sync::Arc::make_mut(context).imported_bindings = bindings.snapshot();
        }
    }
}

fn map_preserves_keys<K, V>(current: &HashMap<K, V>, previous: &HashMap<K, V>) -> bool
where
    K: std::cmp::Eq + std::hash::Hash,
{
    previous.keys().all(|key| current.contains_key(key))
}

fn map_preserves_values<K, V, F>(
    current: &HashMap<K, V>,
    previous: &HashMap<K, V>,
    matches: F,
) -> bool
where
    K: std::cmp::Eq + std::hash::Hash,
    F: Fn(&V, &V) -> bool,
{
    previous.iter().all(|(key, previous_value)| {
        current
            .get(key)
            .is_some_and(|value| matches(value, previous_value))
    })
}

fn set_preserves_values<T>(current: &HashSet<T>, previous: &HashSet<T>) -> bool
where
    T: std::cmp::Eq + std::hash::Hash,
{
    previous.iter().all(|value| current.contains(value))
}

fn option_preserves_value<T, F>(current: &Option<T>, previous: &Option<T>, matches: F) -> bool
where
    F: Fn(&T, &T) -> bool,
{
    match (current, previous) {
        (_, None) => true,
        (Some(current), Some(previous)) => matches(current, previous),
        (None, Some(_)) => false,
    }
}

trait ReusableExportShape {
    fn global_shape_matches(&self, previous: &Self) -> bool {
        let _ = previous;
        false
    }

    fn struct_shape_matches(&self, previous: &Self) -> bool {
        let _ = previous;
        false
    }

    fn interface_shape_matches(&self, previous: &Self) -> bool {
        let _ = previous;
        false
    }

    fn alias_shape_matches(&self, previous: &Self) -> bool {
        let _ = previous;
        false
    }

    fn context_shape_matches(&self, previous: &Self) -> bool {
        let _ = previous;
        false
    }
}

impl ReusableExportShape for GlobalBinding {
    fn global_shape_matches(&self, previous: &Self) -> bool {
        self.typ == previous.typ
            && self.is_const == previous.is_const
            && self.const_value == previous.const_value
    }
}

impl ReusableExportShape for StructTypeDef {
    fn struct_shape_matches(&self, previous: &Self) -> bool {
        self.fields == previous.fields
    }
}

impl ReusableExportShape for InterfaceTypeDef {
    fn interface_shape_matches(&self, previous: &Self) -> bool {
        self.methods == previous.methods
    }
}

impl ReusableExportShape for AliasTypeDef {
    fn alias_shape_matches(&self, previous: &Self) -> bool {
        self.underlying == previous.underlying
    }
}

impl ReusableExportShape for imported_generics::ImportedGenericPackageContext {
    fn context_shape_matches(&self, previous: &Self) -> bool {
        self.package_path == previous.package_path
            && self.package_selector == previous.package_selector
            && self.local_named_types == previous.local_named_types
            && self
                .imported_bindings
                .snapshot_shape_matches(&previous.imported_bindings)
            && map_preserves_values(
                &self.visible_generic_functions,
                &previous.visible_generic_functions,
                PartialEq::eq,
            )
            && map_preserves_values(
                &self.generic_functions,
                &previous.generic_functions,
                PartialEq::eq,
            )
            && map_preserves_values(&self.generic_types, &previous.generic_types, PartialEq::eq)
            && map_preserves_values(
                &self.generic_function_templates,
                &previous.generic_function_templates,
                generic_function_template_shape_matches,
            )
            && map_preserves_values(
                &self.generic_method_templates,
                &previous.generic_method_templates,
                generic_method_templates_shape_match,
            )
            && self
                .instantiation_cache
                .cache_shape_matches(&previous.instantiation_cache)
            && map_preserves_keys(&self.function_ids, &previous.function_ids)
            && map_preserves_values(
                &self.function_result_types,
                &previous.function_result_types,
                PartialEq::eq,
            )
            && map_preserves_values(
                &self.function_types,
                &previous.function_types,
                PartialEq::eq,
            )
            && set_preserves_values(&self.variadic_functions, &previous.variadic_functions)
            && map_preserves_values(&self.globals, &previous.globals, |current, previous| {
                current.global_shape_matches(previous)
            })
            && map_preserves_values(&self.structs, &previous.structs, |current, previous| {
                current.struct_shape_matches(previous)
            })
            && map_preserves_values(
                &self.interfaces,
                &previous.interfaces,
                |current, previous| current.interface_shape_matches(previous),
            )
            && map_preserves_keys(&self.pointers, &previous.pointers)
            && map_preserves_values(&self.aliases, &previous.aliases, |current, previous| {
                current.alias_shape_matches(previous)
            })
            && map_preserves_keys(&self.method_function_ids, &previous.method_function_ids)
            && map_preserves_values(
                &self.promoted_method_bindings,
                &previous.promoted_method_bindings,
                PartialEq::eq,
            )
            && map_preserves_values(&self.method_sets, &previous.method_sets, PartialEq::eq)
    }
}

trait ImportedSnapshotShape {
    fn snapshot_shape_matches(&self, previous: &Self) -> bool;
}

impl ImportedSnapshotShape for imported_generics::ImportedBindingsSnapshot {
    fn snapshot_shape_matches(&self, previous: &Self) -> bool {
        map_preserves_keys(&self.function_ids, &previous.function_ids)
            && map_preserves_keys(
                &self.generic_function_instances,
                &previous.generic_function_instances,
            )
            && map_preserves_values(
                &self.function_result_types,
                &previous.function_result_types,
                PartialEq::eq,
            )
            && map_preserves_values(
                &self.function_types,
                &previous.function_types,
                PartialEq::eq,
            )
            && set_preserves_values(&self.variadic_functions, &previous.variadic_functions)
            && map_preserves_values(&self.globals, &previous.globals, |current, previous| {
                current.global_shape_matches(previous)
            })
            && map_preserves_values(&self.structs, &previous.structs, |current, previous| {
                current.struct_shape_matches(previous)
            })
            && map_preserves_values(
                &self.interfaces,
                &previous.interfaces,
                |current, previous| current.interface_shape_matches(previous),
            )
            && map_preserves_keys(&self.pointers, &previous.pointers)
            && map_preserves_values(&self.aliases, &previous.aliases, |current, previous| {
                current.alias_shape_matches(previous)
            })
            && map_preserves_keys(&self.method_function_ids, &previous.method_function_ids)
            && map_preserves_values(
                &self.promoted_method_bindings,
                &previous.promoted_method_bindings,
                PartialEq::eq,
            )
            && map_preserves_values(&self.method_sets, &previous.method_sets, PartialEq::eq)
            && map_preserves_values(
                &self.generic_package_contexts,
                &previous.generic_package_contexts,
                |current, previous| current.context_shape_matches(previous),
            )
    }
}

trait InstantiationCacheShape {
    fn cache_shape_matches(&self, previous: &Self) -> bool;
}

impl InstantiationCacheShape for InstantiationCache {
    fn cache_shape_matches(&self, previous: &Self) -> bool {
        map_preserves_values(
            &self.function_instances,
            &previous.function_instances,
            PartialEq::eq,
        ) && map_preserves_keys(&self.type_instances, &previous.type_instances)
    }
}

fn generic_function_template_shape_matches(
    current: &GenericFunctionTemplate,
    previous: &GenericFunctionTemplate,
) -> bool {
    current.decl.name == previous.decl.name
        && current.decl.type_params == previous.decl.type_params
        && current.decl.params == previous.decl.params
        && current.decl.result_types == previous.decl.result_types
}

#[allow(clippy::ptr_arg)]
fn generic_method_templates_shape_match(
    current: &Vec<generic_instances::GenericMethodTemplate>,
    previous: &Vec<generic_instances::GenericMethodTemplate>,
) -> bool {
    current.len() == previous.len()
        && current.iter().zip(previous).all(|(current, previous)| {
            current.decl.name == previous.decl.name
                && current.decl.receiver == previous.decl.receiver
                && current.decl.type_params == previous.decl.type_params
                && current.decl.params == previous.decl.params
                && current.decl.result_types == previous.decl.result_types
        })
}

pub(super) fn compile_resolved_workspace(
    resolved: import_resolution::ResolvedWorkspace,
    sources: &[SourceInput<'_>],
) -> Result<CompiledWorkspace, CompileError> {
    let import_resolution::ResolvedWorkspace {
        entry_import_path,
        entry_package_name,
        package_files,
        rebuild_graph,
    } = resolved;
    let mut imported_bindings = QualifiedPackageBindings::default();
    let mut next_user_type_offset = 0u32;
    let mut next_global_index = 0usize;
    let mut artifacts: Vec<CompiledPackageArtifact> =
        Vec::with_capacity(rebuild_graph.dependency_order().len());
    let source_lookup = sources
        .iter()
        .map(|source| (source.path, source.source))
        .collect::<HashMap<_, _>>();

    for import_path in rebuild_graph.dependency_order() {
        let package_files = package_files
            .get(import_path)
            .expect("reachable package files should exist for every rebuild-graph package");
        let mut artifact = compile_package_artifact(
            import_path,
            package_files,
            artifacts
                .last()
                .map(|artifact| artifact.function_start + artifact.functions.len())
                .unwrap_or(0),
            next_global_index,
            next_user_type_offset,
            &imported_bindings,
            &source_lookup,
        )?;
        artifact.attach_dependency_edges(
            rebuild_graph
                .package(import_path)
                .expect("reachable package should exist for every rebuild-graph package"),
        );
        next_user_type_offset += artifact.user_type_span;
        next_global_index += artifact.global_count;
        artifact.extend_imported_bindings(&mut imported_bindings);
        artifacts.push(artifact);
    }

    assemble_compiled_workspace(
        &entry_import_path,
        entry_package_name,
        rebuild_graph,
        sources,
        artifacts,
    )
}

pub(super) fn recompile_workspace_affected_packages(
    previous: &CompiledWorkspace,
    resolved: import_resolution::ResolvedWorkspace,
    sources: &[SourceInput<'_>],
    changed_import_paths: &[String],
) -> Result<Option<(CompiledWorkspace, Vec<String>)>, CompileError> {
    let import_resolution::ResolvedWorkspace {
        entry_import_path,
        entry_package_name,
        package_files,
        rebuild_graph,
    } = resolved;
    if previous.rebuild_graph.entry_import_path() != entry_import_path {
        return Ok(None);
    }

    let previous_artifacts = previous
        .package_artifacts
        .iter()
        .map(|artifact| (artifact.import_path.as_str(), artifact))
        .collect::<HashMap<_, _>>();
    let changed = changed_import_paths
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut invalidated_reverse_dependencies = HashSet::new();
    let mut rebase_map = ArtifactRebaseMap::default();
    let mut imported_bindings = QualifiedPackageBindings::default();
    let mut artifacts = Vec::with_capacity(rebuild_graph.dependency_order().len());
    let mut recompiled_import_paths = Vec::new();
    let mut next_function_start = 0usize;
    let mut next_global_index = 0usize;
    let mut next_user_type_offset = 0u32;
    let source_lookup = sources
        .iter()
        .map(|source| (source.path, source.source))
        .collect::<HashMap<_, _>>();

    for import_path in rebuild_graph.dependency_order() {
        let previous_artifact = previous_artifacts.get(import_path.as_str()).copied();
        let should_recompile = previous_artifact.is_none()
            || changed.contains(import_path.as_str())
            || invalidated_reverse_dependencies.contains(import_path.as_str());
        let artifact = if should_recompile {
            let package_files = package_files
                .get(import_path)
                .expect("affected package files should exist in the resolved workspace");
            let mut artifact = compile_package_artifact(
                import_path,
                package_files,
                next_function_start,
                next_global_index,
                next_user_type_offset,
                &imported_bindings,
                &source_lookup,
            )?;
            artifact.attach_dependency_edges(
                rebuild_graph
                    .package(import_path)
                    .expect("recompiled package should exist in the rebuild graph"),
            );
            if previous_artifact.is_some_and(|previous_artifact| {
                !artifact.export_surface_matches(previous_artifact)
            }) {
                let package = rebuild_graph
                    .package(import_path)
                    .expect("recompiled package should exist in the rebuild graph");
                invalidated_reverse_dependencies
                    .extend(package.reverse_dependencies.iter().map(String::as_str));
            }
            recompiled_import_paths.push(import_path.clone());
            artifact
        } else {
            let previous_artifact = previous_artifact.expect("unchanged package should exist");
            let mut artifact = rebase_map.rebase_artifact(
                previous_artifact,
                next_function_start,
                next_global_index,
                next_user_type_offset,
            );
            artifact.refresh_reused_imported_contexts(&imported_bindings);
            artifact
        };
        next_function_start = artifact.function_start + artifact.functions.len();
        next_global_index = artifact.global_start + artifact.global_count;
        next_user_type_offset = artifact.user_type_offset + artifact.user_type_span;
        artifact.extend_imported_bindings(&mut imported_bindings);
        if let Some(previous_artifact) = previous_artifact {
            rebase_map.record_artifact(previous_artifact, &artifact);
        }
        artifacts.push(artifact);
    }

    assemble_compiled_workspace(
        &entry_import_path,
        entry_package_name,
        rebuild_graph,
        sources,
        artifacts,
    )
    .map(|compiled| Some((compiled, recompiled_import_paths)))
}

fn compile_package_artifact(
    import_path: &str,
    package_files: &[ParsedFile],
    function_start: usize,
    global_start: usize,
    user_type_offset: u32,
    imported_bindings: &QualifiedPackageBindings,
    source_lookup: &HashMap<&str, &str>,
) -> Result<CompiledPackageArtifact, CompileError> {
    let mut artifact = compile_phases::compile_package_artifact_in_explicit_phases(
        import_path,
        package_files,
        function_start,
        global_start,
        user_type_offset,
        imported_bindings,
    )
    .map_err(PhaseFailure::into_compile_error)?;
    artifact.capture_generic_template_sources(source_lookup)?;
    Ok(artifact)
}

fn assemble_compiled_workspace(
    entry_import_path: &str,
    entry_package_name: String,
    rebuild_graph: WorkspaceRebuildGraph,
    sources: &[SourceInput<'_>],
    artifacts: Vec<CompiledPackageArtifact>,
) -> Result<CompiledWorkspace, CompileError> {
    let mut functions = Vec::new();
    let mut debug_infos = Vec::new();
    let mut methods = Vec::new();
    let mut type_inventories = Vec::new();
    let mut package_init_functions = Vec::new();
    let mut entry_function = None;
    let mut next_global_index = 0usize;

    for (artifact, import_path) in artifacts.iter().zip(rebuild_graph.dependency_order()) {
        if artifact.import_path != *import_path
            || artifact.function_start != functions.len()
            || artifact.global_start != next_global_index
        {
            return Err(CompileError::Unsupported {
                detail: "internal package artifact assembly mismatch".into(),
            });
        }
        if import_path == entry_import_path {
            entry_function = artifact.entry_function;
        }
        if let Some(init_function) = artifact.package_init_function {
            package_init_functions.push(init_function);
        }
        functions.extend(artifact.functions.iter().cloned());
        debug_infos.extend(artifact.debug_infos.iter().cloned());
        methods.extend(artifact.methods.iter().cloned());
        type_inventories.push(artifact.type_inventory.clone());
        next_global_index += artifact.global_count;
    }

    let builtin_error_function = functions.len();
    let (builtin_error_method, builtin_error_binding) =
        compile_builtin_error_method(builtin_error_function);
    let builtin_error_method = CompiledFunction::without_debug(builtin_error_method);
    debug_infos.push(builtin_error_method.debug_info);
    functions.push(builtin_error_method.function);
    let context_cancel_helper =
        CompiledFunction::without_debug(compile_builtin_context_cancel_helper());
    debug_infos.push(context_cancel_helper.debug_info);
    functions.push(context_cancel_helper.function);

    let entry_function = entry_function.ok_or(CompileError::MissingMain {
        package: entry_package_name,
    })?;
    let entry_function = if package_init_functions.is_empty() {
        entry_function
    } else {
        let init_function = functions.len();
        let workspace_init = CompiledFunction::without_debug(compile_workspace_init_function(
            &package_init_functions,
        ));
        debug_infos.push(workspace_init.debug_info);
        functions.push(workspace_init.function);
        let entry_wrapper = functions.len();
        let entry_wrapper_function = CompiledFunction::without_debug(
            compile_package_entry_function(entry_function, init_function),
        );
        debug_infos.push(entry_wrapper_function.debug_info);
        functions.push(entry_wrapper_function.function);
        entry_wrapper
    };

    let program = Program {
        functions,
        methods: {
            methods.push(builtin_error_binding);
            methods
        },
        global_count: next_global_index,
        entry_function,
    };
    let type_inventory = merge_program_type_inventories(type_inventories);
    register_program_debug_info(
        &program,
        ProgramDebugInfo {
            functions: debug_infos,
            files: program::source_file_debug_infos(sources),
        },
    );
    register_program_type_inventory(&program, type_inventory);
    Ok(CompiledWorkspace {
        program,
        rebuild_graph,
        package_artifacts: artifacts,
    })
}

fn concrete_function_count(
    package_files: &[ParsedFile],
    generic_types: &HashMap<String, types::GenericTypeDef>,
) -> usize {
    package_files
        .iter()
        .flat_map(|parsed| parsed.file.functions.iter())
        .filter(|function| !program::is_generic_template_function(function, generic_types))
        .count()
}

fn offset_symbol_layout(layout: &mut symbols::SymbolLayout, function_start: usize) {
    if function_start == 0 {
        return;
    }
    for function in layout.function_ids.values_mut() {
        *function += function_start;
    }
    for function in layout.method_function_ids.values_mut() {
        *function += function_start;
    }
    for function in &mut layout.init_functions {
        *function += function_start;
    }
    for method in &mut layout.methods {
        method.function += function_start;
    }
}

fn offset_global_bindings(globals: &mut HashMap<String, GlobalBinding>, global_start: usize) {
    if global_start == 0 {
        return;
    }
    for binding in globals.values_mut() {
        binding.index += global_start;
    }
}
