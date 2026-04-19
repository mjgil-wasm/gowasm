use std::collections::{HashMap, HashSet};

use crate::types::display_type;
use gowasm_lexer::Span;
use gowasm_parser::{
    AssignTarget, BinaryOp, BoundFunctionSourceSpans, Expr, FunctionDecl, FunctionSourceSpans,
    InterfaceMethodDecl, SelectCase, SourceFile, Stmt, SwitchCase, TypeSwitchCase, UnaryOp,
};
use gowasm_vm::{
    register_program_debug_info, register_program_type_inventory, resolve_stdlib_constant,
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_value,
    stdlib_function_mutates_first_arg, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
    stdlib_function_variadic_param_type, Function, FunctionDebugInfo, Instruction,
    InstructionSourceSpan, Program, ProgramDebugInfo, SourceFileDebugInfo, StdlibConstantValue,
    StdlibFunctionId, StdlibValueInit, TypeId, TYPE_POINTER,
};
mod assignment_support;
mod assignments;
mod builder_support;
mod builtins;
mod call_validation;
mod calls;
mod capture_analyzer;
mod closures;
mod comparisons;
mod compile_error;
mod compile_error_context;
mod compiler_context;
mod compiler_phase;
mod concurrency;
mod const_eval;
mod const_lowering;
mod consts;
mod control_flow;
mod data;
mod defers;
mod diagnostics;
mod embedding;
mod generic_instances;
mod generic_substitute;
mod generics;
mod globals;
mod import_resolution;
mod imported_calls;
mod imported_generics;
mod int_ops;
mod interface_satisfaction;
mod lookup_support;
mod method_values;
mod module_graph_validation;
mod multi_calls;
mod package_const_eval;
mod package_init_order;
mod pointers;
mod program;
mod returns;
mod runtime_type_inventory;
mod runtime_visibility;
mod selects;
mod stdlib_function_values;
mod struct_visibility;
mod switch;
mod symbols;
mod typed_lowering;
mod types;
mod workspace;
mod workspace_artifact_exports;
mod workspace_artifacts;

use builder_support::{
    BreakContext, CompilerEnvironment, ControlFlowContext, EmitterState, GenerationState,
    ImportContext, InstructionBuffer, LoopContext, RuntimeMetadataContext, ScopeStack,
    SymbolTables, TypeContext,
};
use closures::{collect_direct_by_ref_captures, GeneratedFunctions};
pub use compile_error::CompileError;
pub(crate) use compile_error_context::{
    clear_last_compile_error_context, record_compile_error_context,
};
pub use compile_error_context::{take_last_compile_error_context, CompileErrorContext};
use compiler_context::{
    CompiledFunction, FunctionBuilder, GenericFunctionTemplate, ImportedPackageTables,
};
pub(crate) use compiler_phase::{CompilerPhase, PhaseFailure};
use globals::{
    collect_globals, compile_builtin_context_cancel_helper, compile_builtin_error_method,
    compile_package_entry_function, compile_package_init_function, compile_workspace_init_function,
    GlobalBinding,
};
pub use import_resolution::{
    module_cache_source_path, module_source_key_for_path, resolve_workspace_rebuild_graph,
    ModuleSourceKey, PackageSourceOrigin, WorkspacePackageNode, WorkspaceRebuildGraph,
};
pub use program::{
    compile_file, compile_workspace, compile_workspace_with_graph,
    recompile_workspace_affected_packages, CompiledWorkspace,
};
use runtime_type_inventory::{build_package_type_inventory, merge_program_type_inventories};
use symbols::collect_symbols;
use types::{
    collect_type_tables, function_signatures_match, lower_compare_op, offset_user_type_ids,
    parse_array_type, parse_channel_type, parse_function_type, parse_map_type, parse_pointer_type,
    user_type_id_span, AliasTypeDef, GenericFunctionDef, GenericTypeDef, InstantiationCache,
    InterfaceTypeDef, StructTypeDef,
};
use workspace::ParsedFile;
pub use workspace::{compile_source, SourceInput};

