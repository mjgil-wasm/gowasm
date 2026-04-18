use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::generic_instances::InstantiatedGenerics;
use super::imported_generics::ImportedGenericPackageContext;
use super::program::{
    collect_generic_function_templates, collect_generic_method_templates, compile_function,
    is_generic_template_function,
};
use super::{
    collect_globals, collect_symbols, collect_type_tables, compile_package_init_function,
    CompiledFunction, GlobalBinding, ImportedPackageTables,
};
use gowasm_parser::{parse_source_file, SourceFile};
use gowasm_vm::{Function, FunctionDebugInfo, Instruction, TypeId};

struct EmptyImports {
    function_ids: HashMap<String, usize>,
    function_result_types: HashMap<String, Vec<String>>,
    function_types: HashMap<String, String>,
    variadic_functions: HashSet<String>,
    generic_function_instances: HashMap<super::types::InstanceKey, usize>,
    globals: HashMap<String, GlobalBinding>,
    structs: HashMap<String, super::StructTypeDef>,
    interfaces: HashMap<String, super::InterfaceTypeDef>,
    pointers: HashMap<String, TypeId>,
    aliases: HashMap<String, super::AliasTypeDef>,
    method_function_ids: HashMap<String, usize>,
    promoted_method_bindings: HashMap<String, super::symbols::PromotedMethodBindingInfo>,
    method_sets: HashMap<String, Vec<gowasm_parser::InterfaceMethodDecl>>,
    generic_package_contexts: HashMap<String, Arc<ImportedGenericPackageContext>>,
}

