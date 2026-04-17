use std::collections::{HashMap, HashSet};

use gowasm_parser::{Expr, SourceFile, SourceFileSpans, UnaryOp};
use gowasm_vm::{
    resolve_stdlib_function, stdlib_function_result_types, Function, Instruction,
    InstructionSourceSpan, MethodBinding, TypeId, TYPE_ERROR,
};
use serde::{Deserialize, Serialize};

use crate::types::{
    parse_array_type, parse_channel_type, parse_function_type, parse_map_type, parse_pointer_type,
    AliasTypeDef, InstantiationCache,
};

use crate::builder_support::{
    CompilerEnvironment, ControlFlowContext, EmitterState, GenerationState, ImportContext,
    RuntimeMetadataContext, ScopeStack, SymbolTables, TypeContext,
};
use crate::const_eval::ConstValueInfo;
use crate::consts::rewrite_const_iota_expr;
use crate::package_const_eval::{coerce_package_const_value, infer_package_const_value};
use crate::package_init_order::{order_package_var_inits, PackageInitFile};
use crate::stdlib_function_values::{
    imported_stdlib_selector_function_type, imported_stdlib_selector_value_type,
};
use crate::{
    closures::GeneratedFunctions, types::format_function_type, CompileError, CompiledFunction,
    FunctionBuilder, ImportedPackageTables, InterfaceMethodDecl, InterfaceTypeDef, StructTypeDef,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GlobalBinding {
    pub(crate) index: usize,
    pub(crate) typ: Option<String>,
    pub(crate) is_const: bool,
    pub(crate) const_value: Option<ConstValueInfo>,
}

pub(crate) fn collect_globals<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a SourceFile)>,
    function_types: &HashMap<String, String>,
) -> Result<HashMap<String, GlobalBinding>, CompileError> {
    let mut globals = HashMap::new();
    let mut const_values = HashMap::new();
    let mut next_index = 0usize;

    for (_path, file) in files {
        let imported_packages: HashMap<String, String> = file
            .imports
            .iter()
            .map(|decl| {
                let short = decl
                    .path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&decl.path)
                    .to_string();
                (short, decl.path.clone())
            })
            .collect();
        for constant in &file.consts {
            if globals.contains_key(&constant.name) {
                return Err(CompileError::DuplicateGlobal {
                    package: file.package_name.clone(),
                    name: constant.name.clone(),
                });
            }
            let value = rewrite_const_iota_expr(&constant.value, constant.iota);
            let const_value = infer_package_const_value(&value, &const_values, &imported_packages)
                .and_then(|value| match constant.typ.as_deref() {
                    Some(typ) => coerce_package_const_value(&value, typ),
                    None => Ok(value),
                })?;
            let typ = Some(match constant.typ.as_deref() {
                Some(typ) => typ.to_string(),
                None => const_value.visible_type_name(),
            });
            globals.insert(
                constant.name.clone(),
                GlobalBinding {
                    index: next_index,
                    typ: typ.clone(),
                    is_const: true,
                    const_value: Some(const_value.clone()),
                },
            );
            const_values.insert(constant.name.clone(), const_value);
            next_index += 1;
        }
        for var in &file.vars {
            if globals.contains_key(&var.name) {
                return Err(CompileError::DuplicateGlobal {
                    package: file.package_name.clone(),
                    name: var.name.clone(),
                });
            }
            globals.insert(
                var.name.clone(),
                GlobalBinding {
                    index: next_index,
                    typ: var.typ.clone().or_else(|| {
                        infer_package_var_type(
                            var.value.as_ref(),
                            function_types,
                            &imported_packages,
                        )
                    }),
                    is_const: false,
                    const_value: None,
                },
            );
            next_index += 1;
        }
    }

    Ok(globals)
}

fn infer_package_var_type(
    value: Option<&Expr>,
    function_types: &HashMap<String, String>,
    imported_packages: &HashMap<String, String>,
) -> Option<String> {
    infer_package_expr_type(value?, function_types, imported_packages)
}

