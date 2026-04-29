use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{
    const_eval::ConstValueInfo, generic_instances, symbols, AliasTypeDef, BoundFunctionSourceSpans,
    GeneratedFunctions, GenericFunctionDef, GenericFunctionTemplate, GenericTypeDef, GlobalBinding,
    ImportedPackageTables, InstantiationCache, InterfaceMethodDecl, InterfaceTypeDef,
    StructTypeDef, TypeId,
};
use gowasm_vm::{FunctionDebugInfo, Instruction, InstructionSourceSpan};

#[derive(Debug, Default, Clone)]
pub(super) struct InstructionBuffer {
    instructions: Vec<Instruction>,
    spans: Vec<Option<InstructionSourceSpan>>,
    active_span: Option<InstructionSourceSpan>,
}

impl InstructionBuffer {
    pub(super) fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
        self.spans.push(self.active_span.clone());
    }

    pub(super) fn len(&self) -> usize {
        self.instructions.len()
    }

    pub(super) fn get_mut(&mut self, index: usize) -> Option<&mut Instruction> {
        self.instructions.get_mut(index)
    }

    pub(super) fn active_span(&self) -> Option<InstructionSourceSpan> {
        self.active_span.clone()
    }

    pub(super) fn set_active_span(&mut self, span: Option<InstructionSourceSpan>) {
        self.active_span = span;
    }

    pub(super) fn into_parts(self) -> (Vec<Instruction>, FunctionDebugInfo) {
        (
            self.instructions,
            FunctionDebugInfo {
                instruction_spans: self.spans,
            },
        )
    }
}

impl std::ops::Index<usize> for InstructionBuffer {
    type Output = Instruction;

    fn index(&self, index: usize) -> &Self::Output {
        &self.instructions[index]
    }
}

impl std::ops::IndexMut<usize> for InstructionBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.instructions[index]
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct EmitterState {
    pub(super) code: InstructionBuffer,
    pub(super) next_register: usize,
    pub(super) default_source_path: Option<String>,
    pub(super) source_spans: Option<Arc<BoundFunctionSourceSpans>>,
}

impl EmitterState {
    pub(super) fn alloc_register(&mut self) -> usize {
        let register = self.next_register;
        self.next_register += 1;
        register
    }
}

#[derive(Clone)]
pub(super) struct ImportContext<'a> {
    pub(super) imported_packages: HashMap<String, String>,
    pub(super) imported_package_tables: ImportedPackageTables<'a>,
}

impl ImportContext<'_> {
    #[allow(dead_code)]
    pub(super) fn package_path(&self, selector: &str) -> Option<&str> {
        self.imported_packages.get(selector).map(String::as_str)
    }
}

#[derive(Clone, Copy)]
pub(super) struct SymbolTables<'a> {
    pub(super) function_ids: &'a HashMap<String, usize>,
    pub(super) function_result_types: &'a HashMap<String, Vec<String>>,
    pub(super) function_types: &'a HashMap<String, String>,
    pub(super) variadic_functions: &'a HashSet<String>,
    pub(super) method_function_ids: &'a HashMap<String, usize>,
    pub(super) promoted_method_bindings: &'a HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(super) globals: &'a HashMap<String, GlobalBinding>,
    pub(super) method_sets: &'a HashMap<String, Vec<InterfaceMethodDecl>>,
}

impl SymbolTables<'_> {
    #[allow(dead_code)]
    pub(super) fn function_type(&self, name: &str) -> Option<&str> {
        self.function_types.get(name).map(String::as_str)
    }

    #[allow(dead_code)]
    pub(super) fn global_index(&self, name: &str) -> Option<usize> {
        self.globals.get(name).map(|binding| binding.index)
    }
}

#[derive(Clone, Copy)]
pub(super) struct TypeContext<'a> {
    pub(super) generic_functions: &'a HashMap<String, GenericFunctionDef>,
    pub(super) generic_types: &'a HashMap<String, GenericTypeDef>,
    pub(super) generic_function_templates: &'a HashMap<String, GenericFunctionTemplate>,
    pub(super) generic_method_templates:
        &'a HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
}

impl TypeContext<'_> {
    #[allow(dead_code)]
    pub(super) fn has_generic_type(&self, name: &str) -> bool {
        self.generic_types.contains_key(name)
    }
}