impl EmptyImports {
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

fn parse_file(source: &str) -> SourceFile {
    parse_source_file(source).expect("source should parse")
}

fn compile_single_function(source: &str, function_name: &str) -> (CompiledFunction, Vec<Function>) {
    let file = parse_file(source);
    let empty_imports = EmptyImports {
        function_ids: HashMap::new(),
        function_result_types: HashMap::new(),
        function_types: HashMap::new(),
        variadic_functions: HashSet::new(),
        generic_function_instances: HashMap::new(),
        globals: HashMap::new(),
        structs: HashMap::new(),
        interfaces: HashMap::new(),
        pointers: HashMap::new(),
        aliases: HashMap::new(),
        method_function_ids: HashMap::new(),
        promoted_method_bindings: HashMap::new(),
        method_sets: HashMap::new(),
        generic_package_contexts: HashMap::new(),
    };
    let mut type_tables =
        collect_type_tables(std::iter::once(("main.go", &file))).expect("types should collect");
    let generic_function_templates = collect_generic_function_templates(std::iter::once((
        "main.go",
        &file,
        None::<&gowasm_parser::SourceFileSpans>,
    )));
    let generic_method_templates = collect_generic_method_templates(
        std::iter::once(("main.go", &file, None::<&gowasm_parser::SourceFileSpans>)),
        &type_tables,
    );
    let layout = collect_symbols(
        std::iter::once(("main.go", &file)),
        &type_tables.structs,
        &type_tables.pointers,
        &type_tables.aliases,
        &type_tables.generic_types,
    )
    .expect("symbols should collect");
    let globals = collect_globals(std::iter::once(("main.go", &file)), &layout.function_types)
        .expect("globals should collect");
    let mut instantiated_generics = InstantiatedGenerics::new(
        &type_tables.structs,
        &type_tables.interfaces,
        &type_tables.aliases,
        &type_tables.pointers,
    );
    let concrete_function_count = file
        .functions
        .iter()
        .filter(|function| !is_generic_template_function(function, &type_tables.generic_types))
        .count();
    let mut generated_functions = super::GeneratedFunctions::new(concrete_function_count);
    let function = file
        .functions
        .iter()
        .find(|function| function.name == function_name)
        .expect("named function should exist");
    let compiled = compile_function(
        &file,
        "main.go",
        None,
        function,
        empty_imports.tables(),
        &layout.function_ids,
        &layout.function_result_types,
        &layout.function_types,
        &layout.variadic_functions,
        &type_tables.generic_functions,
        &type_tables.generic_types,
        &generic_function_templates,
        &generic_method_templates,
        &mut type_tables.instantiation_cache,
        &mut generated_functions,
        &mut instantiated_generics,
        &layout.method_function_ids,
        &layout.promoted_method_bindings,
        &type_tables.structs,
        &type_tables.pointers,
        &type_tables.interfaces,
        &type_tables.aliases,
        &layout.method_sets,
        &globals,
    )
    .expect("function should compile");
    let mut generated = Vec::new();
    let mut debug_infos: Vec<FunctionDebugInfo> = Vec::new();
    generated_functions.append_into(&mut generated, &mut debug_infos);
    (compiled, generated)
}

fn compile_single_package_init(source: &str) -> CompiledFunction {
    let file = parse_file(source);
    let empty_imports = EmptyImports {
        function_ids: HashMap::new(),
        function_result_types: HashMap::new(),
        function_types: HashMap::new(),
        variadic_functions: HashSet::new(),
        generic_function_instances: HashMap::new(),
        globals: HashMap::new(),
        structs: HashMap::new(),
        interfaces: HashMap::new(),
        pointers: HashMap::new(),
        aliases: HashMap::new(),
        method_function_ids: HashMap::new(),
        promoted_method_bindings: HashMap::new(),
        method_sets: HashMap::new(),
        generic_package_contexts: HashMap::new(),
    };
    let mut type_tables =
        collect_type_tables(std::iter::once(("main.go", &file))).expect("types should collect");
    let generic_function_templates = collect_generic_function_templates(std::iter::once((
        "main.go",
        &file,
        None::<&gowasm_parser::SourceFileSpans>,
    )));
    let generic_method_templates = collect_generic_method_templates(
        std::iter::once(("main.go", &file, None::<&gowasm_parser::SourceFileSpans>)),
        &type_tables,
    );
    let layout = collect_symbols(
        std::iter::once(("main.go", &file)),
        &type_tables.structs,
        &type_tables.pointers,
        &type_tables.aliases,
        &type_tables.generic_types,
    )
    .expect("symbols should collect");
    let globals = collect_globals(std::iter::once(("main.go", &file)), &layout.function_types)
        .expect("globals should collect");
    let mut instantiated_generics = InstantiatedGenerics::new(
        &type_tables.structs,
        &type_tables.interfaces,
        &type_tables.aliases,
        &type_tables.pointers,
    );
    let concrete_function_count = file
        .functions
        .iter()
        .filter(|function| !is_generic_template_function(function, &type_tables.generic_types))
        .count();
    let mut generated_functions = super::GeneratedFunctions::new(concrete_function_count);
    compile_package_init_function(
        std::iter::once(("main.go", &file, None::<&gowasm_parser::SourceFileSpans>)),
        empty_imports.tables(),
        &layout.function_ids,
        &layout.function_result_types,
        &layout.function_types,
        &layout.variadic_functions,
        &type_tables.generic_functions,
        &type_tables.generic_types,
        &generic_function_templates,
        &generic_method_templates,
        &mut type_tables.instantiation_cache,
        &mut generated_functions,
        &mut instantiated_generics,
        &layout.method_function_ids,
        &layout.promoted_method_bindings,
        &type_tables.structs,
        &type_tables.pointers,
        &type_tables.interfaces,
        &type_tables.aliases,
        &layout.method_sets,
        &globals,
        &layout.init_functions,
    )
    .expect("package init should compile")
}

#[test]
fn function_builder_contexts_preserve_named_function_bytecode() {
    let source = r#"
package main

func add(a int, b int) int {
    return a + b
}

func main() {}
"#;

    let (compiled, generated) = compile_single_function(source, "add");
    assert!(generated.is_empty());
    assert_eq!(compiled.function.name, "add");
    assert_eq!(compiled.function.register_count, 3);
    assert_eq!(
        compiled.function.code,
        vec![
            Instruction::Add {
                dst: 2,
                left: 0,
                right: 1,
            },
            Instruction::Return { src: Some(2) },
            Instruction::Return { src: None },
        ]
    );
}

#[test]
fn function_builder_contexts_preserve_literal_return_bytecode() {
    let source = r#"
package main

func answer() int {
    return 7
}

func main() {}
"#;

    let (compiled, generated) = compile_single_function(source, "answer");
    assert!(generated.is_empty());
    assert_eq!(compiled.function.name, "answer");
    assert_eq!(compiled.function.register_count, 1);
    assert_eq!(
        compiled.function.code,
        vec![
            Instruction::LoadInt { dst: 0, value: 7 },
            Instruction::Return { src: Some(0) },
            Instruction::Return { src: None },
        ]
    );
}

#[test]
fn function_builder_contexts_preserve_package_init_bytecode() {
    let source = r#"
package main

var answer = 7

func main() {}
"#;

    let init = compile_single_package_init(source);
    assert_eq!(init.function.name, "__gowasm_init");
    assert_eq!(init.function.register_count, 1);
    assert_eq!(
        init.function.code,
        vec![
            Instruction::LoadInt { dst: 0, value: 7 },
            Instruction::StoreGlobal { global: 0, src: 0 },
            Instruction::Return { src: None },
        ]
    );
}

#[test]
fn function_builder_contexts_preserve_generated_closure_bytecode() {
    let source = r#"
package main

func main() {
    run := func() int {
        return 1
    }
    _ = run
}
"#;

    let (compiled, generated) = compile_single_function(source, "main");
    assert_eq!(compiled.function.name, "main");
    assert_eq!(generated.len(), 1);
    let closure = &generated[0];
    assert_eq!(closure.name, "__gowasm_closure$1");
    assert_eq!(closure.register_count, 1);
    assert_eq!(
        closure.code,
        vec![
            Instruction::LoadInt { dst: 0, value: 1 },
            Instruction::Return { src: Some(0) },
            Instruction::Return { src: None },
        ]
    );
}