fn infer_package_expr_type(
    value: &Expr,
    function_types: &HashMap<String, String>,
    imported_packages: &HashMap<String, String>,
) -> Option<String> {
    match value {
        Expr::IntLiteral(_) => Some("int".into()),
        Expr::BoolLiteral(_) => Some("bool".into()),
        Expr::FloatLiteral(_) => Some("float64".into()),
        Expr::StringLiteral(_) => Some("string".into()),
        Expr::Ident(name) => function_types.get(name).cloned(),
        Expr::Unary { op, expr } => {
            let inner = infer_package_expr_type(expr, function_types, imported_packages)?;
            match op {
                UnaryOp::AddressOf => Some(format!("*{inner}")),
                UnaryOp::Deref => parse_pointer_type(&inner).map(str::to_string),
                UnaryOp::Receive => parse_channel_type(&inner)
                    .map(|channel_type| channel_type.element_type.to_string()),
                UnaryOp::Not => (inner == "bool").then(|| "bool".into()),
                UnaryOp::Negate | UnaryOp::BitNot => Some(inner),
            }
        }
        Expr::ArrayLiteral {
            len, element_type, ..
        } => Some(format!("[{len}]{element_type}")),
        Expr::SliceLiteral { element_type, .. } | Expr::SliceConversion { element_type, .. } => {
            Some(format!("[]{element_type}"))
        }
        Expr::MapLiteral {
            key_type,
            value_type,
            ..
        } => Some(format!("map[{key_type}]{value_type}")),
        Expr::StructLiteral { type_name, .. } => Some(type_name.clone()),
        Expr::New { type_name } => Some(format!("*{type_name}")),
        Expr::Make { type_name, .. }
        | Expr::TypeAssert {
            asserted_type: type_name,
            ..
        } => Some(type_name.clone()),
        Expr::Selector { receiver, field } => {
            let Expr::Ident(receiver_name) = receiver.as_ref() else {
                return None;
            };
            let package_path = imported_packages.get(receiver_name)?;
            imported_stdlib_selector_value_type(package_path, field)
                .or_else(|| imported_stdlib_selector_function_type(package_path, field))
        }
        Expr::FunctionLiteral {
            params,
            result_types,
            ..
        } => Some(format_function_type(
            &params
                .iter()
                .map(|parameter| parameter.typ.clone())
                .collect::<Vec<_>>(),
            result_types,
        )),
        Expr::Index { target, .. } => {
            let target_type = infer_package_expr_type(target, function_types, imported_packages)?;
            if let Some((_, element_type)) = parse_array_type(&target_type) {
                Some(element_type.to_string())
            } else if let Some(element_type) = target_type.strip_prefix("[]") {
                Some(element_type.to_string())
            } else if let Some((_, value_type)) = parse_map_type(&target_type) {
                Some(value_type.to_string())
            } else if target_type == "string" {
                Some("int".into())
            } else {
                None
            }
        }
        Expr::SliceExpr { target, .. } => {
            infer_package_expr_type(target, function_types, imported_packages)
        }
        Expr::Call { callee, .. } => match callee.as_ref() {
            Expr::Ident(name) => match name.as_str() {
                "int" | "byte" | "rune" | "float64" | "string" | "bool" => Some(name.clone()),
                _ => function_types
                    .get(name)
                    .and_then(|function_type| single_package_result_type(function_type)),
            },
            Expr::Selector { receiver, field } => {
                let Expr::Ident(receiver_name) = receiver.as_ref() else {
                    return None;
                };
                let package_path = imported_packages.get(receiver_name)?;
                resolve_stdlib_function(package_path, field)
                    .and_then(stdlib_function_result_types)
                    .and_then(single_result_type)
                    .map(str::to_string)
            }
            _ => None,
        },
        _ => None,
    }
}

fn single_package_result_type(function_type: &str) -> Option<String> {
    let (_, result_types) = parse_function_type(function_type)?;
    (result_types.len() == 1).then(|| result_types[0].clone())
}

