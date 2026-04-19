use super::*;
use crate::program::compile_function;
use crate::types::TypeTables;

pub(super) fn compile_package_artifact_in_explicit_phases(
    import_path: &str,
    package_files: &[ParsedFile],
    function_start: usize,
    global_start: usize,
    user_type_offset: u32,
    imported_bindings: &QualifiedPackageBindings,
) -> Result<CompiledPackageArtifact, PhaseFailure> {
    PreparedPackageArtifactCompilation::prepare(
        import_path,
        package_files,
        function_start,
        global_start,
        user_type_offset,
        imported_bindings,
    )?
    .execute()
}

struct PreparedPackageArtifactCompilation<'a> {
    import_path: &'a str,
    package_files: &'a [ParsedFile],
    function_start: usize,
    global_start: usize,
    user_type_offset: u32,
    imported_bindings: &'a QualifiedPackageBindings,
    type_tables: TypeTables,
    user_type_span: u32,
    generic_function_templates: HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    layout: symbols::SymbolLayout,
    globals: HashMap<String, GlobalBinding>,
    total_functions: usize,
}

impl<'a> PreparedPackageArtifactCompilation<'a> {
    fn prepare(
        import_path: &'a str,
        package_files: &'a [ParsedFile],
        function_start: usize,
        global_start: usize,
        user_type_offset: u32,
        imported_bindings: &'a QualifiedPackageBindings,
    ) -> Result<Self, PhaseFailure> {
        let mut type_tables = collect_type_tables(
            package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file)),
        )
        .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let user_type_span = user_type_id_span(&type_tables);
        offset_user_type_ids(&mut type_tables, user_type_offset);
        let generic_function_templates = program::collect_generic_function_templates(
            package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file, Some(&parsed.spans))),
        );
        let generic_method_templates = program::collect_generic_method_templates(
            package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file, Some(&parsed.spans))),
            &type_tables,
        );
        let mut layout = collect_symbols(
            package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file)),
            &type_tables.structs,
            &type_tables.pointers,
            &type_tables.aliases,
            &type_tables.generic_types,
        )
        .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let mut globals = collect_globals(
            package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file)),
            &layout.function_types,
        )
        .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let total_functions = concrete_function_count(package_files, &type_tables.generic_types);
        offset_symbol_layout(&mut layout, function_start);
        offset_global_bindings(&mut globals, global_start);
        Ok(Self {
            import_path,
            package_files,
            function_start,
            global_start,
            user_type_offset,
            imported_bindings,
            type_tables,
            user_type_span,
            generic_function_templates,
            generic_method_templates,
            layout,
            globals,
            total_functions,
        })
    }

    fn execute(self) -> Result<CompiledPackageArtifact, PhaseFailure> {
        self.type_check()?;
        let lowered = self.lower()?;
        let emitted = lowered.emit_bytecode();
        let diagnostics_ready = emitted.register_runtime_metadata()?;
        Ok(diagnostics_ready.attach_diagnostics())
    }

    fn imported_package_tables(&self) -> ImportedPackageTables<'_> {
        ImportedPackageTables {
            function_ids: &self.imported_bindings.function_ids,
            generic_function_instances: &self.imported_bindings.generic_function_instances,
            function_result_types: &self.imported_bindings.function_result_types,
            function_types: &self.imported_bindings.function_types,
            variadic_functions: &self.imported_bindings.variadic_functions,
            globals: &self.imported_bindings.globals,
            structs: &self.imported_bindings.structs,
            interfaces: &self.imported_bindings.interfaces,
            pointers: &self.imported_bindings.pointers,
            aliases: &self.imported_bindings.aliases,
            method_function_ids: &self.imported_bindings.method_function_ids,
            promoted_method_bindings: &self.imported_bindings.promoted_method_bindings,
            method_sets: &self.imported_bindings.method_sets,
            generic_package_contexts: &self.imported_bindings.generic_package_contexts,
        }
    }

    fn type_check(&self) -> Result<(), PhaseFailure> {
        program::validate_package_declared_types(
            self.package_files
                .iter()
                .map(|parsed| (parsed.path.as_str(), &parsed.file)),
            self.imported_package_tables(),
            &self.layout.function_types,
            &self.layout.variadic_functions,
            &self.type_tables.generic_functions,
            &self.type_tables.generic_types,
            &self.generic_function_templates,
            &self.generic_method_templates,
            &self.layout.method_function_ids,
            &self.layout.promoted_method_bindings,
            &self.type_tables.structs,
            &self.type_tables.pointers,
            &self.type_tables.interfaces,
            &self.type_tables.aliases,
            &self.layout.method_sets,
            &self.globals,
        )
        .map_err(|error| PhaseFailure::new(CompilerPhase::TypeChecking, error, 0, 0))
    }

    fn lower(mut self) -> Result<LoweredPackageArtifactCompilation<'a>, PhaseFailure> {
        let imported_bindings = self.imported_bindings;
        let imported_package_tables = ImportedPackageTables {
            function_ids: &imported_bindings.function_ids,
            generic_function_instances: &imported_bindings.generic_function_instances,
            function_result_types: &imported_bindings.function_result_types,
            function_types: &imported_bindings.function_types,
            variadic_functions: &imported_bindings.variadic_functions,
            globals: &imported_bindings.globals,
            structs: &imported_bindings.structs,
            interfaces: &imported_bindings.interfaces,
            pointers: &imported_bindings.pointers,
            aliases: &imported_bindings.aliases,
            method_function_ids: &imported_bindings.method_function_ids,
            promoted_method_bindings: &imported_bindings.promoted_method_bindings,
            method_sets: &imported_bindings.method_sets,
            generic_package_contexts: &imported_bindings.generic_package_contexts,
        };
        let mut instantiated_generics = generic_instances::InstantiatedGenerics::new(
            &self.type_tables.structs,
            &self.type_tables.interfaces,
            &self.type_tables.aliases,
            &self.type_tables.pointers,
        );
        let mut generated_functions =
            GeneratedFunctions::new(self.function_start.saturating_add(self.total_functions));
        let mut functions = Vec::new();
        let mut debug_infos = Vec::new();

        for parsed in self.package_files {
            for (index, function) in parsed.file.functions.iter().enumerate() {
                if program::is_generic_template_function(function, &self.type_tables.generic_types)
                {
                    continue;
                }
                let compiled = compile_function(
                    &parsed.file,
                    &parsed.path,
                    parsed.spans.functions.get(index),
                    function,
                    imported_package_tables,
                    &self.layout.function_ids,
                    &self.layout.function_result_types,
                    &self.layout.function_types,
                    &self.layout.variadic_functions,
                    &self.type_tables.generic_functions,
                    &self.type_tables.generic_types,
                    &self.generic_function_templates,
                    &self.generic_method_templates,
                    &mut self.type_tables.instantiation_cache,
                    &mut generated_functions,
                    &mut instantiated_generics,
                    &self.layout.method_function_ids,
                    &self.layout.promoted_method_bindings,
                    &self.type_tables.structs,
                    &self.type_tables.pointers,
                    &self.type_tables.interfaces,
                    &self.type_tables.aliases,
                    &self.layout.method_sets,
                    &self.globals,
                )
                .map_err(|error| {
                    PhaseFailure::new(
                        CompilerPhase::Lowering,
                        error,
                        functions.len(),
                        debug_infos.len(),
                    )
                })?;
                debug_infos.push(compiled.debug_info);
                functions.push(compiled.function);
            }
        }

        let init_function_body = if self.globals.is_empty() && self.layout.init_functions.is_empty()
        {
            None
        } else {
            Some(
                compile_package_init_function(
                    self.package_files
                        .iter()
                        .map(|parsed| (parsed.path.as_str(), &parsed.file, Some(&parsed.spans))),
                    imported_package_tables,
                    &self.layout.function_ids,
                    &self.layout.function_result_types,
                    &self.layout.function_types,
                    &self.layout.variadic_functions,
                    &self.type_tables.generic_functions,
                    &self.type_tables.generic_types,
                    &self.generic_function_templates,
                    &self.generic_method_templates,
                    &mut self.type_tables.instantiation_cache,
                    &mut generated_functions,
                    &mut instantiated_generics,
                    &self.layout.method_function_ids,
                    &self.layout.promoted_method_bindings,
                    &self.type_tables.structs,
                    &self.type_tables.pointers,
                    &self.type_tables.interfaces,
                    &self.type_tables.aliases,
                    &self.layout.method_sets,
                    &self.globals,
                    &self.layout.init_functions,
                )
                .map_err(|error| {
                    PhaseFailure::new(
                        CompilerPhase::Lowering,
                        error,
                        functions.len(),
                        debug_infos.len(),
                    )
                })?,
            )
        };

        Ok(LoweredPackageArtifactCompilation {
            import_path: self.import_path,
            function_start: self.function_start,
            global_start: self.global_start,
            user_type_offset: self.user_type_offset,
            user_type_span: self.user_type_span,
            imported_bindings: self.imported_bindings,
            package_files: self.package_files,
            type_tables: self.type_tables,
            generic_function_templates: self.generic_function_templates,
            generic_method_templates: self.generic_method_templates,
            layout: self.layout,
            globals: self.globals,
            instantiated_generics,
            generated_functions,
            functions,
            debug_infos,
            init_function_body,
        })
    }
}

