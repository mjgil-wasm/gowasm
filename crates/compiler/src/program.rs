use super::*;

#[path = "program_compile_phases.rs"]
mod compile_phases;

#[cfg(test)]
pub(crate) use compile_phases::compile_file_in_explicit_phases_for_tests;

pub fn compile_file(file: &SourceFile) -> Result<Program, CompileError> {
    clear_last_compile_error_context();
    compile_phases::compile_file_in_explicit_phases(file).map_err(PhaseFailure::into_compile_error)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_package_declared_types<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a SourceFile)>,
    imported_package_tables: ImportedPackageTables<'_>,
    function_types: &HashMap<String, String>,
    variadic_functions: &HashSet<String>,
    generic_functions: &HashMap<String, GenericFunctionDef>,
    generic_types: &HashMap<String, GenericTypeDef>,
    generic_function_templates: &HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: &HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    method_function_ids: &HashMap<String, usize>,
    promoted_method_bindings: &HashMap<String, symbols::PromotedMethodBindingInfo>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    globals: &HashMap<String, GlobalBinding>,
) -> Result<(), CompileError> {
    let empty_function_ids = HashMap::new();
    let empty_function_result_types = HashMap::new();
    let mut instantiation_cache = InstantiationCache::default();
    let mut generated_functions = GeneratedFunctions::new(0);
    let mut instantiated_generics = generic_instances::InstantiatedGenerics::new(
        struct_types,
        interface_types,
        alias_types,
        pointer_types,
    );
    let mut builder = FunctionBuilder {
        emitter: EmitterState::default(),
        env: CompilerEnvironment::new(
            ImportContext {
                imported_packages: HashMap::new(),
                imported_package_tables,
            },
            SymbolTables {
                function_ids: &empty_function_ids,
                function_result_types: &empty_function_result_types,
                function_types,
                variadic_functions,
                method_function_ids,
                promoted_method_bindings,
                globals,
                method_sets,
            },
            TypeContext {
                generic_functions,
                generic_types,
                generic_function_templates,
                generic_method_templates,
            },
            RuntimeMetadataContext {
                struct_types,
                pointer_types,
                interface_types,
                alias_types,
            },
        ),
        generation: GenerationState {
            instantiation_cache: &mut instantiation_cache,
            generated_functions: &mut generated_functions,
            instantiated_generics: &mut instantiated_generics,
            generic_instance_namespace: None,
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

    for (_path, file) in files {
        builder
            .env
            .set_imported_packages(imported_packages_for(file));

        for type_decl in &file.types {
            let type_params = type_decl
                .type_params
                .iter()
                .map(types::lower_type_param)
                .collect::<Vec<_>>();
            match &type_decl.kind {
                gowasm_parser::TypeDeclKind::Struct { fields } => {
                    for field in fields {
                        builder.ensure_runtime_visible_type_in_context(&field.typ, &type_params)?;
                    }
                }
                gowasm_parser::TypeDeclKind::Alias { underlying } => {
                    builder.ensure_runtime_visible_type_in_context(underlying, &type_params)?;
                }
                gowasm_parser::TypeDeclKind::Interface { methods, embeds } => {
                    for method in methods {
                        for param in &method.params {
                            builder
                                .ensure_runtime_visible_type_in_context(&param.typ, &type_params)?;
                        }
                        for result in &method.result_types {
                            builder.ensure_runtime_visible_type_in_context(result, &type_params)?;
                        }
                    }
                    for embed in embeds {
                        builder.ensure_runtime_visible_type_in_context(embed, &type_params)?;
                    }
                }
            }
        }

        for function in &file.functions {
            let type_params = function
                .type_params
                .iter()
                .map(types::lower_type_param)
                .collect::<Vec<_>>();
            if let Some(receiver) = &function.receiver {
                builder.ensure_runtime_visible_type_in_context(&receiver.typ, &type_params)?;
            }
            for param in &function.params {
                builder.ensure_runtime_visible_type_in_context(&param.typ, &type_params)?;
            }
            for result in &function.result_types {
                builder.ensure_runtime_visible_type_in_context(result, &type_params)?;
            }
        }

        for constant in &file.consts {
            if let Some(typ) = &constant.typ {
                builder.ensure_runtime_visible_type_in_context(typ, &[])?;
            }
        }

        for var in &file.vars {
            if let Some(typ) = &var.typ {
                builder.ensure_runtime_visible_type_in_context(typ, &[])?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_function(
    file: &SourceFile,
    source_path: &str,
    function_spans: Option<&FunctionSourceSpans>,
    function: &FunctionDecl,
    imported_package_tables: ImportedPackageTables<'_>,
    function_ids: &HashMap<String, usize>,
    function_result_types: &HashMap<String, Vec<String>>,
    function_types: &HashMap<String, String>,
    variadic_functions: &HashSet<String>,
    generic_functions: &HashMap<String, GenericFunctionDef>,
    generic_types: &HashMap<String, GenericTypeDef>,
    generic_function_templates: &HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: &HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    instantiation_cache: &mut InstantiationCache,
    generated_functions: &mut GeneratedFunctions,
    instantiated_generics: &mut generic_instances::InstantiatedGenerics,
    method_function_ids: &HashMap<String, usize>,
    promoted_method_bindings: &HashMap<String, symbols::PromotedMethodBindingInfo>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    globals: &HashMap<String, GlobalBinding>,
) -> Result<CompiledFunction, CompileError> {
    compile_function_with_imports(
        imported_packages_for(file),
        source_path,
        function_spans,
        function,
        imported_package_tables,
        function_ids,
        function_result_types,
        function_types,
        variadic_functions,
        generic_functions,
        generic_types,
        generic_function_templates,
        generic_method_templates,
        instantiation_cache,
        generated_functions,
        instantiated_generics,
        method_function_ids,
        promoted_method_bindings,
        struct_types,
        pointer_types,
        interface_types,
        alias_types,
        method_sets,
        globals,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_function_with_imports(
    imported_packages: HashMap<String, String>,
    source_path: &str,
    function_spans: Option<&FunctionSourceSpans>,
    function: &FunctionDecl,
    imported_package_tables: ImportedPackageTables<'_>,
    function_ids: &HashMap<String, usize>,
    function_result_types: &HashMap<String, Vec<String>>,
    function_types: &HashMap<String, String>,
    variadic_functions: &HashSet<String>,
    generic_functions: &HashMap<String, GenericFunctionDef>,
    generic_types: &HashMap<String, GenericTypeDef>,
    generic_function_templates: &HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: &HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    instantiation_cache: &mut InstantiationCache,
    generated_functions: &mut GeneratedFunctions,
    instantiated_generics: &mut generic_instances::InstantiatedGenerics,
    method_function_ids: &HashMap<String, usize>,
    promoted_method_bindings: &HashMap<String, symbols::PromotedMethodBindingInfo>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    globals: &HashMap<String, GlobalBinding>,
) -> Result<CompiledFunction, CompileError> {
    compile_function_with_namespace(
        imported_packages,
        source_path,
        function_spans,
        function,
        imported_package_tables,
        function_ids,
        function_result_types,
        function_types,
        variadic_functions,
        generic_functions,
        generic_types,
        generic_function_templates,
        generic_method_templates,
        instantiation_cache,
        generated_functions,
        instantiated_generics,
        method_function_ids,
        promoted_method_bindings,
        struct_types,
        pointer_types,
        interface_types,
        alias_types,
        method_sets,
        globals,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_function_with_namespace(
    imported_packages: HashMap<String, String>,
    source_path: &str,
    function_spans: Option<&FunctionSourceSpans>,
    function: &FunctionDecl,
    imported_package_tables: ImportedPackageTables<'_>,
    function_ids: &HashMap<String, usize>,
    function_result_types: &HashMap<String, Vec<String>>,
    function_types: &HashMap<String, String>,
    variadic_functions: &HashSet<String>,
    generic_functions: &HashMap<String, GenericFunctionDef>,
    generic_types: &HashMap<String, GenericTypeDef>,
    generic_function_templates: &HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: &HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    instantiation_cache: &mut InstantiationCache,
    generated_functions: &mut GeneratedFunctions,
    instantiated_generics: &mut generic_instances::InstantiatedGenerics,
    method_function_ids: &HashMap<String, usize>,
    promoted_method_bindings: &HashMap<String, symbols::PromotedMethodBindingInfo>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    globals: &HashMap<String, GlobalBinding>,
    generic_instance_namespace: Option<String>,
) -> Result<CompiledFunction, CompileError> {
    let mut params = HashMap::new();
    let mut param_types = HashMap::new();
    let mut initial_bindings = Vec::new();
    let mut next_param = 0usize;
    if let Some(receiver) = &function.receiver {
        if !receiver.name.is_empty() {
            params.insert(receiver.name.clone(), next_param);
            param_types.insert(receiver.name.clone(), receiver.typ.clone());
            initial_bindings.push((receiver.name.clone(), false));
        }
        next_param += 1;
    }
    for parameter in &function.params {
        params.insert(parameter.name.clone(), next_param);
        param_types.insert(parameter.name.clone(), parameter.typ.clone());
        initial_bindings.push((parameter.name.clone(), false));
        next_param += 1;
    }

    let source_spans = function_spans.map(|spans| std::sync::Arc::new(spans.bind(function)));
    let mut code = InstructionBuffer::default();
    if let Some(source_spans) = &source_spans {
        code.set_active_span(Some(InstructionSourceSpan {
            path: source_path.to_string(),
            start: source_spans.function_span().start,
            end: source_spans.function_span().end,
        }));
    }
    let mut builder = FunctionBuilder {
        emitter: EmitterState {
            code,
            next_register: next_param,
            default_source_path: Some(source_path.to_string()),
            source_spans,
        },
        env: CompilerEnvironment::new(
            ImportContext {
                imported_packages,
                imported_package_tables,
            },
            SymbolTables {
                function_ids,
                function_result_types,
                function_types,
                variadic_functions,
                method_function_ids,
                promoted_method_bindings,
                globals,
                method_sets,
            },
            TypeContext {
                generic_functions,
                generic_types,
                generic_function_templates,
                generic_method_templates,
            },
            RuntimeMetadataContext {
                struct_types,
                pointer_types,
                interface_types,
                alias_types,
            },
        ),
        generation: GenerationState {
            instantiation_cache,
            generated_functions,
            instantiated_generics,
            generic_instance_namespace,
        },
        scopes: ScopeStack {
            scopes: vec![params],
            captured_by_ref: collect_direct_by_ref_captures(initial_bindings, &function.body),
            const_scopes: vec![HashSet::new()],
            const_value_scopes: vec![HashMap::new()],
            type_scopes: vec![param_types],
        },
        control: ControlFlowContext {
            in_package_init: false,
            current_result_types: function.result_types.clone(),
            current_result_names: function.result_names.clone(),
            break_scopes: Vec::new(),
            loops: Vec::new(),
            pending_label: None,
        },
    };
    builder
        .scopes
        .captured_by_ref
        .extend(builder.collect_address_taken_bindings(&function.body));
    if let Some(receiver) = &function.receiver {
        if !receiver.name.is_empty() {
            builder.box_captured_parameter(&receiver.name, &receiver.typ);
        }
    }
    builder.box_captured_parameters(&function.params);
    builder.declare_named_results()?;

    for stmt in &function.body {
        builder.compile_stmt(stmt)?;
    }
    builder.emit_implicit_return();
    let (code, debug_info) = builder.emitter.code.into_parts();

    Ok(CompiledFunction {
        function: Function {
            name: qualified_function_name(function),
            param_count: next_param,
            register_count: builder.emitter.next_register,
            code,
        },
        debug_info,
    })
}

fn qualified_function_name(function: &FunctionDecl) -> String {
    function.receiver.as_ref().map_or_else(
        || function.name.clone(),
        |receiver| format!("{}.{}", receiver.typ, function.name),
    )
}

#[derive(Clone)]
pub struct CompiledWorkspace {
    pub program: Program,
    pub rebuild_graph: WorkspaceRebuildGraph,
    pub(crate) package_artifacts: Vec<workspace_artifacts::CompiledPackageArtifact>,
}

pub fn compile_workspace(
    sources: &[SourceInput<'_>],
    entry_path: &str,
) -> Result<Program, CompileError> {
    clear_last_compile_error_context();
    compile_workspace_with_graph(sources, entry_path).map(|compiled| compiled.program)
}

pub fn compile_workspace_with_graph(
    sources: &[SourceInput<'_>],
    entry_path: &str,
) -> Result<CompiledWorkspace, CompileError> {
    clear_last_compile_error_context();
    let resolved = import_resolution::resolve_workspace_imports(sources, entry_path)?;
    workspace_artifacts::compile_resolved_workspace(resolved, sources)
}

pub fn recompile_workspace_affected_packages(
    previous: &CompiledWorkspace,
    sources: &[SourceInput<'_>],
    entry_path: &str,
    changed_import_paths: &[String],
) -> Result<Option<(CompiledWorkspace, Vec<String>)>, CompileError> {
    clear_last_compile_error_context();
    let resolved = import_resolution::resolve_workspace_imports(sources, entry_path)?;
    workspace_artifacts::recompile_workspace_affected_packages(
        previous,
        resolved,
        sources,
        changed_import_paths,
    )
}

pub(super) fn is_generic_template_function(
    function: &FunctionDecl,
    generic_types: &HashMap<String, types::GenericTypeDef>,
) -> bool {
    !function.type_params.is_empty()
        || function
            .receiver
            .as_ref()
            .is_some_and(|receiver| types::is_generic_receiver_type(&receiver.typ, generic_types))
}

pub(super) fn collect_generic_function_templates<'a>(
    files: impl IntoIterator<
        Item = (
            &'a str,
            &'a SourceFile,
            Option<&'a gowasm_parser::SourceFileSpans>,
        ),
    >,
) -> HashMap<String, GenericFunctionTemplate> {
    let mut templates = HashMap::new();
    for (path, file, spans) in files {
        let imported_packages = imported_packages_for(file);
        for (index, function) in file.functions.iter().enumerate() {
            if function.type_params.is_empty() {
                continue;
            }
            templates.insert(
                function.name.clone(),
                GenericFunctionTemplate {
                    decl: function.clone(),
                    source_path: path.to_string(),
                    spans: spans
                        .and_then(|spans| spans.functions.get(index).cloned())
                        .unwrap_or_else(FunctionSourceSpans::empty),
                    imported_packages: imported_packages.clone(),
                },
            );
        }
    }
    templates
}

pub(super) fn collect_generic_method_templates<'a>(
    files: impl IntoIterator<
        Item = (
            &'a str,
            &'a SourceFile,
            Option<&'a gowasm_parser::SourceFileSpans>,
        ),
    >,
    type_tables: &types::TypeTables,
) -> HashMap<String, Vec<generic_instances::GenericMethodTemplate>> {
    let mut templates = HashMap::new();
    for (path, file, spans) in files {
        let imported_packages = imported_packages_for(file);
        for (index, function) in file.functions.iter().enumerate() {
            let Some(receiver) = &function.receiver else {
                continue;
            };
            let Some((_pointer_receiver, base_name, _type_args)) =
                types::parse_generic_receiver_type(&receiver.typ)
            else {
                continue;
            };
            if !type_tables.generic_types.contains_key(&base_name) {
                continue;
            }
            templates.entry(base_name).or_insert_with(Vec::new).push(
                generic_instances::GenericMethodTemplate {
                    decl: function.clone(),
                    source_path: path.to_string(),
                    spans: spans
                        .and_then(|spans| spans.functions.get(index).cloned())
                        .unwrap_or_else(FunctionSourceSpans::empty),
                    imported_packages: imported_packages.clone(),
                },
            );
        }
    }
    templates
}

pub(super) fn imported_packages_for(file: &SourceFile) -> HashMap<String, String> {
    file.imports
        .iter()
        .map(|decl| (decl.selector().to_string(), decl.path.clone()))
        .collect()
}

pub(super) fn source_file_debug_infos(
    sources: &[SourceInput<'_>],
) -> HashMap<String, SourceFileDebugInfo> {
    sources
        .iter()
        .map(|source| {
            (
                source.path.to_string(),
                SourceFileDebugInfo::from_source(source.source),
            )
        })
        .collect()
}