#[derive(Clone, Copy)]
pub(super) struct RuntimeMetadataContext<'a> {
    pub(super) struct_types: &'a HashMap<String, StructTypeDef>,
    pub(super) pointer_types: &'a HashMap<String, TypeId>,
    pub(super) interface_types: &'a HashMap<String, InterfaceTypeDef>,
    pub(super) alias_types: &'a HashMap<String, AliasTypeDef>,
}

impl RuntimeMetadataContext<'_> {
    #[allow(dead_code)]
    pub(super) fn contains_named_type(&self, name: &str) -> bool {
        self.struct_types.contains_key(name)
            || self.interface_types.contains_key(name)
            || self.alias_types.contains_key(name)
            || self.pointer_types.contains_key(name)
    }
}

#[derive(Clone)]
pub(super) struct CompilerEnvironment<'a> {
    pub(super) imports: ImportContext<'a>,
    #[allow(dead_code)]
    pub(super) symbols: SymbolTables<'a>,
    #[allow(dead_code)]
    pub(super) types: TypeContext<'a>,
    #[allow(dead_code)]
    pub(super) runtime_metadata: RuntimeMetadataContext<'a>,
    pub(super) imported_packages: HashMap<String, String>,
    pub(super) imported_package_tables: ImportedPackageTables<'a>,
    pub(super) function_ids: &'a HashMap<String, usize>,
    pub(super) function_result_types: &'a HashMap<String, Vec<String>>,
    pub(super) function_types: &'a HashMap<String, String>,
    pub(super) variadic_functions: &'a HashSet<String>,
    pub(super) generic_functions: &'a HashMap<String, GenericFunctionDef>,
    pub(super) generic_types: &'a HashMap<String, GenericTypeDef>,
    pub(super) generic_function_templates: &'a HashMap<String, GenericFunctionTemplate>,
    pub(super) generic_method_templates:
        &'a HashMap<String, Vec<generic_instances::GenericMethodTemplate>>,
    pub(super) method_function_ids: &'a HashMap<String, usize>,
    pub(super) promoted_method_bindings: &'a HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(super) struct_types: &'a HashMap<String, StructTypeDef>,
    pub(super) pointer_types: &'a HashMap<String, TypeId>,
    pub(super) interface_types: &'a HashMap<String, InterfaceTypeDef>,
    pub(super) alias_types: &'a HashMap<String, AliasTypeDef>,
    pub(super) globals: &'a HashMap<String, GlobalBinding>,
    pub(super) method_sets: &'a HashMap<String, Vec<InterfaceMethodDecl>>,
}

impl CompilerEnvironment<'_> {
    pub(super) fn new<'a>(
        imports: ImportContext<'a>,
        symbols: SymbolTables<'a>,
        types: TypeContext<'a>,
        runtime_metadata: RuntimeMetadataContext<'a>,
    ) -> CompilerEnvironment<'a> {
        CompilerEnvironment {
            imported_packages: imports.imported_packages.clone(),
            imported_package_tables: imports.imported_package_tables,
            function_ids: symbols.function_ids,
            function_result_types: symbols.function_result_types,
            function_types: symbols.function_types,
            variadic_functions: symbols.variadic_functions,
            generic_functions: types.generic_functions,
            generic_types: types.generic_types,
            generic_function_templates: types.generic_function_templates,
            generic_method_templates: types.generic_method_templates,
            method_function_ids: symbols.method_function_ids,
            promoted_method_bindings: symbols.promoted_method_bindings,
            struct_types: runtime_metadata.struct_types,
            pointer_types: runtime_metadata.pointer_types,
            interface_types: runtime_metadata.interface_types,
            alias_types: runtime_metadata.alias_types,
            globals: symbols.globals,
            method_sets: symbols.method_sets,
            imports,
            symbols,
            types,
            runtime_metadata,
        }
    }

    #[allow(dead_code)]
    pub(super) fn imported_function_type(&self, package: &str, symbol: &str) -> Option<&str> {
        self.imports
            .imported_package_tables
            .function_types
            .get(&format!("{package}.{symbol}"))
            .map(String::as_str)
    }

    pub(super) fn set_imported_packages(&mut self, imported_packages: HashMap<String, String>) {
        self.imports.imported_packages = imported_packages.clone();
        self.imported_packages = imported_packages;
    }

    pub(super) fn extend_imported_packages(
        &mut self,
        imported_packages: impl IntoIterator<Item = (String, String)>,
    ) {
        for (selector, package_path) in imported_packages {
            self.imports
                .imported_packages
                .insert(selector.clone(), package_path.clone());
            self.imported_packages.insert(selector, package_path);
        }
    }
}