struct LoweredPackageArtifactCompilation<'a> {
    import_path: &'a str,
    function_start: usize,
    global_start: usize,
    user_type_offset: u32,
    user_type_span: u32,
    imported_bindings: &'a QualifiedPackageBindings,
    package_files: &'a [ParsedFile],
    type_tables: TypeTables,
    generic_function_templates: HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    layout: symbols::SymbolLayout,
    globals: HashMap<String, GlobalBinding>,
    instantiated_generics: generic_instances::InstantiatedGenerics,
    generated_functions: GeneratedFunctions,
    functions: Vec<Function>,
    debug_infos: Vec<FunctionDebugInfo>,
    init_function_body: Option<CompiledFunction>,
}

impl<'a> LoweredPackageArtifactCompilation<'a> {
    fn emit_bytecode(mut self) -> EmittedPackageArtifactCompilation {
        let package_selector = workspace_artifact_exports::package_selector_name(self.import_path);
        let local_named_types =
            workspace_artifact_exports::local_named_type_names(self.package_files);
        let qualified_generic_function_instances =
            workspace_artifact_exports::qualified_generic_function_instances(
                &exported_generic_function_instances(
                    &self.type_tables.instantiation_cache,
                    &self.generated_functions,
                ),
                self.import_path,
                package_selector,
                &local_named_types,
            );
        self.generated_functions
            .append_into(&mut self.functions, &mut self.debug_infos);
        let package_init_function = if let Some(init_function) = self.init_function_body {
            let init_index = self.function_start + self.functions.len();
            self.debug_infos.push(init_function.debug_info);
            self.functions.push(init_function.function);
            Some(init_index)
        } else {
            None
        };

        let mut qualified_structs = workspace_artifact_exports::qualified_structs(
            self.package_files,
            &self.type_tables.structs,
            package_selector,
            &local_named_types,
        );
        qualified_structs.extend(workspace_artifact_exports::qualified_instantiated_structs(
            self.instantiated_generics.struct_types(),
            package_selector,
            &local_named_types,
        ));
        let mut qualified_interfaces = workspace_artifact_exports::qualified_interfaces(
            self.package_files,
            &self.type_tables.interfaces,
            package_selector,
            &local_named_types,
        );
        qualified_interfaces.extend(
            workspace_artifact_exports::qualified_instantiated_interfaces(
                self.instantiated_generics.interface_types(),
                package_selector,
                &local_named_types,
            ),
        );
        let mut qualified_pointers = workspace_artifact_exports::qualified_pointers(
            self.package_files,
            &self.type_tables.pointers,
            package_selector,
            &local_named_types,
        );
        qualified_pointers.extend(workspace_artifact_exports::qualified_instantiated_pointers(
            self.instantiated_generics.pointer_types(),
            package_selector,
            &local_named_types,
        ));
        let mut qualified_aliases = workspace_artifact_exports::qualified_aliases(
            self.package_files,
            &self.type_tables.aliases,
            package_selector,
            &local_named_types,
        );
        qualified_aliases.extend(workspace_artifact_exports::qualified_instantiated_aliases(
            self.instantiated_generics.alias_types(),
            package_selector,
            &local_named_types,
        ));
        let mut qualified_method_function_ids =
            workspace_artifact_exports::qualified_method_function_ids(
                &self.layout,
                package_selector,
                &local_named_types,
            );
        qualified_method_function_ids.extend(
            workspace_artifact_exports::qualified_instantiated_method_function_ids(
                self.instantiated_generics.method_function_ids(),
                package_selector,
                &local_named_types,
            ),
        );
        let mut qualified_promoted_method_bindings =
            workspace_artifact_exports::qualified_promoted_method_bindings(
                &self.layout,
                package_selector,
                &local_named_types,
            );
        qualified_promoted_method_bindings.extend(
            workspace_artifact_exports::qualified_instantiated_promoted_method_bindings(
                self.instantiated_generics.promoted_method_bindings(),
                package_selector,
                &local_named_types,
            ),
        );
        let mut qualified_method_sets = workspace_artifact_exports::qualified_method_sets(
            &self.layout,
            package_selector,
            &local_named_types,
        );
        qualified_method_sets.extend(
            workspace_artifact_exports::qualified_instantiated_method_sets(
                self.instantiated_generics.all_method_sets(),
                package_selector,
                &local_named_types,
            ),
        );
        let generic_package_context = (!self.type_tables.generic_functions.is_empty()
            || !self.type_tables.generic_types.is_empty())
        .then(|| {
            std::sync::Arc::new(imported_generics::ImportedGenericPackageContext {
                package_path: self.import_path.to_string(),
                package_selector: package_selector.to_string(),
                local_named_types: local_named_types.clone(),
                imported_bindings: imported_generics::ImportedBindingsSnapshot {
                    function_ids: self.imported_bindings.function_ids.clone(),
                    generic_function_instances: self
                        .imported_bindings
                        .generic_function_instances
                        .clone(),
                    function_result_types: self.imported_bindings.function_result_types.clone(),
                    function_types: self.imported_bindings.function_types.clone(),
                    variadic_functions: self.imported_bindings.variadic_functions.clone(),
                    globals: self.imported_bindings.globals.clone(),
                    structs: self.imported_bindings.structs.clone(),
                    interfaces: self.imported_bindings.interfaces.clone(),
                    pointers: self.imported_bindings.pointers.clone(),
                    aliases: self.imported_bindings.aliases.clone(),
                    method_function_ids: self.imported_bindings.method_function_ids.clone(),
                    promoted_method_bindings: self
                        .imported_bindings
                        .promoted_method_bindings
                        .clone(),
                    method_sets: self.imported_bindings.method_sets.clone(),
                    generic_package_contexts: self
                        .imported_bindings
                        .generic_package_contexts
                        .clone(),
                },
                visible_generic_functions: workspace_artifact_exports::qualified_generic_functions(
                    &self.type_tables.generic_functions,
                    package_selector,
                    &local_named_types,
                ),
                generic_functions: self.type_tables.generic_functions.clone(),
                generic_types: self.type_tables.generic_types.clone(),
                generic_function_templates: self.generic_function_templates.clone(),
                generic_method_templates: self.generic_method_templates.clone(),
                instantiation_cache: self.type_tables.instantiation_cache.clone(),
                function_ids: self.layout.function_ids.clone(),
                function_result_types: self.layout.function_result_types.clone(),
                function_types: self.layout.function_types.clone(),
                variadic_functions: self.layout.variadic_functions.clone(),
                globals: self.globals.clone(),
                structs: self.type_tables.structs.clone(),
                interfaces: self.type_tables.interfaces.clone(),
                pointers: self.type_tables.pointers.clone(),
                aliases: self.type_tables.aliases.clone(),
                method_function_ids: self.layout.method_function_ids.clone(),
                promoted_method_bindings: self.layout.promoted_method_bindings.clone(),
                method_sets: self.layout.method_sets.clone(),
            })
        });

        let mut methods = self.layout.methods.clone();
        methods.extend(self.instantiated_generics.methods().iter().cloned());

        EmittedPackageArtifactCompilation {
            import_path: self.import_path.to_string(),
            function_start: self.function_start,
            global_start: self.global_start,
            global_count: self.globals.len(),
            user_type_offset: self.user_type_offset,
            user_type_span: self.user_type_span,
            type_tables: self.type_tables,
            instantiated_generics: self.instantiated_generics,
            functions: self.functions,
            debug_infos: self.debug_infos,
            methods,
            entry_function: self.layout.function_ids.get("main").copied(),
            package_init_function,
            qualified_function_ids: workspace_artifact_exports::qualified_function_ids(
                self.import_path,
                &self.layout,
            ),
            qualified_generic_function_instances,
            qualified_function_result_types:
                workspace_artifact_exports::qualified_function_result_types(
                    self.import_path,
                    &self.layout,
                    package_selector,
                    &local_named_types,
                ),
            qualified_function_types: workspace_artifact_exports::qualified_function_types(
                self.import_path,
                &self.layout,
                package_selector,
                &local_named_types,
            ),
            qualified_variadic_functions: workspace_artifact_exports::qualified_variadic_functions(
                self.import_path,
                &self.layout,
            ),
            qualified_globals: workspace_artifact_exports::qualified_globals(
                self.import_path,
                &self.globals,
                package_selector,
                &local_named_types,
            ),
            qualified_structs,
            qualified_interfaces,
            qualified_pointers,
            qualified_aliases,
            qualified_method_function_ids,
            qualified_promoted_method_bindings,
            qualified_method_sets,
            generic_package_context,
        }
    }
}

