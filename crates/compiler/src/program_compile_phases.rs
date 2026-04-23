use super::*;
use crate::types::TypeTables;

#[derive(Default)]
struct EmptyImportedBindings {
    function_ids: HashMap<String, usize>,
    generic_function_instances: HashMap<types::InstanceKey, usize>,
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

impl EmptyImportedBindings {
    fn tables(&self) -> ImportedPackageTables<'_> {
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

pub(super) fn compile_file_in_explicit_phases(file: &SourceFile) -> Result<Program, PhaseFailure> {
    PreparedSingleFileCompilation::prepare(file)?.execute()
}

#[cfg(test)]
pub(crate) fn compile_file_in_explicit_phases_for_tests(
    file: &SourceFile,
) -> Result<Program, PhaseFailure> {
    compile_file_in_explicit_phases(file)
}

struct PreparedSingleFileCompilation<'a> {
    file: &'a SourceFile,
    source_path: &'static str,
    empty_imports: EmptyImportedBindings,
    type_tables: TypeTables,
    generic_function_templates: HashMap<String, GenericFunctionTemplate>,
    generic_method_templates: HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    layout: symbols::SymbolLayout,
    globals: HashMap<String, GlobalBinding>,
    concrete_function_count: usize,
}

impl<'a> PreparedSingleFileCompilation<'a> {
    fn prepare(file: &'a SourceFile) -> Result<Self, PhaseFailure> {
        let source_path = "main.go";
        let type_tables = collect_type_tables(std::iter::once((source_path, file)))
            .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let generic_function_templates = collect_generic_function_templates(std::iter::once((
            source_path,
            file,
            None::<&gowasm_parser::SourceFileSpans>,
        )));
        let generic_method_templates = collect_generic_method_templates(
            std::iter::once((source_path, file, None::<&gowasm_parser::SourceFileSpans>)),
            &type_tables,
        );
        let layout = collect_symbols(
            std::iter::once((source_path, file)),
            &type_tables.structs,
            &type_tables.pointers,
            &type_tables.aliases,
            &type_tables.generic_types,
        )
        .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let globals = collect_globals(std::iter::once((source_path, file)), &layout.function_types)
            .map_err(|error| PhaseFailure::new(CompilerPhase::ParseValidation, error, 0, 0))?;
        let concrete_function_count = file
            .functions
            .iter()
            .filter(|function| !is_generic_template_function(function, &type_tables.generic_types))
            .count();
        Ok(Self {
            file,
            source_path,
            empty_imports: EmptyImportedBindings::default(),
            type_tables,
            generic_function_templates,
            generic_method_templates,
            layout,
            globals,
            concrete_function_count,
        })
    }

    fn execute(self) -> Result<Program, PhaseFailure> {
        self.type_check()?;
        let lowered = self.lower()?;
        let emitted = lowered.emit_bytecode()?;
        let diagnostics_ready = emitted.register_runtime_metadata()?;
        Ok(diagnostics_ready.register_diagnostics())
    }