pub(super) struct InstantiationContext<'a> {
    pub(super) instantiation_cache: &'a mut InstantiationCache,
    pub(super) generated_functions: &'a mut GeneratedFunctions,
    pub(super) instantiated_generics: &'a mut generic_instances::InstantiatedGenerics,
    pub(super) generic_instance_namespace: Option<String>,
}

impl InstantiationContext<'_> {
    #[allow(dead_code)]
    pub(super) fn namespace(&self) -> Option<&str> {
        self.generic_instance_namespace.as_deref()
    }
}

pub(super) type GenerationState<'a> = InstantiationContext<'a>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScopeStack {
    pub(super) scopes: Vec<HashMap<String, usize>>,
    pub(super) captured_by_ref: HashSet<String>,
    pub(super) const_scopes: Vec<HashSet<String>>,
    pub(super) const_value_scopes: Vec<HashMap<String, ConstValueInfo>>,
    pub(super) type_scopes: Vec<HashMap<String, String>>,
}

impl ScopeStack {
    pub(super) fn current_scope(&self) -> &HashMap<String, usize> {
        self.scopes.last().expect("scope should exist")
    }

    pub(super) fn current_scope_mut(&mut self) -> &mut HashMap<String, usize> {
        self.scopes.last_mut().expect("scope should exist")
    }

    pub(super) fn current_type_scope_mut(&mut self) -> &mut HashMap<String, String> {
        self.type_scopes
            .last_mut()
            .expect("type scope should exist")
    }

    pub(super) fn current_const_value_scope_mut(&mut self) -> &mut HashMap<String, ConstValueInfo> {
        self.const_value_scopes
            .last_mut()
            .expect("const value scope should exist")
    }

    pub(super) fn lookup_local(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    pub(super) fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.const_scopes.push(HashSet::new());
        self.const_value_scopes.push(HashMap::new());
        self.type_scopes.push(HashMap::new());
    }

    pub(super) fn end_scope(&mut self) {
        self.scopes.pop();
        self.const_scopes.pop();
        self.const_value_scopes.pop();
        self.type_scopes.pop();
    }
}

impl std::ops::Deref for ScopeStack {
    type Target = Vec<HashMap<String, usize>>;

    fn deref(&self) -> &Self::Target {
        &self.scopes
    }
}

impl std::ops::DerefMut for ScopeStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scopes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ControlFlowContext {
    pub(super) in_package_init: bool,
    pub(super) current_result_types: Vec<String>,
    pub(super) current_result_names: Vec<String>,
    pub(super) break_scopes: Vec<BreakContext>,
    pub(super) loops: Vec<LoopContext>,
    pub(super) pending_label: Option<String>,
}