struct EmittedPackageArtifactCompilation {
    import_path: String,
    function_start: usize,
    functions: Vec<Function>,
    debug_infos: Vec<FunctionDebugInfo>,
    methods: Vec<gowasm_vm::MethodBinding>,
    global_start: usize,
    global_count: usize,
    user_type_offset: u32,
    user_type_span: u32,
    type_tables: TypeTables,
    instantiated_generics: generic_instances::InstantiatedGenerics,
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
    generic_package_context:
        Option<std::sync::Arc<imported_generics::ImportedGenericPackageContext>>,
}

impl EmittedPackageArtifactCompilation {
    fn register_runtime_metadata(
        self,
    ) -> Result<DiagnosticsReadyPackageArtifactCompilation, PhaseFailure> {
        let type_inventory = build_package_type_inventory(
            &self.import_path,
            &self.type_tables,
            &self.instantiated_generics,
        )
        .map_err(|error| {
            PhaseFailure::new(
                CompilerPhase::RuntimeMetadataRegistration,
                error,
                self.functions.len(),
                self.debug_infos.len(),
            )
        })?;
        Ok(DiagnosticsReadyPackageArtifactCompilation {
            import_path: self.import_path,
            function_start: self.function_start,
            functions: self.functions,
            debug_infos: self.debug_infos,
            methods: self.methods,
            global_start: self.global_start,
            global_count: self.global_count,
            user_type_offset: self.user_type_offset,
            user_type_span: self.user_type_span,
            type_inventory,
            entry_function: self.entry_function,
            package_init_function: self.package_init_function,
            qualified_function_ids: self.qualified_function_ids,
            qualified_generic_function_instances: self.qualified_generic_function_instances,
            qualified_function_result_types: self.qualified_function_result_types,
            qualified_function_types: self.qualified_function_types,
            qualified_variadic_functions: self.qualified_variadic_functions,
            qualified_globals: self.qualified_globals,
            qualified_structs: self.qualified_structs,
            qualified_interfaces: self.qualified_interfaces,
            qualified_pointers: self.qualified_pointers,
            qualified_aliases: self.qualified_aliases,
            qualified_method_function_ids: self.qualified_method_function_ids,
            qualified_promoted_method_bindings: self.qualified_promoted_method_bindings,
            qualified_method_sets: self.qualified_method_sets,
            generic_package_context: self.generic_package_context,
        })
    }
}

