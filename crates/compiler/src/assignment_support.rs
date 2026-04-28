use super::*;
use crate::diagnostics::CallableContext;
use crate::interface_satisfaction::{
    first_interface_method_mismatch, first_method_set_mismatch, method_signatures_match,
};
use crate::stdlib_function_values::{
    is_registered_stdlib_package, unsupported_stdlib_selector_detail,
};
use crate::types::{channel_types_assignable, display_type};
use std::collections::HashSet;

impl FunctionBuilder<'_> {
    pub(super) fn box_captured_parameter(&mut self, name: &str, typ: &str) {
        if !self.scopes.captured_by_ref.contains(name) {
            return;
        }
        let Some(src) = self.lookup_local(name) else {
            return;
        };
        let pointer = self.alloc_register();
        self.emitter.code.push(Instruction::BoxHeap {
            dst: pointer,
            src,
            typ: self.pointer_runtime_type(typ),
        });
        for scope in self.scopes.iter_mut().rev() {
            if let Some(slot) = scope.get_mut(name) {
                *slot = pointer;
                break;
            }
        }
    }

    pub(super) fn box_captured_parameters(&mut self, params: &[gowasm_parser::Parameter]) {
        for parameter in params {
            self.box_captured_parameter(&parameter.name, &parameter.typ);
        }
    }

    pub(super) fn bind_initialized_local(
        &mut self,
        name: &str,
        register: usize,
        typ: Option<String>,
    ) {
        if name == "_" {
            return;
        }
        let binding = if self.scopes.captured_by_ref.contains(name) {
            let pointer = self.alloc_register();
            self.emitter.code.push(Instruction::BoxHeap {
                dst: pointer,
                src: register,
                typ: typ
                    .as_deref()
                    .map(|typ| self.pointer_runtime_type(typ))
                    .unwrap_or(TYPE_POINTER),
            });
            pointer
        } else {
            register
        };
        self.current_scope_mut().insert(name.to_string(), binding);
        if let Some(typ) = typ {
            self.current_type_scope_mut().insert(name.to_string(), typ);
        }
    }

    pub(super) fn store_local_binding(
        &mut self,
        name: &str,
        src: usize,
    ) -> Result<(), CompileError> {
        if name == "_" {
            return Ok(());
        }
        let register =
            self.lookup_local(name)
                .ok_or_else(|| CompileError::UnknownAssignmentTarget {
                    name: name.to_string(),
                })?;
        if self.scopes.captured_by_ref.contains(name) {
            self.emitter.code.push(Instruction::StoreIndirect {
                target: register,
                src,
            });
        } else {
            self.emitter
                .code
                .push(Instruction::Move { dst: register, src });
        }
        Ok(())
    }

    pub(super) fn compile_increment(&mut self, name: &str) -> Result<(), CompileError> {
        self.compile_adjust(name, true)
    }

    pub(super) fn compile_decrement(&mut self, name: &str) -> Result<(), CompileError> {
        self.compile_adjust(name, false)
    }

    fn compile_adjust(&mut self, name: &str, increment: bool) -> Result<(), CompileError> {
        if self.binding_is_const(name) {
            return Err(CompileError::Unsupported {
                detail: format!("cannot modify const `{name}` in the current subset"),
            });
        }
        if self.scopes.captured_by_ref.contains(name) {
            let pointer = self
                .lookup_local(name)
                .expect("captured local should be in scope");
            let register = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: register,
                src: pointer,
            });
            let one = self.alloc_register();
            self.emitter
                .code
                .push(Instruction::LoadInt { dst: one, value: 1 });
            if increment {
                self.emitter.code.push(Instruction::Add {
                    dst: register,
                    left: register,
                    right: one,
                });
            } else {
                self.emitter.code.push(Instruction::Subtract {
                    dst: register,
                    left: register,
                    right: one,
                });
            }
            self.emitter.code.push(Instruction::StoreIndirect {
                target: pointer,
                src: register,
            });
            return Ok(());
        }
        let (register, global) = if let Some(register) = self.lookup_local(name) {
            (register, None)
        } else if let Some(global) = self.lookup_global(name) {
            let register = self.alloc_register();
            self.emitter.code.push(Instruction::LoadGlobal {
                dst: register,
                global,
            });
            (register, Some(global))
        } else {
            return Err(CompileError::UnknownAssignmentTarget {
                name: name.to_string(),
            });
        };
        let one = self.alloc_register();
        self.emitter
            .code
            .push(Instruction::LoadInt { dst: one, value: 1 });
        if increment {
            self.emitter.code.push(Instruction::Add {
                dst: register,
                left: register,
                right: one,
            });
        } else {
            self.emitter.code.push(Instruction::Subtract {
                dst: register,
                left: register,
                right: one,
            });
        }
        if let Some(global) = global {
            self.emitter.code.push(Instruction::StoreGlobal {
                global,
                src: register,
            });
        }
        Ok(())
    }

    pub(super) fn binding_is_const(&self, name: &str) -> bool {
        for (scope, consts) in self
            .scopes
            .iter()
            .rev()
            .zip(self.scopes.const_scopes.iter().rev())
        {
            if scope.contains_key(name) {
                return consts.contains(name);
            }
        }
        self.env
            .globals
            .get(name)
            .map(|binding| binding.is_const)
            .unwrap_or(false)
    }

    pub(super) fn infer_expr_result_types(&self, expr: &Expr) -> Option<Vec<String>> {
        match expr {
            Expr::Call {
                callee,
                type_args,
                args,
            } => {
                if let Some(generic_call) = self.resolve_generic_call(callee, type_args, args) {
                    return generic_call
                        .ok()
                        .map(|generic_call| generic_call.result_types);
                }
                match callee.as_ref() {
                    Expr::Ident(name) => self
                        .env
                        .function_result_types
                        .get(name)
                        .cloned()
                        .or_else(|| self.expr_function_result_types(callee)),
                    Expr::Selector { receiver, field } => {
                        if let Expr::Ident(receiver_name) = receiver.as_ref() {
                            if let Some(package_path) =
                                self.env.imported_packages.get(receiver_name)
                            {
                                if let Some(function) = resolve_stdlib_function(package_path, field)
                                {
                                    return stdlib_function_result_types(function).map(|types| {
                                        types.iter().map(|typ| (*typ).to_string()).collect()
                                    });
                                }
                                return self
                                    .lookup_imported_function_result_types(package_path, field)
                                    .cloned();
                            }
                        }
                        let receiver_type = self.infer_expr_type_name(receiver)?;
                        if let Some(interface_type) =
                            self.instantiated_interface_type(&receiver_type)
                        {
                            interface_type
                                .methods
                                .iter()
                                .find(|method| method.name == *field)
                                .map(|method| method.result_types.clone())
                        } else {
                            self.lookup_concrete_method(receiver, field)
                                .map(|method| method.result_types.clone())
                                .or_else(|| self.expr_function_result_types(callee))
                        }
                    }
                    _ => self.expr_function_result_types(callee),
                }
            }
            _ => self.infer_expr_type_name(expr).map(|typ| vec![typ]),
        }
    }

    pub(super) fn validate_short_decl_names(
        &self,
        names: &[&str],
    ) -> Result<Vec<bool>, CompileError> {
        let mut seen = HashSet::new();
        let mut has_new_name = false;
        let mut bindings = Vec::with_capacity(names.len());
        for name in names {
            if *name == "_" {
                bindings.push(false);
                continue;
            }
            if !seen.insert(*name) {
                return Err(CompileError::DuplicateLocal {
                    name: (*name).to_string(),
                });
            }
            let is_new = !self.current_scope().contains_key(*name);
            if is_new {
                has_new_name = true;
            }
            bindings.push(is_new);
        }
        if !has_new_name {
            return Err(CompileError::Unsupported {
                detail: "no new variables on the left side of `:=` in the current scope".into(),
            });
        }
        Ok(bindings)
    }

    pub(super) fn assignment_target_type(&self, target: &AssignTarget) -> Option<String> {
        self.infer_expr_type_name(&assignment_target_expr(target))
    }

    pub(super) fn compile_assignment_expr_list(
        &mut self,
        values: &[Expr],
        dsts: &[usize],
        target_types: &[Option<String>],
    ) -> Result<Vec<Option<String>>, CompileError> {
        if values.len() > dsts.len() {
            return Err(self.assignment_count_mismatch(values.len(), dsts.len()));
        }

        let mut result_types = Vec::with_capacity(dsts.len());
        let prefix_len = values.len().saturating_sub(1);
        for index in 0..prefix_len {
            let value = &values[index];
            if self.multi_result_call_arity(value).is_some() {
                return Err(CompileError::Unsupported {
                    detail: "multi-result call expressions must appear in the final assignment position in the current subset"
                        .into(),
                });
            }
            result_types.push(self.compile_single_assignment_expr(
                &target_types[index],
                value,
                dsts[index],
            )?);
        }

        let final_slots = dsts.len() - prefix_len;
        let final_value = values
            .last()
            .expect("assignment expression list should not be empty");
        if final_slots == 1 {
            if let Some(arity) = self.multi_result_call_arity(final_value) {
                return Err(self.assignment_count_mismatch(prefix_len + arity, dsts.len()));
            }
            result_types.push(self.compile_single_assignment_expr(
                &target_types[prefix_len],
                final_value,
                dsts[prefix_len],
            )?);
            return Ok(result_types);
        }

        let Some(arity) = self.assignment_tail_expansion_arity(final_value) else {
            return Err(self.assignment_count_mismatch(prefix_len + 1, dsts.len()));
        };
        if arity != final_slots {
            return Err(self.assignment_count_mismatch(prefix_len + arity, dsts.len()));
        }

        let expanded_result_types = self.inferred_result_types(final_value, final_slots);
        self.validate_multi_result_target_types(
            &target_types[prefix_len..],
            &expanded_result_types,
        )?;
        self.compile_multi_value_expr(final_value, &dsts[prefix_len..])?;
        result_types.extend(expanded_result_types);
        Ok(result_types)
    }

    pub(super) fn validate_assignable_type(
        &self,
        target_type: Option<&str>,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let Some(target_type) = target_type else {
            return Ok(());
        };
        if matches!(value, Expr::NilLiteral) {
            if self.type_allows_nil(target_type) {
                return Ok(());
            }
            return Err(self.unsupported_with_active_span(format!(
                "cannot use `nil` as `{target_type}` in the current subset"
            )));
        }
        if let Some(info) = self.try_eval_const_expr(value) {
            return self
                .validate_const_assignable_type(target_type, &info)
                .map_err(|error| self.annotate_compile_error(error));
        }
        if self.literal_assignable_to(target_type, value) {
            return Ok(());
        }
        let target_uses_function_shape = self.type_uses_function_shape(target_type);
        let Some(source_type) = self.infer_expr_type_name(value) else {
            if target_uses_function_shape {
                if let Expr::Selector { receiver, field } = value {
                    if let Expr::Ident(receiver_name) = receiver.as_ref() {
                        if let Some(package_path) = self.env.imported_packages.get(receiver_name) {
                            if let Some(function) = resolve_stdlib_function(package_path, field) {
                                if stdlib_function_variadic_param_type(function).is_some() {
                                    return Err(self.unsupported_with_active_span(format!(
                                        "package selector `{receiver_name}.{field}` cannot be used in value position"
                                    )));
                                }
                            }
                        }
                    }
                }
                return Err(self.unsupported_with_active_span(format!(
                    "cannot infer the function type required for `{}` in the current subset",
                    display_type(target_type)
                )));
            }
            return Ok(());
        };

        self.validate_assignable_source_type(target_type, &source_type)
            .map_err(|error| self.annotate_compile_error(error))
    }

    pub(super) fn validate_multi_result_target_types(
        &mut self,
        target_types: &[Option<String>],
        result_types: &[Option<String>],
    ) -> Result<(), CompileError> {
        for (index, (target_type, result_type)) in
            target_types.iter().zip(result_types.iter()).enumerate()
        {
            let Some(target_type) = target_type.as_deref() else {
                continue;
            };
            self.ensure_runtime_visible_type(target_type)?;
            let Some(result_type) = result_type.as_deref() else {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "cannot infer result {} type for multi-result assignment in the current subset",
                        index + 1
                    ),
                });
            };
            self.validate_assignable_source_type(target_type, result_type)?;
        }
        Ok(())
    }

    pub(super) fn validate_assignable_source_type(
        &self,
        target_type: &str,
        source_type: &str,
    ) -> Result<(), CompileError> {
        let target_uses_function_shape = self.type_uses_function_shape(target_type);
        if self.types_assignable(target_type, source_type) {
            return Ok(());
        }

        if target_uses_function_shape || self.type_uses_function_shape(source_type) {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "function value of type `{}` is not assignable to `{}` in the current subset",
                    display_type(source_type),
                    display_type(target_type)
                ),
            });
        }

        if self.type_uses_channel_shape(target_type) || self.type_uses_channel_shape(source_type) {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "channel value of type `{source_type}` is not assignable to `{target_type}` in the current subset"
                ),
            });
        }

        if let Some(interface_type) = self.instantiated_interface_type(target_type) {
            let detail = self
                .interface_satisfaction_mismatch_detail(target_type, source_type, &interface_type)
                .map(|detail| format!(": {detail}"))
                .unwrap_or_default();
            return Err(CompileError::Unsupported {
                detail: format!(
                    "type `{source_type}` does not satisfy interface `{target_type}` in the current subset{detail}"
                ),
            });
        }

        Err(CompileError::Unsupported {
            detail: format!(
                "value of type `{source_type}` is not assignable to `{target_type}` in the current subset"
            ),
        })
    }

    pub(super) fn type_satisfies_interface(
        &self,
        target_type: &str,
        source_type: &str,
        interface_type: &InterfaceTypeDef,
    ) -> bool {
        if interface_type.methods.is_empty() || source_type == target_type {
            return true;
        }
        if let Some(source_interface) = self.instantiated_interface_type(source_type) {
            return interface_type.methods.iter().all(|required| {
                source_interface
                    .methods
                    .iter()
                    .any(|candidate| method_signatures_match(candidate, required))
            });
        }
        interface_type.methods.iter().all(|required| {
            self.method_sets_satisfy(source_type, required)
                || parse_pointer_type(source_type)
                    .is_some_and(|inner| self.method_sets_satisfy(inner, required))
        })
    }

    fn interface_satisfaction_mismatch_detail(
        &self,
        target_type: &str,
        source_type: &str,
        interface_type: &InterfaceTypeDef,
    ) -> Option<String> {
        if interface_type.methods.is_empty() || source_type == target_type {
            return None;
        }
        if let Some(source_interface) = self.instantiated_interface_type(source_type) {
            return first_interface_method_mismatch(
                &source_interface.methods,
                &interface_type.methods,
            );
        }

        let mut candidate_sets = Vec::new();
        if let Some(methods) = self.instantiated_method_set(source_type) {
            candidate_sets.push(methods);
        }
        if let Some(inner) = parse_pointer_type(source_type) {
            if let Some(methods) = self.instantiated_method_set(inner) {
                candidate_sets.push(methods);
            }
        }
        first_method_set_mismatch(&candidate_sets, &interface_type.methods)
    }

    fn method_sets_satisfy(&self, source_type: &str, required: &InterfaceMethodDecl) -> bool {
        self.instantiated_method_set(source_type)
            .is_some_and(|methods| {
                methods
                    .iter()
                    .any(|candidate| method_signatures_match(candidate, required))
            })
    }

    pub(super) fn type_allows_nil(&self, typ: &str) -> bool {
        if typ == "interface{}" || typ == "any" || self.instantiated_interface_type(typ).is_some() {
            return true;
        }

        let underlying = self.instantiated_underlying_type_name(typ);
        parse_map_type(&underlying).is_some()
            || underlying.starts_with("[]")
            || parse_pointer_type(&underlying).is_some()
            || parse_channel_type(&underlying).is_some()
            || parse_function_type(&underlying).is_some()
    }

    pub(super) fn type_uses_function_shape(&self, typ: &str) -> bool {
        parse_function_type(typ).is_some()
            || parse_function_type(&self.instantiated_underlying_type_name(typ)).is_some()
    }

    pub(super) fn type_uses_channel_shape(&self, typ: &str) -> bool {
        parse_channel_type(typ).is_some()
            || parse_channel_type(&self.instantiated_underlying_type_name(typ)).is_some()
    }

    pub(super) fn types_assignable(&self, target_type: &str, source_type: &str) -> bool {
        if target_type == source_type || target_type == "interface{}" || target_type == "any" {
            return true;
        }

        if let Some(interface_type) = self.instantiated_interface_type(target_type) {
            return self.type_satisfies_interface(target_type, source_type, &interface_type);
        }

        let target_underlying = self.instantiated_underlying_type_name(target_type);
        let source_underlying = self.instantiated_underlying_type_name(source_type);
        let target_named = self.instantiated_named_type(target_type);
        let source_named = self.instantiated_named_type(source_type);

        if parse_channel_type(&target_underlying).is_some()
            && parse_channel_type(&source_underlying).is_some()
        {
            return (!target_named || !source_named)
                && channel_types_assignable(&target_underlying, &source_underlying);
        }

        if function_signatures_match(&target_underlying, &source_underlying) {
            return !target_named || !source_named;
        }

        target_underlying == source_underlying && (!target_named || !source_named)
    }

    pub(super) fn literal_assignable_to(&self, target_type: &str, value: &Expr) -> bool {
        let underlying = self.instantiated_underlying_type_name(target_type);
        matches!(
            (value, underlying.as_str()),
            (Expr::IntLiteral(_), "int" | "byte" | "rune")
                | (Expr::FloatLiteral(_), "float64")
                | (Expr::BoolLiteral(_), "bool")
                | (Expr::StringLiteral(_), "string")
        )
    }

    pub(super) fn lookup_pointer_target(&mut self, name: &str) -> Result<usize, CompileError> {
        if self.scopes.captured_by_ref.contains(name) {
            return self.compile_value_expr(&Expr::Ident(name.to_string()));
        }
        if let Some(register) = self.lookup_local(name) {
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
        Err(CompileError::UnknownAssignmentTarget {
            name: name.to_string(),
        })
    }

    pub(super) fn inferred_result_types(&self, value: &Expr, arity: usize) -> Vec<Option<String>> {
        if let Expr::Index { target, .. } = value {
            if arity == 2 {
                if let Some(map_type) = self.infer_expr_type_name(target) {
                    if let Some((_key_type, value_type)) = parse_map_type(&map_type) {
                        return vec![Some(value_type.to_string()), Some("bool".into())];
                    }
                }
            }
        }
        if let Expr::TypeAssert { asserted_type, .. } = value {
            if arity == 2 {
                return vec![Some(asserted_type.clone()), Some("bool".into())];
            }
        }
        if let Expr::Unary {
            op: UnaryOp::Receive,
            expr,
        } = value
        {
            if arity == 2 {
                if let Some(channel_type) = self.infer_expr_type_name(expr) {
                    if let Some(channel_type) = parse_channel_type(&channel_type) {
                        if channel_type.direction.accepts_recv() {
                            return vec![
                                Some(channel_type.element_type.to_string()),
                                Some("bool".into()),
                            ];
                        }
                    }
                }
            }
        }
        let mut result = self
            .infer_expr_result_types(value)
            .unwrap_or_default()
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();
        result.resize(arity, None);
        result
    }

    fn compile_single_assignment_expr(
        &mut self,
        target_type: &Option<String>,
        value: &Expr,
        dst: usize,
    ) -> Result<Option<String>, CompileError> {
        if let Some(target_type) = target_type.as_deref() {
            self.ensure_runtime_visible_type(target_type)?;
        }
        self.ensure_expr_runtime_types(value)?;
        self.validate_assignable_type(target_type.as_deref(), value)?;
        self.compile_expr_into_with_hint(dst, value, target_type.as_deref())?;
        Ok(target_type
            .clone()
            .or_else(|| self.infer_expr_type_name(value)))
    }

    pub(super) fn assignment_tail_expansion_arity(&self, value: &Expr) -> Option<usize> {
        match value {
            Expr::Index { .. }
            | Expr::TypeAssert { .. }
            | Expr::Unary {
                op: UnaryOp::Receive,
                ..
            } => Some(2),
            Expr::Call { .. } => self.multi_result_call_arity(value),
            _ => None,
        }
    }

    fn multi_result_call_arity(&self, value: &Expr) -> Option<usize> {
        match value {
            Expr::Call { .. } => self
                .infer_expr_result_types(value)
                .map(|result_types| result_types.len())
                .filter(|arity| *arity > 1),
            _ => None,
        }
    }

    fn compile_multi_value_expr(
        &mut self,
        value: &Expr,
        dsts: &[usize],
    ) -> Result<(), CompileError> {
        match dsts {
            [first, second] => self.compile_pair_value(value, *first, *second),
            [first, second, third] => self.compile_triple_value(value, *first, *second, *third),
            [first, second, third, fourth] => {
                self.compile_quad_value(value, *first, *second, *third, *fourth)
            }
            _ => Err(CompileError::Unsupported {
                detail: format!(
                    "assignment lists currently support one to four targets; found {}",
                    dsts.len()
                ),
            }),
        }
    }

    pub(super) fn assignment_count_mismatch(
        &self,
        value_count: usize,
        target_count: usize,
    ) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "assignment value count {value_count} does not match {target_count} target(s) in the current subset"
            ),
        }
    }

    pub(super) fn missing_multi_result_type_error(&self, index: usize) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "cannot infer result {} type for multi-result assignment in the current subset",
                index + 1
            ),
        }
    }

    pub(super) fn assignment_tail_diagnostic_or_count_mismatch(
        &self,
        value: &Expr,
        target_count: usize,
    ) -> CompileError {
        let Expr::Call { callee, .. } = value else {
            return self.assignment_count_mismatch(1, target_count);
        };

        match callee.as_ref() {
            Expr::Ident(name) if self.lookup_local(name).is_some() => self
                .local_non_callable_call_error(
                    name,
                    callee,
                    CallableContext::MultiCall {
                        expected_results: target_count,
                    },
                ),
            Expr::Ident(name) if self.lookup_global(name).is_some() => self
                .global_non_callable_call_error(
                    name,
                    callee,
                    CallableContext::MultiCall {
                        expected_results: target_count,
                    },
                ),
            Expr::Ident(name) => {
                if let Some(result_types) = self.env.function_result_types.get(name) {
                    return CompileError::Unsupported {
                        detail: format!(
                            "`{name}` returns {} value(s), not {target_count}",
                            result_types.len()
                        ),
                    };
                }
                self.unsupported_call_target_error(
                    callee,
                    CallableContext::MultiCall {
                        expected_results: target_count,
                    },
                )
            }
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(package_path) = self.env.imported_packages.get(receiver_name) {
                        if let Some(function) = resolve_stdlib_function(package_path, field) {
                            return CompileError::Unsupported {
                                detail: format!(
                                    "`{receiver_name}.{field}` returns {} value(s), not {target_count}",
                                    stdlib_function_result_count(function)
                                ),
                            };
                        }
                        if let Some(result_types) =
                            self.lookup_imported_function_result_types(package_path, field)
                        {
                            return CompileError::Unsupported {
                                detail: format!(
                                    "`{receiver_name}.{field}` returns {} value(s), not {target_count}",
                                    result_types.len()
                                ),
                            };
                        }
                        let unsupported_stdlib_selector =
                            resolve_stdlib_function(package_path, field).is_none()
                                && self
                                    .lookup_imported_function_id(package_path, field)
                                    .is_none()
                                && is_registered_stdlib_package(package_path);
                        if unsupported_stdlib_selector {
                            return CompileError::Unsupported {
                                detail: unsupported_stdlib_selector_detail(receiver_name, field),
                            };
                        }
                    }
                }
                self.unsupported_call_target_error(
                    callee,
                    CallableContext::MultiCall {
                        expected_results: target_count,
                    },
                )
            }
            _ => self.unsupported_call_target_error(
                callee,
                CallableContext::MultiCall {
                    expected_results: target_count,
                },
            ),
        }
    }
}

fn assignment_target_expr(target: &AssignTarget) -> Expr {
    match target {
        AssignTarget::Ident(name) => Expr::Ident(name.clone()),
        AssignTarget::Deref { target } => Expr::Unary {
            op: UnaryOp::Deref,
            expr: Box::new(Expr::Ident(target.clone())),
        },
        AssignTarget::DerefSelector { target, field } => Expr::Selector {
            receiver: Box::new(Expr::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(Expr::Ident(target.clone())),
            }),
            field: field.clone(),
        },
        AssignTarget::DerefIndex { target, index } => Expr::Index {
            target: Box::new(Expr::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(Expr::Ident(target.clone())),
            }),
            index: Box::new(index.clone()),
        },
        AssignTarget::Selector { receiver, field } => Expr::Selector {
            receiver: Box::new(Expr::Ident(receiver.clone())),
            field: field.clone(),
        },
        AssignTarget::Index { target, index } => Expr::Index {
            target: Box::new(Expr::Ident(target.clone())),
            index: Box::new(index.clone()),
        },
    }
}