impl ControlFlowContext {
    #[allow(dead_code)]
    pub(super) fn has_pending_label(&self) -> bool {
        self.pending_label.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LoopContext {
    pub(super) label: Option<String>,
    pub(super) continue_jumps: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BreakContext {
    pub(super) label: Option<String>,
    pub(super) break_jumps: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitter_state_allocates_registers_sequentially() {
        let mut emitter = EmitterState {
            next_register: 3,
            ..Default::default()
        };
        assert_eq!(emitter.alloc_register(), 3);
        assert_eq!(emitter.alloc_register(), 4);
        assert_eq!(emitter.next_register, 5);
    }

    #[test]
    fn compiler_environment_reads_imported_function_types() {
        let function_ids = HashMap::new();
        let function_result_types = HashMap::new();
        let function_types = HashMap::new();
        let variadic_functions = HashSet::new();
        let globals = HashMap::new();
        let structs = HashMap::new();
        let interfaces = HashMap::new();
        let pointers = HashMap::new();
        let aliases = HashMap::new();
        let method_function_ids = HashMap::new();
        let promoted_method_bindings = HashMap::new();
        let method_sets = HashMap::new();
        let generic_package_contexts = HashMap::new();
        let imported_function_types =
            HashMap::from([("pkg.Run".to_string(), "func()".to_string())]);
        let generic_function_instances = HashMap::new();
        let imported_package_tables = ImportedPackageTables {
            function_ids: &function_ids,
            generic_function_instances: &generic_function_instances,
            function_result_types: &function_result_types,
            function_types: &imported_function_types,
            variadic_functions: &variadic_functions,
            globals: &globals,
            structs: &structs,
            interfaces: &interfaces,
            pointers: &pointers,
            aliases: &aliases,
            method_function_ids: &method_function_ids,
            promoted_method_bindings: &promoted_method_bindings,
            method_sets: &method_sets,
            generic_package_contexts: &generic_package_contexts,
        };
        let generic_functions = HashMap::new();
        let generic_types = HashMap::new();
        let generic_function_templates = HashMap::new();
        let generic_method_templates = HashMap::new();
        let imports = ImportContext {
            imported_package_tables,
            imported_packages: HashMap::from([("pkg".to_string(), "example.com/pkg".to_string())]),
        };
        let symbols = SymbolTables {
            function_ids: &function_ids,
            function_result_types: &function_result_types,
            function_types: &function_types,
            variadic_functions: &variadic_functions,
            method_function_ids: &method_function_ids,
            promoted_method_bindings: &promoted_method_bindings,
            globals: &globals,
            method_sets: &method_sets,
        };
        let types = TypeContext {
            generic_functions: &generic_functions,
            generic_types: &generic_types,
            generic_function_templates: &generic_function_templates,
            generic_method_templates: &generic_method_templates,
        };
        let runtime_metadata = RuntimeMetadataContext {
            struct_types: &structs,
            pointer_types: &pointers,
            interface_types: &interfaces,
            alias_types: &aliases,
        };
        let env = CompilerEnvironment::new(imports.clone(), symbols, types, runtime_metadata);

        assert_eq!(env.imported_function_type("pkg", "Run"), Some("func()"));
        assert_eq!(imports.package_path("pkg"), Some("example.com/pkg"));
        assert_eq!(env.imports.package_path("pkg"), Some("example.com/pkg"));
    }

    #[test]
    fn symbol_tables_expose_function_and_global_views() {
        let function_ids = HashMap::from([("run".to_string(), 7usize)]);
        let function_result_types = HashMap::from([("run".to_string(), vec!["int".to_string()])]);
        let function_types = HashMap::from([("run".to_string(), "func() int".to_string())]);
        let variadic_functions = HashSet::new();
        let method_function_ids = HashMap::new();
        let promoted_method_bindings = HashMap::new();
        let globals = HashMap::from([(
            "answer".to_string(),
            GlobalBinding {
                index: 3,
                typ: Some("int".to_string()),
                is_const: false,
                const_value: None,
            },
        )]);
        let method_sets = HashMap::new();
        let symbols = SymbolTables {
            function_ids: &function_ids,
            function_result_types: &function_result_types,
            function_types: &function_types,
            variadic_functions: &variadic_functions,
            method_function_ids: &method_function_ids,
            promoted_method_bindings: &promoted_method_bindings,
            globals: &globals,
            method_sets: &method_sets,
        };

        assert_eq!(symbols.function_type("run"), Some("func() int"));
        assert_eq!(symbols.global_index("answer"), Some(3));
    }

    #[test]
    fn type_and_runtime_metadata_contexts_keep_lookup_boundaries_separate() {
        let generic_functions = HashMap::new();
        let generic_types = HashMap::from([(
            "Box".to_string(),
            GenericTypeDef {
                type_params: Vec::new(),
                kind: gowasm_parser::TypeDeclKind::Struct { fields: Vec::new() },
                methods: Vec::new(),
            },
        )]);
        let generic_function_templates = HashMap::new();
        let generic_method_templates = HashMap::new();
        let types = TypeContext {
            generic_functions: &generic_functions,
            generic_types: &generic_types,
            generic_function_templates: &generic_function_templates,
            generic_method_templates: &generic_method_templates,
        };
        let structs = HashMap::from([(
            "Point".to_string(),
            StructTypeDef {
                type_id: TypeId(1000),
                fields: Vec::new(),
            },
        )]);
        let pointers = HashMap::new();
        let interfaces = HashMap::new();
        let aliases = HashMap::new();
        let runtime_metadata = RuntimeMetadataContext {
            struct_types: &structs,
            pointer_types: &pointers,
            interface_types: &interfaces,
            alias_types: &aliases,
        };

        assert!(types.has_generic_type("Box"));
        assert!(runtime_metadata.contains_named_type("Point"));
        assert!(!runtime_metadata.contains_named_type("Box"));
    }

    #[test]
    fn instantiation_context_reports_namespace() {
        let mut instantiation_cache = InstantiationCache::default();
        let mut generated_functions = GeneratedFunctions::new(0);
        let mut instantiated_generics = generic_instances::InstantiatedGenerics::default();
        let state = InstantiationContext {
            instantiation_cache: &mut instantiation_cache,
            generated_functions: &mut generated_functions,
            instantiated_generics: &mut instantiated_generics,
            generic_instance_namespace: Some("pkg/sub".to_string()),
        };

        assert_eq!(state.namespace(), Some("pkg/sub"));
    }

    #[test]
    fn scope_stack_tracks_nested_bindings() {
        let mut scopes = ScopeStack {
            scopes: vec![HashMap::from([("root".to_string(), 1usize)])],
            captured_by_ref: HashSet::new(),
            const_scopes: vec![HashSet::new()],
            const_value_scopes: vec![HashMap::new()],
            type_scopes: vec![HashMap::new()],
        };

        scopes.begin_scope();
        scopes.current_scope_mut().insert("inner".to_string(), 2);
        scopes
            .current_type_scope_mut()
            .insert("inner".to_string(), "int".to_string());

        assert_eq!(scopes.lookup_local("root"), Some(1));
        assert_eq!(scopes.lookup_local("inner"), Some(2));

        scopes.end_scope();
        assert_eq!(scopes.lookup_local("inner"), None);
    }

    #[test]
    fn control_flow_context_tracks_pending_labels() {
        let control = ControlFlowContext {
            in_package_init: false,
            current_result_types: vec!["int".to_string()],
            current_result_names: vec!["value".to_string()],
            break_scopes: Vec::new(),
            loops: Vec::new(),
            pending_label: Some("loop".to_string()),
        };

        assert!(control.has_pending_label());
        assert_eq!(control.current_result_types, vec!["int".to_string()]);
        assert_eq!(control.current_result_names, vec!["value".to_string()]);
    }

    #[test]
    fn compiler_environment_preserves_flat_compatibility_views() {
        let function_ids = HashMap::new();
        let function_result_types = HashMap::new();
        let function_types = HashMap::new();
        let variadic_functions = HashSet::new();
        let globals = HashMap::new();
        let structs = HashMap::new();
        let interfaces = HashMap::new();
        let pointers = HashMap::new();
        let aliases = HashMap::new();
        let method_function_ids = HashMap::new();
        let promoted_method_bindings = HashMap::new();
        let method_sets = HashMap::new();
        let generic_package_contexts = HashMap::new();
        let generic_function_instances = HashMap::new();
        let imported_package_tables = ImportedPackageTables {
            function_ids: &function_ids,
            generic_function_instances: &generic_function_instances,
            function_result_types: &function_result_types,
            function_types: &function_types,
            variadic_functions: &variadic_functions,
            globals: &globals,
            structs: &structs,
            interfaces: &interfaces,
            pointers: &pointers,
            aliases: &aliases,
            method_function_ids: &method_function_ids,
            promoted_method_bindings: &promoted_method_bindings,
            method_sets: &method_sets,
            generic_package_contexts: &generic_package_contexts,
        };
        let generic_functions = HashMap::new();
        let generic_types = HashMap::new();
        let generic_function_templates = HashMap::new();
        let generic_method_templates = HashMap::new();
        let env = CompilerEnvironment::new(
            ImportContext {
                imported_packages: HashMap::from([(
                    "json".to_string(),
                    "encoding/json".to_string(),
                )]),
                imported_package_tables,
            },
            SymbolTables {
                function_ids: &function_ids,
                function_result_types: &function_result_types,
                function_types: &function_types,
                variadic_functions: &variadic_functions,
                method_function_ids: &method_function_ids,
                promoted_method_bindings: &promoted_method_bindings,
                globals: &globals,
                method_sets: &method_sets,
            },
            TypeContext {
                generic_functions: &generic_functions,
                generic_types: &generic_types,
                generic_function_templates: &generic_function_templates,
                generic_method_templates: &generic_method_templates,
            },
            RuntimeMetadataContext {
                struct_types: &structs,
                pointer_types: &pointers,
                interface_types: &interfaces,
                alias_types: &aliases,
            },
        );

        assert_eq!(
            env.imported_packages.get("json"),
            Some(&"encoding/json".to_string())
        );
        assert_eq!(env.imports.package_path("json"), Some("encoding/json"));
        assert!(std::ptr::eq(env.function_ids, env.symbols.function_ids));
        assert!(std::ptr::eq(
            env.runtime_metadata.struct_types,
            env.struct_types
        ));
    }
}