struct DiagnosticsReadyPackageArtifactCompilation {
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
    generic_package_context:
        Option<std::sync::Arc<imported_generics::ImportedGenericPackageContext>>,
}

impl DiagnosticsReadyPackageArtifactCompilation {
    fn attach_diagnostics(self) -> CompiledPackageArtifact {
        CompiledPackageArtifact {
            import_path: self.import_path,
            function_start: self.function_start,
            functions: self.functions,
            debug_infos: self.debug_infos,
            methods: self.methods,
            global_start: self.global_start,
            global_count: self.global_count,
            user_type_offset: self.user_type_offset,
            user_type_span: self.user_type_span,
            type_inventory: self.type_inventory,
            entry_function: self.entry_function,
            package_init_function: self.package_init_function,
            qualified_function_ids: self.qualified_function_ids,
            qualified_generic_function_instances: self.qualified_generic_function_instances,
            qualified_function_result_types: self.qualified_function_result_types,
            qualified_function_types: self.qualified_function_types,
            qualified_variadic_functions: self.qualified_variadic_functions,
            qualified_globals: self.qualified_globals,
            qualified_structs: self.qualified_structs,
            qualified_interfaces: self.qualified_interfaces,
            qualified_pointers: self.qualified_pointers,
            qualified_aliases: self.qualified_aliases,
            qualified_method_function_ids: self.qualified_method_function_ids,
            qualified_promoted_method_bindings: self.qualified_promoted_method_bindings,
            qualified_method_sets: self.qualified_method_sets,
            dependency_edges: ArtifactDependencyEdges::default(),
            generic_function_template_sources: HashMap::new(),
            generic_method_template_sources: HashMap::new(),
            generic_package_context: self.generic_package_context,
        }
    }
}

fn exported_generic_function_instances(
    instantiation_cache: &InstantiationCache,
    generated_functions: &GeneratedFunctions,
) -> HashMap<InstanceKey, usize> {
    instantiation_cache
        .function_instances
        .iter()
        .filter_map(|(key, name)| {
            generated_functions
                .instance_function_ids()
                .get(name)
                .copied()
                .map(|function| (key.clone(), function))
        })
        .collect()
}