    fn type_check(&self) -> Result<(), PhaseFailure> {
        validate_package_declared_types(
            std::iter::once((self.source_path, self.file)),
            self.empty_imports.tables(),
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

    fn lower(mut self) -> Result<LoweredSingleFileCompilation, PhaseFailure> {
        let mut instantiated_generics = generic_instances::InstantiatedGenerics::new(
            &self.type_tables.structs,
            &self.type_tables.interfaces,
            &self.type_tables.aliases,
            &self.type_tables.pointers,
        );
        let mut generated_functions = GeneratedFunctions::new(self.concrete_function_count);
        let mut functions = Vec::with_capacity(self.concrete_function_count);
        let mut debug_infos = Vec::with_capacity(self.concrete_function_count + 3);

        for function in &self.file.functions {
            if is_generic_template_function(function, &self.type_tables.generic_types) {
                continue;
            }
            let compiled = compile_function(
                self.file,
                self.source_path,
                None,
                function,
                self.empty_imports.tables(),
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

        let init_function_body =
            if self.file.vars.is_empty() && self.layout.init_functions.is_empty() {
                None
            } else {
                Some(
                    compile_package_init_function(
                        std::iter::once((
                            self.source_path,
                            self.file,
                            None::<&gowasm_parser::SourceFileSpans>,
                        )),
                        self.empty_imports.tables(),
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

        let entry_function = self.layout.function_ids.get("main").copied();
        Ok(LoweredSingleFileCompilation {
            package_name: self.file.package_name.clone(),
            type_tables: self.type_tables,
            layout: self.layout,
            globals: self.globals,
            entry_function,
            instantiated_generics,
            generated_functions,
            functions,
            debug_infos,
            init_function_body,
        })
    }
}

struct LoweredSingleFileCompilation {
    package_name: String,
    type_tables: TypeTables,
    layout: symbols::SymbolLayout,
    globals: HashMap<String, GlobalBinding>,
    entry_function: Option<usize>,
    instantiated_generics: generic_instances::InstantiatedGenerics,
    generated_functions: GeneratedFunctions,
    functions: Vec<Function>,
    debug_infos: Vec<FunctionDebugInfo>,
    init_function_body: Option<CompiledFunction>,
}

impl LoweredSingleFileCompilation {
    fn emit_bytecode(mut self) -> Result<EmittedSingleFileCompilation, PhaseFailure> {
        self.generated_functions
            .append_into(&mut self.functions, &mut self.debug_infos);
        let builtin_error_function = self.functions.len();
        let (builtin_error_method, builtin_error_binding) =
            compile_builtin_error_method(builtin_error_function);
        let builtin_error_method = CompiledFunction::without_debug(builtin_error_method);
        self.debug_infos.push(builtin_error_method.debug_info);
        self.functions.push(builtin_error_method.function);
        let context_cancel_helper =
            CompiledFunction::without_debug(compile_builtin_context_cancel_helper());
        self.debug_infos.push(context_cancel_helper.debug_info);
        self.functions.push(context_cancel_helper.function);

        let entry_function = self.entry_function.ok_or_else(|| {
            PhaseFailure::new(
                CompilerPhase::BytecodeEmission,
                CompileError::MissingMain {
                    package: self.package_name.clone(),
                },
                self.functions.len(),
                self.debug_infos.len(),
            )
        })?;
        let entry_function = if let Some(init_body) = self.init_function_body {
            let init_function = self.functions.len();
            self.debug_infos.push(init_body.debug_info);
            self.functions.push(init_body.function);
            let entry_wrapper = self.functions.len();
            let entry_wrapper_function = CompiledFunction::without_debug(
                compile_package_entry_function(entry_function, init_function),
            );
            self.debug_infos.push(entry_wrapper_function.debug_info);
            self.functions.push(entry_wrapper_function.function);
            entry_wrapper
        } else {
            entry_function
        };
        let mut methods = self.layout.methods.clone();
        methods.extend(self.instantiated_generics.methods().iter().cloned());
        methods.push(builtin_error_binding);

        Ok(EmittedSingleFileCompilation {
            package_name: self.package_name,
            type_tables: self.type_tables,
            instantiated_generics: self.instantiated_generics,
            program: Program {
                functions: self.functions,
                methods,
                global_count: self.globals.len(),
                entry_function,
            },
            debug_infos: self.debug_infos,
        })
    }
}

struct EmittedSingleFileCompilation {
    package_name: String,
    type_tables: TypeTables,
    instantiated_generics: generic_instances::InstantiatedGenerics,
    program: Program,
    debug_infos: Vec<FunctionDebugInfo>,
}

impl EmittedSingleFileCompilation {
    fn register_runtime_metadata(
        self,
    ) -> Result<DiagnosticsReadySingleFileCompilation, PhaseFailure> {
        let type_inventory = build_package_type_inventory(
            &self.package_name,
            &self.type_tables,
            &self.instantiated_generics,
        )
        .map_err(|error| {
            PhaseFailure::new(
                CompilerPhase::RuntimeMetadataRegistration,
                error,
                self.program.functions.len(),
                self.debug_infos.len(),
            )
        })?;
        register_program_type_inventory(&self.program, type_inventory);
        Ok(DiagnosticsReadySingleFileCompilation {
            program: self.program,
            debug_infos: self.debug_infos,
        })
    }
}

struct DiagnosticsReadySingleFileCompilation {
    program: Program,
    debug_infos: Vec<FunctionDebugInfo>,
}

impl DiagnosticsReadySingleFileCompilation {
    fn register_diagnostics(self) -> Program {
        register_program_debug_info(
            &self.program,
            ProgramDebugInfo {
                functions: self.debug_infos,
                files: HashMap::new(),
            },
        );
        self.program
    }
}
