use std::collections::{HashMap, HashSet};

use super::*;
use gowasm_vm::{Function, FunctionDebugInfo};

#[derive(Clone, Copy)]
pub(super) struct ImportedPackageTables<'a> {
    pub(super) function_ids: &'a HashMap<String, usize>,
    pub(super) generic_function_instances: &'a HashMap<types::InstanceKey, usize>,
    pub(super) function_result_types: &'a HashMap<String, Vec<String>>,
    pub(super) function_types: &'a HashMap<String, String>,
    pub(super) variadic_functions: &'a HashSet<String>,
    pub(super) globals: &'a HashMap<String, GlobalBinding>,
    pub(super) structs: &'a HashMap<String, StructTypeDef>,
    pub(super) interfaces: &'a HashMap<String, InterfaceTypeDef>,
    pub(super) pointers: &'a HashMap<String, TypeId>,
    pub(super) aliases: &'a HashMap<String, AliasTypeDef>,
    pub(super) method_function_ids: &'a HashMap<String, usize>,
    pub(super) promoted_method_bindings: &'a HashMap<String, symbols::PromotedMethodBindingInfo>,
    pub(super) method_sets: &'a HashMap<String, Vec<InterfaceMethodDecl>>,
    pub(super) generic_package_contexts:
        &'a HashMap<String, std::sync::Arc<imported_generics::ImportedGenericPackageContext>>,
}

pub(super) struct FunctionBuilder<'a> {
    pub(super) emitter: EmitterState,
    pub(super) env: CompilerEnvironment<'a>,
    pub(super) generation: GenerationState<'a>,
    pub(super) scopes: ScopeStack,
    pub(super) control: ControlFlowContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GenericFunctionTemplate {
    pub(super) decl: FunctionDecl,
    pub(super) source_path: String,
    pub(super) spans: FunctionSourceSpans,
    pub(super) imported_packages: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(super) struct CompiledFunction {
    pub(super) function: Function,
    pub(super) debug_info: FunctionDebugInfo,
}

impl CompiledFunction {
    pub(super) fn without_debug(function: Function) -> Self {
        let debug_info = FunctionDebugInfo::empty(function.code.len());
        Self {
            function,
            debug_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ImportedPackageTables;
    use std::collections::HashMap;

    #[test]
    fn imported_package_tables_can_read_registered_function_type() {
        let function_ids = HashMap::new();
        let generic_function_instances = HashMap::new();
        let function_result_types = HashMap::new();
        let function_types = HashMap::from([("pkg.Run".to_string(), "func()".to_string())]);
        let variadic_functions = std::collections::HashSet::new();
        let globals = HashMap::new();
        let structs = HashMap::new();
        let interfaces = HashMap::new();
        let pointers = HashMap::new();
        let aliases = HashMap::new();
        let method_function_ids = HashMap::new();
        let promoted_method_bindings = HashMap::new();
        let method_sets = HashMap::new();
        let generic_package_contexts = HashMap::new();
        let tables = ImportedPackageTables {
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

        assert_eq!(
            tables.function_types.get("pkg.Run"),
            Some(&"func()".to_string())
        );
    }
}