fn single_result_type<'a>(result_types: &'a [&'a str]) -> Option<&'a str> {
    (result_types.len() == 1).then_some(result_types[0])
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_package_init_function<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a SourceFile, Option<&'a SourceFileSpans>)>,
    imported_package_tables: ImportedPackageTables<'_>,
    function_ids: &HashMap<String, usize>,
    function_result_types: &HashMap<String, Vec<String>>,
    function_types: &HashMap<String, String>,
    variadic_functions: &HashSet<String>,
    generic_functions: &HashMap<String, crate::GenericFunctionDef>,
    generic_types: &HashMap<String, crate::GenericTypeDef>,
    generic_function_templates: &HashMap<String, crate::GenericFunctionTemplate>,
    generic_method_templates: &HashMap<
        String,
        Vec<crate::generic_instances::GenericMethodTemplate>,
    >,
    instantiation_cache: &mut InstantiationCache,
    generated_functions: &mut GeneratedFunctions,
    instantiated_generics: &mut crate::generic_instances::InstantiatedGenerics,
    method_function_ids: &HashMap<String, usize>,
    promoted_method_bindings: &HashMap<String, crate::symbols::PromotedMethodBindingInfo>,
    struct_types: &HashMap<String, StructTypeDef>,
    pointer_types: &HashMap<String, TypeId>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
    alias_types: &HashMap<String, AliasTypeDef>,
    method_sets: &HashMap<String, Vec<InterfaceMethodDecl>>,
    globals: &HashMap<String, GlobalBinding>,
    init_functions: &[usize],
) -> Result<CompiledFunction, CompileError> {
    let mut files = files
        .into_iter()
        .map(|(path, file, spans)| PackageInitFile { path, file, spans })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(right.path));

    let mut builder = FunctionBuilder {
        emitter: EmitterState::default(),
        env: CompilerEnvironment::new(
            ImportContext {
                imported_packages: Default::default(),
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
            generic_instance_namespace: None,
        },
        scopes: ScopeStack {
            scopes: vec![HashMap::new()],
            captured_by_ref: std::collections::HashSet::new(),
            const_scopes: vec![std::collections::HashSet::new()],
            const_value_scopes: vec![HashMap::new()],
            type_scopes: vec![HashMap::new()],
        },
        control: ControlFlowContext {
            in_package_init: true,
            current_result_types: Vec::new(),
            current_result_names: Vec::new(),
            break_scopes: Vec::new(),
            loops: Vec::new(),
            pending_label: None,
        },
    };

    for file in &files {
        builder.env.extend_imported_packages(
            file.file
                .imports
                .iter()
                .map(|decl| (decl.selector().to_string(), decl.path.clone())),
        );
    }

    for file in &files {
        let path = file.path;
        builder.emitter.default_source_path = Some(path.to_string());
        for (const_index, constant) in file.file.consts.iter().enumerate() {
            let binding = globals
                .get(&constant.name)
                .expect("package const binding should exist");
            let span = file
                .spans
                .and_then(|spans| spans.consts.get(const_index))
                .copied();
            builder.with_instruction_span(
                span.map(|span| InstructionSourceSpan {
                    path: path.to_string(),
                    start: span.start,
                    end: span.end,
                }),
                |builder| {
                    compile_package_const_init(
                        builder,
                        &constant.name,
                        binding.index,
                        constant.typ.as_deref(),
                        &constant.value,
                        constant.iota,
                    )
                },
            )?;
        }
    }

    for var_init in order_package_var_inits(&files)? {
        let binding = globals
            .get(&var_init.decl.name)
            .expect("package var binding should exist");
        let path = var_init.path;
        builder.emitter.default_source_path = Some(path.to_string());
        let span = var_init
            .spans
            .and_then(|spans| spans.vars.get(var_init.index))
            .copied();
        builder.with_instruction_span(
            span.map(|span| InstructionSourceSpan {
                path: path.to_string(),
                start: span.start,
                end: span.end,
            }),
            |builder| {
                compile_package_var_init(
                    builder,
                    binding.index,
                    var_init.decl.typ.as_deref(),
                    var_init.decl.value.as_ref(),
                )
            },
        )?;
    }
    for function in init_functions {
        builder.emitter.code.push(Instruction::CallFunction {
            function: *function,
            args: vec![],
            dst: None,
        });
    }

    builder.emitter.code.push(Instruction::Return { src: None });
    let (code, debug_info) = builder.emitter.code.into_parts();
    Ok(CompiledFunction {
        function: Function {
            name: "__gowasm_init".into(),
            param_count: 0,
            register_count: builder.emitter.next_register,
            code,
        },
        debug_info,
    })
}

pub(crate) fn compile_package_entry_function(
    main_function: usize,
    init_function: usize,
) -> Function {
    Function {
        name: "__gowasm_entry".into(),
        param_count: 0,
        register_count: 0,
        code: vec![
            Instruction::CallFunction {
                function: init_function,
                args: vec![],
                dst: None,
            },
            Instruction::CallFunction {
                function: main_function,
                args: vec![],
                dst: None,
            },
            Instruction::Return { src: None },
        ],
    }
}

pub(crate) fn compile_workspace_init_function(init_functions: &[usize]) -> Function {
    let mut code = Vec::with_capacity(init_functions.len() + 1);
    for function in init_functions {
        code.push(Instruction::CallFunction {
            function: *function,
            args: vec![],
            dst: None,
        });
    }
    code.push(Instruction::Return { src: None });
    Function {
        name: "__gowasm_workspace_init".into(),
        param_count: 0,
        register_count: 0,
        code,
    }
}

pub(crate) fn compile_builtin_error_method(function: usize) -> (Function, MethodBinding) {
    (
        Function {
            name: "__gowasm_builtin_error.Error".into(),
            param_count: 1,
            register_count: 2,
            code: vec![
                Instruction::LoadErrorMessage { dst: 1, src: 0 },
                Instruction::Return { src: Some(1) },
            ],
        },
        MethodBinding {
            receiver_type: TYPE_ERROR,
            target_receiver_type: TYPE_ERROR,
            name: "Error".into(),
            function,
            param_types: Vec::new(),
            result_types: vec!["string".into()],
            promoted_fields: Vec::new(),
        },
    )
}

pub(crate) fn compile_builtin_context_cancel_helper() -> Function {
    let function = resolve_stdlib_function("context", "__cancel")
        .expect("context.__cancel stdlib helper should exist");
    Function {
        name: "__gowasm_context_cancel".into(),
        param_count: 1,
        register_count: 1,
        code: vec![
            Instruction::CallStdlib {
                function,
                args: vec![0],
                dst: None,
            },
            Instruction::Return { src: None },
        ],
    }
}

fn compile_package_var_init(
    builder: &mut FunctionBuilder<'_>,
    global: usize,
    typ: Option<&str>,
    value: Option<&Expr>,
) -> Result<(), CompileError> {
    let src = builder.alloc_register();
    match (typ, value) {
        (_, Some(value)) => {
            builder.validate_assignable_type(typ, value)?;
            builder.compile_expr_into_with_hint(src, value, typ)?
        }
        (Some(typ), None) => builder.compile_zero_value(src, typ)?,
        (None, None) => {
            return Err(CompileError::Unsupported {
                detail: format!("package var {global} must include a type or initializer"),
            });
        }
    }
    builder
        .emitter
        .code
        .push(Instruction::StoreGlobal { global, src });
    Ok(())
}

fn compile_package_const_init(
    builder: &mut FunctionBuilder<'_>,
    name: &str,
    global: usize,
    typ: Option<&str>,
    value: &Expr,
    iota: usize,
) -> Result<(), CompileError> {
    let value = rewrite_const_iota_expr(value, iota);
    builder.validate_const_initializer(typ, &value)?;
    let const_value: ConstValueInfo =
        builder
            .eval_const_expr(&value)
            .and_then(|value| match typ {
                Some(typ) => builder.coerce_const_value_info(&value, typ),
                None => Ok(value),
            })?;
    let src = builder.alloc_register();
    builder.compile_expr_into_with_hint(src, &value, typ)?;
    builder
        .emitter
        .code
        .push(Instruction::StoreGlobal { global, src });
    builder.current_scope_mut().insert(name.to_string(), src);
    builder
        .scopes
        .const_scopes
        .last_mut()
        .expect("const scope should exist")
        .insert(name.to_string());
    builder
        .scopes
        .current_const_value_scope_mut()
        .insert(name.to_string(), const_value.clone());
    builder
        .current_type_scope_mut()
        .insert(name.to_string(), const_value.visible_type_name());
    Ok(())
}
