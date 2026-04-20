use super::*;
use crate::consts::rewrite_const_iota_expr;

impl FunctionBuilder<'_> {
    fn emit_selector_path_set(&mut self, target: usize, path: &[String], src: usize) {
        if path.len() == 1 {
            self.emitter.code.push(Instruction::SetField {
                target,
                field: path[0].clone(),
                src,
            });
            return;
        }

        let mut containers = vec![target];
        let mut current = target;
        for field in &path[..path.len() - 1] {
            let next = self.alloc_register();
            self.emitter.code.push(Instruction::GetField {
                dst: next,
                target: current,
                field: field.clone(),
            });
            containers.push(next);
            current = next;
        }

        self.emitter.code.push(Instruction::SetField {
            target: current,
            field: path[path.len() - 1].clone(),
            src,
        });

        for index in (1..containers.len()).rev() {
            self.emitter.code.push(Instruction::SetField {
                target: containers[index - 1],
                field: path[index - 1].clone(),
                src: containers[index],
            });
        }
    }

    pub(super) fn compile_short_var_decl(
        &mut self,
        name: &str,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let is_new = self.validate_short_decl_names(std::slice::from_ref(&name))?;
        let inferred_type = self.infer_expr_type_name(value);

        let register = self.alloc_register();
        self.compile_expr_into_with_hint(register, value, inferred_type.as_deref())?;
        if is_new[0] {
            self.bind_initialized_local(name, register, inferred_type);
        } else {
            self.store_local_binding(name, register)?;
        }
        Ok(())
    }

    pub(super) fn compile_short_var_decl_list(
        &mut self,
        names: &[String],
        values: &[Expr],
    ) -> Result<(), CompileError> {
        let bindings = names.iter().map(String::as_str).collect::<Vec<_>>();
        let is_new = self.validate_short_decl_names(&bindings)?;
        let registers = names
            .iter()
            .map(|_| self.alloc_register())
            .collect::<Vec<_>>();
        let target_types = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                if is_new[index] || name == "_" {
                    None
                } else {
                    self.lookup_local_type(name)
                        .or_else(|| self.lookup_global_type(name))
                        .map(str::to_string)
                }
            })
            .collect::<Vec<_>>();
        let result_types = self.compile_assignment_expr_list(values, &registers, &target_types)?;
        for (index, name) in names.iter().enumerate() {
            if is_new[index] {
                self.bind_initialized_local(name, registers[index], result_types[index].clone());
            } else if name != "_" {
                self.store_local_binding(name, registers[index])?;
            }
        }
        Ok(())
    }

    pub(super) fn compile_var_decl(
        &mut self,
        name: &str,
        typ: Option<&str>,
        value: Option<&Expr>,
    ) -> Result<(), CompileError> {
        if self.current_scope().contains_key(name) {
            return Err(CompileError::DuplicateLocal {
                name: name.to_string(),
            });
        }

        let register = self.alloc_register();
        match (typ, value) {
            (_, Some(value)) => {
                let inferred_type = typ
                    .map(str::to_string)
                    .or_else(|| self.infer_expr_type_name(value));
                if let Some(typ) = typ {
                    self.ensure_runtime_visible_type(typ)
                        .map_err(|error| self.annotate_compile_error(error))?;
                }
                self.ensure_expr_runtime_types(value)
                    .map_err(|error| self.annotate_compile_error(error))?;
                self.validate_assignable_type(typ, value)
                    .map_err(|error| self.annotate_compile_error(error))?;
                self.compile_expr_into_with_hint(register, value, inferred_type.as_deref())?
            }
            (Some(typ), None) => {
                self.ensure_runtime_visible_type(typ)
                    .map_err(|error| self.annotate_compile_error(error))?;
                self.compile_zero_value(register, typ)?
            }
            (None, None) => {
                return Err(self.unsupported_with_active_span(format!(
                    "`var {name}` must include a type or initializer"
                )));
            }
        }
        self.bind_initialized_local(
            name,
            register,
            typ.map(str::to_string)
                .or_else(|| self.infer_expr_type_name(value.expect("value exists"))),
        );
        Ok(())
    }

    pub(super) fn compile_const_decl(
        &mut self,
        name: &str,
        typ: Option<&str>,
        value: &Expr,
        iota: usize,
    ) -> Result<(), CompileError> {
        if self.current_scope().contains_key(name) {
            return Err(CompileError::DuplicateLocal {
                name: name.to_string(),
            });
        }

        let value = rewrite_const_iota_expr(value, iota);
        self.validate_const_initializer(typ, &value)
            .map_err(|error| self.annotate_compile_error(error))?;
        let const_value: crate::const_eval::ConstValueInfo = self
            .eval_const_expr(&value)
            .and_then(|value| match typ {
                Some(typ) => self.coerce_const_value_info(&value, typ),
                None => Ok(value),
            })
            .map_err(|error| self.annotate_compile_error(error))?;
        let register = self.alloc_register();
        self.compile_expr_into_with_hint(register, &value, typ)?;
        self.current_scope_mut().insert(name.to_string(), register);
        self.scopes
            .const_scopes
            .last_mut()
            .expect("const scope should exist")
            .insert(name.to_string());
        self.scopes
            .current_const_value_scope_mut()
            .insert(name.to_string(), const_value.clone());
        self.current_type_scope_mut()
            .insert(name.to_string(), const_value.visible_type_name());
        Ok(())
    }

    pub(super) fn compile_short_var_decl_pair(
        &mut self,
        first: &str,
        second: &str,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let is_new = self.validate_short_decl_names(&[first, second])?;
        let target_types = vec![
            short_decl_target_type(self, first, is_new[0]),
            short_decl_target_type(self, second, is_new[1]),
        ];
        let result_types = self.fixed_multi_value_result_types(value, &target_types)?;

        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        self.compile_pair_value(value, first_register, second_register)?;
        if is_new[0] {
            self.bind_initialized_local(first, first_register, result_types[0].clone());
        } else if first != "_" {
            self.store_local_binding(first, first_register)?;
        }

        if is_new[1] {
            self.bind_initialized_local(second, second_register, result_types[1].clone());
        } else if second != "_" {
            self.store_local_binding(second, second_register)?;
        }
        Ok(())
    }

    pub(super) fn compile_short_var_decl_triple(
        &mut self,
        first: &str,
        second: &str,
        third: &str,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let is_new = self.validate_short_decl_names(&[first, second, third])?;
        let target_types = vec![
            short_decl_target_type(self, first, is_new[0]),
            short_decl_target_type(self, second, is_new[1]),
            short_decl_target_type(self, third, is_new[2]),
        ];
        let result_types = self.fixed_multi_value_result_types(value, &target_types)?;

        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        let third_register = self.alloc_register();
        self.compile_triple_value(value, first_register, second_register, third_register)?;
        if is_new[0] {
            self.bind_initialized_local(first, first_register, result_types[0].clone());
        } else if first != "_" {
            self.store_local_binding(first, first_register)?;
        }

        if is_new[1] {
            self.bind_initialized_local(second, second_register, result_types[1].clone());
        } else if second != "_" {
            self.store_local_binding(second, second_register)?;
        }

        if is_new[2] {
            self.bind_initialized_local(third, third_register, result_types[2].clone());
        } else if third != "_" {
            self.store_local_binding(third, third_register)?;
        }
        Ok(())
    }

    pub(super) fn compile_short_var_decl_quad(
        &mut self,
        first: &str,
        second: &str,
        third: &str,
        fourth: &str,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let is_new = self.validate_short_decl_names(&[first, second, third, fourth])?;
        let target_types = vec![
            short_decl_target_type(self, first, is_new[0]),
            short_decl_target_type(self, second, is_new[1]),
            short_decl_target_type(self, third, is_new[2]),
            short_decl_target_type(self, fourth, is_new[3]),
        ];
        let result_types = self.fixed_multi_value_result_types(value, &target_types)?;

        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        let third_register = self.alloc_register();
        let fourth_register = self.alloc_register();
        self.compile_quad_value(
            value,
            first_register,
            second_register,
            third_register,
            fourth_register,
        )?;
        if is_new[0] {
            self.bind_initialized_local(first, first_register, result_types[0].clone());
        } else if first != "_" {
            self.store_local_binding(first, first_register)?;
        }

        if is_new[1] {
            self.bind_initialized_local(second, second_register, result_types[1].clone());
        } else if second != "_" {
            self.store_local_binding(second, second_register)?;
        }

        if is_new[2] {
            self.bind_initialized_local(third, third_register, result_types[2].clone());
        } else if third != "_" {
            self.store_local_binding(third, third_register)?;
        }

        if is_new[3] {
            self.bind_initialized_local(fourth, fourth_register, result_types[3].clone());
        } else if fourth != "_" {
            self.store_local_binding(fourth, fourth_register)?;
        }
        Ok(())
    }

    pub(super) fn compile_assign(
        &mut self,
        target: &AssignTarget,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let target_type = self.assignment_target_type(target);
        if let Some(target_type) = target_type.as_deref() {
            self.ensure_runtime_visible_type(target_type)?;
        }
        self.ensure_expr_runtime_types(value)?;
        self.validate_assignable_type(target_type.as_deref(), value)?;
        let src = self.alloc_register();
        self.compile_expr_into_with_hint(src, value, target_type.as_deref())?;
        self.compile_assign_from_register(target, src)
    }

    pub(super) fn compile_assign_list(
        &mut self,
        targets: &[AssignTarget],
        values: &[Expr],
    ) -> Result<(), CompileError> {
        let registers = targets
            .iter()
            .map(|_| self.alloc_register())
            .collect::<Vec<_>>();
        let target_types = targets
            .iter()
            .map(|target| self.assignment_target_type(target))
            .collect::<Vec<_>>();
        self.compile_assignment_expr_list(values, &registers, &target_types)?;
        for (target, register) in targets.iter().zip(registers.iter()) {
            self.compile_assign_from_register(target, *register)?;
        }
        Ok(())
    }

    pub(super) fn compile_assign_pair(
        &mut self,
        first: &AssignTarget,
        second: &AssignTarget,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let target_types = vec![
            self.assignment_target_type(first),
            self.assignment_target_type(second),
        ];
        self.fixed_multi_value_result_types(value, &target_types)?;
        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        self.compile_pair_value(value, first_register, second_register)?;
        self.compile_assign_from_register(first, first_register)?;
        self.compile_assign_from_register(second, second_register)?;
        Ok(())
    }

    pub(super) fn compile_assign_triple(
        &mut self,
        first: &AssignTarget,
        second: &AssignTarget,
        third: &AssignTarget,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let target_types = vec![
            self.assignment_target_type(first),
            self.assignment_target_type(second),
            self.assignment_target_type(third),
        ];
        self.fixed_multi_value_result_types(value, &target_types)?;
        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        let third_register = self.alloc_register();
        self.compile_triple_value(value, first_register, second_register, third_register)?;
        self.compile_assign_from_register(first, first_register)?;
        self.compile_assign_from_register(second, second_register)?;
        self.compile_assign_from_register(third, third_register)?;
        Ok(())
    }

    pub(super) fn compile_assign_quad(
        &mut self,
        first: &AssignTarget,
        second: &AssignTarget,
        third: &AssignTarget,
        fourth: &AssignTarget,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let target_types = vec![
            self.assignment_target_type(first),
            self.assignment_target_type(second),
            self.assignment_target_type(third),
            self.assignment_target_type(fourth),
        ];
        self.fixed_multi_value_result_types(value, &target_types)?;
        let first_register = self.alloc_register();
        let second_register = self.alloc_register();
        let third_register = self.alloc_register();
        let fourth_register = self.alloc_register();
        self.compile_quad_value(
            value,
            first_register,
            second_register,
            third_register,
            fourth_register,
        )?;
        self.compile_assign_from_register(first, first_register)?;
        self.compile_assign_from_register(second, second_register)?;
        self.compile_assign_from_register(third, third_register)?;
        self.compile_assign_from_register(fourth, fourth_register)?;
        Ok(())
    }

    fn fixed_multi_value_result_types(
        &mut self,
        value: &Expr,
        target_types: &[Option<String>],
    ) -> Result<Vec<Option<String>>, CompileError> {
        let expected = target_types.len();
        let Some(actual) = self.assignment_tail_expansion_arity(value) else {
            return match value {
                Expr::Call { .. } => {
                    Err(self.assignment_tail_diagnostic_or_count_mismatch(value, expected))
                }
                _ => Err(self.missing_multi_result_type_error(1)),
            };
        };
        if actual != expected {
            return Err(self.assignment_count_mismatch(actual, expected));
        }
        let result_types = self.inferred_result_types(value, expected);
        self.validate_multi_result_target_types(target_types, &result_types)?;
        Ok(result_types)
    }

    pub(super) fn compile_assign_from_register(
        &mut self,
        target: &AssignTarget,
        src: usize,
    ) -> Result<(), CompileError> {
        match target {
            AssignTarget::Ident(name) => {
                if name == "_" {
                    return Ok(());
                }
                if self.binding_is_const(name) {
                    return Err(CompileError::Unsupported {
                        detail: format!("cannot assign to const `{name}` in the current subset"),
                    });
                }
                if let Some(register) = self.lookup_local(name) {
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
                    return Ok(());
                }
                if let Some(global) = self.lookup_global(name) {
                    self.emitter
                        .code
                        .push(Instruction::StoreGlobal { global, src });
                    return Ok(());
                }
                Err(CompileError::UnknownAssignmentTarget {
                    name: name.to_string(),
                })
            }
            AssignTarget::Deref { target } => {
                let pointer = self.lookup_pointer_target(target)?;
                self.emitter.code.push(Instruction::StoreIndirect {
                    target: pointer,
                    src,
                });
                Ok(())
            }
            AssignTarget::DerefSelector { target, field } => {
                let pointer = self.lookup_pointer_target(target)?;
                let value = self.alloc_register();
                self.emitter.code.push(Instruction::Deref {
                    dst: value,
                    src: pointer,
                });
                self.emitter.code.push(Instruction::SetField {
                    target: value,
                    field: field.clone(),
                    src,
                });
                self.emitter.code.push(Instruction::StoreIndirect {
                    target: pointer,
                    src: value,
                });
                Ok(())
            }
            AssignTarget::DerefIndex { target, index } => {
                let pointer = self.lookup_pointer_target(target)?;
                let value = self.alloc_register();
                self.emitter.code.push(Instruction::Deref {
                    dst: value,
                    src: pointer,
                });
                let index = self.compile_value_expr(index)?;
                self.emitter.code.push(Instruction::SetIndex {
                    target: value,
                    index,
                    src,
                });
                self.emitter.code.push(Instruction::StoreIndirect {
                    target: pointer,
                    src: value,
                });
                Ok(())
            }
            AssignTarget::Selector { receiver, field } => {
                let receiver_type = self
                    .lookup_local_type(receiver)
                    .or_else(|| self.lookup_global_type(receiver))
                    .map(str::to_string)
                    .ok_or_else(|| CompileError::UnknownAssignmentTarget {
                        name: receiver.to_string(),
                    })?;
                let selector_path = self
                    .resolve_field_selector(&receiver_type, field)?
                    .map(|resolution| resolution.path)
                    .ok_or_else(|| CompileError::Unsupported {
                        detail: format!(
                            "unknown field selector `{receiver_type}.{field}` in the current subset"
                        ),
                    })?;
                if self.scopes.captured_by_ref.contains(receiver) {
                    let pointer = self
                        .lookup_local(receiver)
                        .expect("captured local should be in scope");
                    if self
                        .lookup_local_type(receiver)
                        .and_then(parse_pointer_type)
                        .filter(|inner| self.instantiated_struct_type(inner).is_some())
                        .is_some()
                    {
                        let target = self.compile_value_expr(&Expr::Ident(receiver.clone()))?;
                        let value = self.alloc_register();
                        self.emitter.code.push(Instruction::Deref {
                            dst: value,
                            src: target,
                        });
                        self.emit_selector_path_set(value, &selector_path, src);
                        self.emitter
                            .code
                            .push(Instruction::StoreIndirect { target, src: value });
                        return Ok(());
                    }
                    let target = self.compile_value_expr(&Expr::Ident(receiver.clone()))?;
                    self.emit_selector_path_set(target, &selector_path, src);
                    self.emitter.code.push(Instruction::StoreIndirect {
                        target: pointer,
                        src: target,
                    });
                    return Ok(());
                }
                if self
                    .lookup_local_type(receiver)
                    .or_else(|| self.lookup_global_type(receiver))
                    .and_then(parse_pointer_type)
                    .filter(|inner| self.instantiated_struct_type(inner).is_some())
                    .is_some()
                {
                    let pointer = self.lookup_pointer_target(receiver)?;
                    let value = self.alloc_register();
                    self.emitter.code.push(Instruction::Deref {
                        dst: value,
                        src: pointer,
                    });
                    self.emit_selector_path_set(value, &selector_path, src);
                    self.emitter.code.push(Instruction::StoreIndirect {
                        target: pointer,
                        src: value,
                    });
                    return Ok(());
                }
                let (target, global) = if let Some(target) = self.lookup_local(receiver) {
                    (target, None)
                } else if let Some(global) = self.lookup_global(receiver) {
                    let target = self.alloc_register();
                    self.emitter.code.push(Instruction::LoadGlobal {
                        dst: target,
                        global,
                    });
                    (target, Some(global))
                } else {
                    return Err(CompileError::UnknownAssignmentTarget {
                        name: receiver.to_string(),
                    });
                };
                self.emit_selector_path_set(target, &selector_path, src);
                if let Some(global) = global {
                    self.emitter.code.push(Instruction::StoreGlobal {
                        global,
                        src: target,
                    });
                }
                Ok(())
            }
            AssignTarget::Index { target, index } => {
                if self.scopes.captured_by_ref.contains(target) {
                    let pointer = self
                        .lookup_local(target)
                        .expect("captured local should be in scope");
                    let value = self.compile_value_expr(&Expr::Ident(target.clone()))?;
                    let index = self.compile_value_expr(index)?;
                    self.emitter.code.push(Instruction::SetIndex {
                        target: value,
                        index,
                        src,
                    });
                    self.emitter.code.push(Instruction::StoreIndirect {
                        target: pointer,
                        src: value,
                    });
                    return Ok(());
                }
                let (target, global) = if let Some(target) = self.lookup_local(target) {
                    (target, None)
                } else if let Some(global) = self.lookup_global(target) {
                    let target_register = self.alloc_register();
                    self.emitter.code.push(Instruction::LoadGlobal {
                        dst: target_register,
                        global,
                    });
                    (target_register, Some(global))
                } else {
                    return Err(CompileError::UnknownAssignmentTarget {
                        name: target.to_string(),
                    });
                };
                let index = self.compile_value_expr(index)?;
                self.emitter
                    .code
                    .push(Instruction::SetIndex { target, index, src });
                if let Some(global) = global {
                    self.emitter.code.push(Instruction::StoreGlobal {
                        global,
                        src: target,
                    });
                }
                Ok(())
            }
        }
    }

    pub(super) fn compile_pair_value(
        &mut self,
        value: &Expr,
        value_dst: usize,
        ok_dst: usize,
    ) -> Result<(), CompileError> {
        match value {
            Expr::Call {
                callee,
                type_args,
                args,
            } => self.compile_multi_call(callee, type_args, args, &[value_dst, ok_dst]),
            Expr::Index { target, index } => {
                let target = self.compile_value_expr(target)?;
                let index = self.compile_value_expr(index)?;
                self.emitter.code.push(Instruction::Index {
                    dst: value_dst,
                    target,
                    index,
                });
                self.emitter.code.push(Instruction::MapContains {
                    dst: ok_dst,
                    target,
                    index,
                });
                Ok(())
            }
            Expr::TypeAssert {
                expr,
                asserted_type,
            } => {
                let src = self.compile_value_expr(expr)?;
                self.ensure_runtime_visible_type(asserted_type)?;
                let target = self.lower_type_assert_target(asserted_type)?;
                self.emitter.code.push(Instruction::TypeMatches {
                    dst: ok_dst,
                    src,
                    target: target.clone(),
                });
                let failure_jump = self.emitter.code.len();
                self.emitter.code.push(Instruction::JumpIfFalse {
                    cond: ok_dst,
                    target: usize::MAX,
                });
                self.emitter.code.push(Instruction::AssertType {
                    dst: value_dst,
                    src,
                    target,
                });
                let end_jump = self.emitter.code.len();
                self.emitter.code.push(Instruction::Jump { target: usize::MAX });
                let failure_target = self.emitter.code.len();
                self.compile_zero_value(value_dst, asserted_type)?;
                let end_target = self.emitter.code.len();
                if let Instruction::JumpIfFalse { target, .. } = &mut self.emitter.code[failure_jump] {
                    *target = failure_target;
                }
                if let Instruction::Jump { target } = &mut self.emitter.code[end_jump] {
                    *target = end_target;
                }
                Ok(())
            }
            Expr::Unary {
                op: UnaryOp::Receive,
                expr,
            } => self.compile_receive_ok_expr(value_dst, ok_dst, expr),
            _ => Err(CompileError::Unsupported {
                detail: "comma-ok forms currently require a map index expression, type assertion, or channel receive"
                    .into(),
            }),
        }
    }

    pub(super) fn compile_triple_value(
        &mut self,
        value: &Expr,
        first_dst: usize,
        second_dst: usize,
        third_dst: usize,
    ) -> Result<(), CompileError> {
        match value {
            Expr::Call {
                callee,
                type_args,
                args,
            } => self.compile_multi_call(
                callee,
                type_args,
                args,
                &[first_dst, second_dst, third_dst],
            ),
            _ => Err(CompileError::Unsupported {
                detail: "triple-value forms currently require a multi-result call".into(),
            }),
        }
    }

    pub(super) fn compile_quad_value(
        &mut self,
        value: &Expr,
        first_dst: usize,
        second_dst: usize,
        third_dst: usize,
        fourth_dst: usize,
    ) -> Result<(), CompileError> {
        match value {
            Expr::Call {
                callee,
                type_args,
                args,
            } => self.compile_multi_call(
                callee,
                type_args,
                args,
                &[first_dst, second_dst, third_dst, fourth_dst],
            ),
            _ => Err(CompileError::Unsupported {
                detail: "quad-value forms currently require a multi-result call".into(),
            }),
        }
    }
}

fn short_decl_target_type(
    builder: &FunctionBuilder<'_>,
    name: &str,
    is_new: bool,
) -> Option<String> {
    if is_new || name == "_" {
        return None;
    }
    builder
        .lookup_local_type(name)
        .or_else(|| builder.lookup_global_type(name))
        .map(str::to_string)
}