impl FunctionBuilder<'_> {
    fn pointer_runtime_type(&self, pointee_type: &str) -> TypeId {
        self.instantiated_pointer_type(&format!("*{pointee_type}"))
            .unwrap_or(TYPE_POINTER)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        let previous_span = self.emitter.code.active_span();
        if let Some(span) = self.stmt_instruction_span(stmt) {
            self.emitter.code.set_active_span(Some(span));
        }
        let result = self.compile_stmt_inner(stmt);
        self.emitter.code.set_active_span(previous_span);
        result
    }

    fn compile_stmt_inner(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expr(expr) => self.compile_expr_stmt(expr),
            Stmt::VarDecl { name, typ, value } => {
                self.compile_var_decl(name, typ.as_deref(), value.as_ref())
            }
            Stmt::ConstDecl {
                name,
                typ,
                value,
                iota,
            } => self.compile_const_decl(name, typ.as_deref(), value, *iota),
            Stmt::ConstGroup { decls } => {
                for decl in decls {
                    self.compile_const_decl(
                        &decl.name,
                        decl.typ.as_deref(),
                        &decl.value,
                        decl.iota,
                    )?;
                }
                Ok(())
            }
            Stmt::ShortVarDecl { name, value } => self.compile_short_var_decl(name, value),
            Stmt::ShortVarDeclPair {
                first,
                second,
                value,
            } => self.compile_short_var_decl_pair(first, second, value),
            Stmt::ShortVarDeclTriple {
                first,
                second,
                third,
                value,
            } => self.compile_short_var_decl_triple(first, second, third, value),
            Stmt::ShortVarDeclQuad {
                first,
                second,
                third,
                fourth,
                value,
            } => self.compile_short_var_decl_quad(first, second, third, fourth, value),
            Stmt::ShortVarDeclList { names, values } => {
                self.compile_short_var_decl_list(names, values)
            }
            Stmt::Assign { target, value } => self.compile_assign(target, value),
            Stmt::AssignPair {
                first,
                second,
                value,
            } => self.compile_assign_pair(first, second, value),
            Stmt::AssignTriple {
                first,
                second,
                third,
                value,
            } => self.compile_assign_triple(first, second, third, value),
            Stmt::AssignQuad {
                first,
                second,
                third,
                fourth,
                value,
            } => self.compile_assign_quad(first, second, third, fourth, value),
            Stmt::AssignList { targets, values } => self.compile_assign_list(targets, values),
            Stmt::Increment { name } => self.compile_increment(name),
            Stmt::Decrement { name } => self.compile_decrement(name),
            Stmt::If {
                init,
                condition,
                then_body,
                else_body,
            } => self.compile_if(init.as_deref(), condition, then_body, else_body.as_deref()),
            Stmt::For {
                init,
                condition,
                post,
                body,
            } => self.compile_for(init.as_deref(), condition.as_ref(), post.as_deref(), body),
            Stmt::RangeFor {
                key,
                value,
                assign,
                expr,
                body,
            } => self.compile_range_for(key, value.as_deref(), *assign, expr, body),
            Stmt::Switch {
                init,
                expr,
                cases,
                default,
                default_index,
                default_fallthrough,
            } => self.compile_switch(
                init.as_deref(),
                expr.as_ref(),
                cases,
                default.as_deref(),
                *default_index,
                *default_fallthrough,
            ),
            Stmt::TypeSwitch {
                init,
                binding,
                expr,
                cases,
                default,
                default_index,
            } => self.compile_type_switch(
                init.as_deref(),
                binding.as_deref(),
                expr,
                cases,
                default.as_deref(),
                *default_index,
            ),
            Stmt::Select { cases, default } => self.compile_select(cases, default.as_deref()),
            Stmt::Send { chan, value } => self.compile_send_stmt(chan, value),
            Stmt::Go { call } => self.compile_go_stmt(call),
            Stmt::Defer { call } => self.compile_defer(call),
            Stmt::Labeled { label, stmt } => self.compile_labeled(label, stmt),
            Stmt::Break { label } => self.compile_break(label.as_deref()),
            Stmt::Continue { label } => self.compile_continue(label.as_deref()),
            Stmt::Return(values) => self.compile_return(values),
        }
    }

    fn instruction_span(&self, span: Span) -> Option<InstructionSourceSpan> {
        self.emitter
            .default_source_path
            .as_ref()
            .map(|path| InstructionSourceSpan {
                path: path.clone(),
                start: span.start,
                end: span.end,
            })
    }

    fn stmt_instruction_span(&self, stmt: &Stmt) -> Option<InstructionSourceSpan> {
        self.emitter
            .source_spans
            .as_ref()
            .and_then(|source_spans| source_spans.stmt_span(stmt))
            .and_then(|span| self.instruction_span(span))
    }

    fn with_instruction_span<T>(
        &mut self,
        span: Option<InstructionSourceSpan>,
        apply: impl FnOnce(&mut Self) -> Result<T, CompileError>,
    ) -> Result<T, CompileError> {
        let previous_span = self.emitter.code.active_span();
        if span.is_some() {
            self.emitter.code.set_active_span(span);
        }
        let result = apply(self);
        self.emitter.code.set_active_span(previous_span);
        result
    }

    fn annotate_compile_error(&self, error: CompileError) -> CompileError {
        if let Some(span) = self.emitter.code.active_span() {
            record_compile_error_context(
                span.path,
                Span {
                    start: span.start,
                    end: span.end,
                },
            );
        }
        error
    }

    fn unsupported_with_active_span(&self, detail: impl Into<String>) -> CompileError {
        self.annotate_compile_error(CompileError::Unsupported {
            detail: detail.into(),
        })
    }

    fn compile_value_expr(&mut self, expr: &Expr) -> Result<usize, CompileError> {
        if let Expr::Ident(name) = expr {
            if let Some(register) = self.lookup_local(name) {
                if self.scopes.captured_by_ref.contains(name) {
                    let value = self.alloc_register();
                    self.emitter.code.push(Instruction::Deref {
                        dst: value,
                        src: register,
                    });
                    return Ok(value);
                }
                return Ok(register);
            }
            if let Some(global) = self.lookup_global(name) {
                let register = self.alloc_register();
                self.emitter.code.push(Instruction::LoadGlobal {
                    dst: register,
                    global,
                });
                return Ok(register);
            }
            if let Some(function) = self.env.function_ids.get(name).copied() {
                let register = self.alloc_register();
                self.emitter.code.push(Instruction::MakeClosure {
                    dst: register,
                    concrete_type: self
                        .env
                        .function_types
                        .get(name)
                        .map(|typ| self.lower_runtime_concrete_type(typ))
                        .transpose()?,
                    function,
                    captures: Vec::new(),
                });
                return Ok(register);
            }
            return Err(CompileError::UnknownIdentifier { name: name.clone() });
        }

        let register = self.alloc_register();
        self.compile_expr_into(register, expr)?;
        Ok(register)
    }

    fn compile_expr_into(&mut self, dst: usize, expr: &Expr) -> Result<(), CompileError> {
        self.ensure_expr_runtime_types(expr)?;
        if self.compile_const_expr_value(dst, expr, None)? {
            return Ok(());
        }
        match expr {
            Expr::IntLiteral(value) => {
                self.emitter
                    .code
                    .push(Instruction::LoadInt { dst, value: *value });
                Ok(())
            }
            Expr::FloatLiteral(bits) => {
                self.emitter.code.push(Instruction::LoadFloat {
                    dst,
                    value: gowasm_vm::Float64(f64::from_bits(*bits)),
                });
                Ok(())
            }
            Expr::BoolLiteral(value) => {
                self.emitter
                    .code
                    .push(Instruction::LoadBool { dst, value: *value });
                Ok(())
            }
            Expr::NilLiteral => {
                self.emitter.code.push(Instruction::LoadNil { dst });
                Ok(())
            }
            Expr::StringLiteral(value) => {
                self.emitter.code.push(Instruction::LoadString {
                    dst,
                    value: value.clone(),
                });
                Ok(())
            }
            Expr::ArrayLiteral {
                len,
                element_type: _,
                elements,
            } => {
                if *len != elements.len() {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "array literal length {len} must match {} element(s) in the current subset",
                            elements.len()
                        ),
                    });
                }
                let items = self.compile_collection_items(elements)?;
                self.emitter.code.push(Instruction::MakeArray {
                    dst,
                    concrete_type: self
                        .infer_expr_type_name(expr)
                        .map(|typ| self.lower_runtime_concrete_type(&typ))
                        .transpose()?,
                    items,
                });
                Ok(())
            }
            Expr::SliceLiteral {
                element_type: _,
                elements,
            } => {
                let items = self.compile_collection_items(elements)?;
                self.emitter.code.push(Instruction::MakeSlice {
                    dst,
                    concrete_type: self
                        .infer_expr_type_name(expr)
                        .map(|typ| self.lower_runtime_concrete_type(&typ))
                        .transpose()?,
                    items,
                });
                Ok(())
            }
            Expr::SliceConversion { element_type, expr } => {
                let Some(source_type) = self.infer_expr_type_name(expr) else {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "cannot convert expression to `[]{}` in the current subset",
                            element_type
                        ),
                    });
                };
                let source_underlying = self.instantiated_underlying_type_name(&source_type);
                if source_underlying != "string" {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "cannot convert expression of type `{}` to `[]{}` in the current subset",
                            display_type(&source_type),
                            element_type
                        ),
                    });
                }
                let src = self.compile_value_expr(expr)?;
                match element_type.as_str() {
                    "byte" => {
                        self.emitter
                            .code
                            .push(Instruction::ConvertToByteSlice { dst, src });
                    }
                    "rune" => {
                        self.emitter
                            .code
                            .push(Instruction::ConvertToRuneSlice { dst, src });
                    }
                    _ => {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "slice conversion to []{element_type} is not supported"
                            ),
                        });
                    }
                }
                Ok(())
            }
            Expr::MapLiteral {
                key_type,
                value_type,
                entries,
            } => self.compile_map_literal(dst, key_type, value_type, entries),
            Expr::StructLiteral { type_name, fields } => {
                self.compile_struct_literal(dst, type_name, fields)
            }
            Expr::Index { target, index } => {
                let target = self.compile_value_expr(target)?;
                let index = self.compile_value_expr(index)?;
                self.emitter
                    .code
                    .push(Instruction::Index { dst, target, index });
                Ok(())
            }
            Expr::SliceExpr { target, low, high } => {
                let target = self.compile_value_expr(target)?;
                let low = low
                    .as_ref()
                    .map(|e| self.compile_value_expr(e))
                    .transpose()?;
                let high = high
                    .as_ref()
                    .map(|e| self.compile_value_expr(e))
                    .transpose()?;
                self.emitter.code.push(Instruction::Slice {
                    dst,
                    target,
                    low,
                    high,
                });
                Ok(())
            }
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(package_path) = self.env.imported_packages.get(receiver_name) {
                        let package_path = package_path.clone();
                        return self.compile_imported_package_selector(
                            dst,
                            receiver_name,
                            &package_path,
                            field,
                        );
                    }
                }
                if self.selector_is_method_expression(receiver, field) {
                    return self.compile_method_expression_value(dst, receiver, field);
                }
                if self.lookup_interface_type(receiver).is_some() {
                    return self.compile_interface_method_value(dst, receiver, field);
                }
                if self.lookup_stdlib_value_method(receiver, field).is_some() {
                    return self.compile_stdlib_method_value(dst, receiver, field);
                }
                if let Some(function) = self.lookup_concrete_method_function(receiver, field) {
                    let receiver_reg = self.compile_method_receiver(receiver, field)?;
                    self.emitter.code.push(Instruction::MakeClosure {
                        dst,
                        concrete_type: self
                            .infer_expr_type_name(expr)
                            .map(|typ| self.lower_runtime_concrete_type(&typ))
                            .transpose()?,
                        function,
                        captures: vec![receiver_reg],
                    });
                    return Ok(());
                }
                self.compile_selector_expr(dst, receiver, field)
            }
            Expr::TypeAssert {
                expr,
                asserted_type,
            } => {
                let src = self.compile_value_expr(expr)?;
                self.ensure_runtime_visible_type(asserted_type)?;
                let target = self.lower_type_assert_target(asserted_type)?;
                self.emitter
                    .code
                    .push(Instruction::AssertType { dst, src, target });
                Ok(())
            }
            Expr::New { type_name } => self.compile_new_expr(dst, type_name),
            Expr::Make { type_name, args } => self.compile_make_expr(dst, type_name, args),
            Expr::FunctionLiteral {
                params,
                result_types,
                body,
            } => self.compile_function_literal(dst, params, result_types, body),
            Expr::Unary { op, expr } => match op {
                UnaryOp::AddressOf => match expr.as_ref() {
                    Expr::Selector { receiver, field } => {
                        self.compile_address_of_selector(dst, receiver, field)
                    }
                    _ => self.compile_address_of_expr(dst, expr),
                },
                UnaryOp::Deref => self.compile_deref_expr(dst, expr),
                UnaryOp::Receive => self.compile_receive_expr(dst, expr),
                UnaryOp::Not => {
                    let src = self.compile_value_expr(expr)?;
                    self.emitter.code.push(Instruction::Not { dst, src });
                    Ok(())
                }
                UnaryOp::Negate => {
                    let src = self.compile_value_expr(expr)?;
                    self.emitter.code.push(Instruction::Negate { dst, src });
                    Ok(())
                }
                UnaryOp::BitNot => {
                    let src = self.compile_value_expr(expr)?;
                    self.emitter.code.push(Instruction::BitNot { dst, src });
                    Ok(())
                }
            },
            Expr::Ident(name) => {
                if let Some(src) = self.lookup_local(name) {
                    if self.scopes.captured_by_ref.contains(name) {
                        self.emitter.code.push(Instruction::Deref { dst, src });
                    } else {
                        self.emitter.code.push(Instruction::Move { dst, src });
                    }
                    return Ok(());
                }
                if let Some(global) = self.lookup_global(name) {
                    self.emitter
                        .code
                        .push(Instruction::LoadGlobal { dst, global });
                    return Ok(());
                }
                if let Some(function) = self.env.function_ids.get(name).copied() {
                    self.emitter.code.push(Instruction::MakeClosure {
                        dst,
                        concrete_type: self
                            .env
                            .function_types
                            .get(name)
                            .map(|typ| self.lower_runtime_concrete_type(typ))
                            .transpose()?,
                        function,
                        captures: Vec::new(),
                    });
                    return Ok(());
                }
                Err(CompileError::UnknownIdentifier { name: name.clone() })
            }
            Expr::Call {
                callee,
                type_args,
                args,
            } => self.compile_call(callee, type_args, args, Some(dst)),
            Expr::Binary { left, op, right } => match op {
                BinaryOp::Add => {
                    let left = self.compile_value_expr(left)?;
                    let right = self.compile_value_expr(right)?;
                    self.emitter
                        .code
                        .push(Instruction::Add { dst, left, right });
                    Ok(())
                }
                BinaryOp::Subtract
                | BinaryOp::BitOr
                | BinaryOp::BitXor
                | BinaryOp::BitAnd
                | BinaryOp::BitClear
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftRight => self.compile_int_binary(dst, *op, left, right),
                BinaryOp::And => self.compile_logical_and(dst, left, right),
                BinaryOp::Or => self.compile_logical_or(dst, left, right),
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => {
                    self.validate_comparison_operands(*op, left, right)?;
                    if self.try_compile_interface_nil_compare(dst, *op, left, right)? {
                        return Ok(());
                    }
                    let left = self.compile_value_expr(left)?;
                    let right = self.compile_value_expr(right)?;
                    self.emitter.code.push(Instruction::Compare {
                        dst,
                        op: lower_compare_op(*op),
                        left,
                        right,
                    });
                    Ok(())
                }
            },
            Expr::Spread { .. } => Err(CompileError::Unsupported {
                detail: "`...` spread operator can only be used in function call arguments".into(),
            }),
        }
    }

    fn compile_block(&mut self, body: &[Stmt]) -> Result<(), CompileError> {
        self.begin_scope();
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        self.end_scope();
        Ok(())
    }

    fn alloc_register(&mut self) -> usize {
        self.emitter.alloc_register()
    }

    fn bind_local(&mut self, name: &str, typ: Option<&str>) -> Result<Option<usize>, CompileError> {
        if name == "_" {
            return Ok(None);
        }
        if self.current_scope().contains_key(name) {
            return Err(CompileError::DuplicateLocal {
                name: name.to_string(),
            });
        }
        let register = if self.scopes.captured_by_ref.contains(name) {
            let value = self.alloc_register();
            if let Some(typ) = typ {
                self.compile_zero_value(value, typ)?;
                self.current_type_scope_mut()
                    .insert(name.to_string(), typ.to_string());
            } else {
                self.emitter.code.push(Instruction::LoadNil { dst: value });
            }
            let pointer = self.alloc_register();
            let pointer_type = typ
                .map(|typ| self.pointer_runtime_type(typ))
                .unwrap_or(TYPE_POINTER);
            self.emitter.code.push(Instruction::BoxHeap {
                dst: pointer,
                src: value,
                typ: pointer_type,
            });
            pointer
        } else {
            let register = self.alloc_register();
            if let Some(typ) = typ {
                self.current_type_scope_mut()
                    .insert(name.to_string(), typ.to_string());
            }
            register
        };
        self.current_scope_mut().insert(name.to_string(), register);
        Ok(Some(register))
    }

    fn begin_scope(&mut self) {
        self.scopes.begin_scope();
    }

    fn end_scope(&mut self) {
        self.scopes.end_scope();
    }
}

#[cfg(test)]
mod tests;

include!("test_modules.rs");

#[cfg(test)]
mod cap_tests {
    use cap::Cap;
    use std::alloc;

    const TEST_ALLOCATION_LIMIT_BYTES: usize = 64 * 1024 * 1024;

    #[global_allocator]
    static TEST_ALLOCATOR: Cap<alloc::System> =
        Cap::new(alloc::System, TEST_ALLOCATION_LIMIT_BYTES);

    #[test]
    fn small_allocations_fit_under_test_cap() {
        let bytes: Vec<u8> = std::iter::repeat_n(0_u8, 16 * 1024).collect();
        assert_eq!(bytes.len(), 16 * 1024);
    }
}
