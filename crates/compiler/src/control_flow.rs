use super::*;
use gowasm_vm::CompareOp;

impl FunctionBuilder<'_> {
    fn validate_range_assign_target(
        &self,
        name: &str,
        source_type: Option<&str>,
    ) -> Result<(), CompileError> {
        if name == "_" {
            return Ok(());
        }

        let Some(target_type) = self
            .lookup_local_type(name)
            .or_else(|| self.lookup_global_type(name))
        else {
            return Err(CompileError::UnknownAssignmentTarget {
                name: name.to_string(),
            });
        };

        let Some(source_type) = source_type else {
            return Ok(());
        };
        if self.types_assignable(target_type, source_type) {
            return Ok(());
        }

        Err(CompileError::Unsupported {
            detail: format!(
                "range value of type `{source_type}` is not assignable to `{target_type}` in the current subset"
            ),
        })
    }

    fn range_binding_types(
        &self,
        expr: &Expr,
    ) -> Result<(Option<String>, Option<String>), CompileError> {
        let Some(typ) = self.infer_expr_type_name(expr) else {
            return Err(CompileError::Unsupported {
                detail: "range requires a known iterable type in the current subset".into(),
            });
        };
        if let Some((_, element_type)) = parse_array_type(&typ) {
            return Ok((Some("int".into()), Some(element_type.to_string())));
        }
        if let Some(element_type) = typ.strip_prefix("[]") {
            return Ok((Some("int".into()), Some(element_type.to_string())));
        }
        if let Some((key_type, value_type)) = parse_map_type(&typ) {
            return Ok((Some(key_type.to_string()), Some(value_type.to_string())));
        }
        if typ == "string" {
            return Ok((Some("int".into()), Some("int".into())));
        }
        if typ == "int" {
            return Ok((Some("int".into()), None));
        }
        Err(CompileError::Unsupported {
            detail: format!("cannot range over non-iterable type `{typ}` in the current subset"),
        })
    }

    pub(super) fn compile_range_for(
        &mut self,
        key: &str,
        value: Option<&str>,
        assign: bool,
        expr: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        if let Some(iterable_type) = self.infer_expr_type_name(expr) {
            if parse_channel_type(&iterable_type).is_some() {
                return self.compile_channel_range_for(key, value, assign, expr, body);
            }
            if iterable_type == "int" {
                return self.compile_int_range_for(key, value, assign, expr, body);
            }
        }

        let len_builtin =
            resolve_stdlib_function("builtin", "len").expect("builtin len should be registered");
        let range_keys_builtin = resolve_stdlib_function("builtin", "__range_keys")
            .expect("internal range keys helper should be registered");
        let range_value_builtin = resolve_stdlib_function("builtin", "__range_value")
            .expect("internal range value helper should be registered");

        self.begin_scope();
        let iterable = self.compile_value_expr(expr)?;
        let keys = self.alloc_register();
        self.emitter.code.push(Instruction::CallStdlib {
            function: range_keys_builtin,
            args: vec![iterable],
            dst: Some(keys),
        });
        let len = self.alloc_register();
        self.emitter.code.push(Instruction::CallStdlib {
            function: len_builtin,
            args: vec![keys],
            dst: Some(len),
        });
        let index = self.alloc_register();
        self.emitter.code.push(Instruction::LoadInt {
            dst: index,
            value: 0,
        });
        let (key_type, value_type) = self.range_binding_types(expr)?;
        if assign {
            self.validate_range_assign_target(key, key_type.as_deref())?;
            if let Some(value_name) = value {
                self.validate_range_assign_target(value_name, value_type.as_deref())?;
            }
        }
        let key_binding = if assign {
            None
        } else {
            self.bind_local(key, key_type.as_deref())?
        };
        let value_binding = if assign {
            None
        } else {
            value
                .map(|name| self.bind_local(name, value_type.as_deref()))
                .transpose()?
                .flatten()
        };
        let loop_start = self.emitter.code.len();
        let cond = self.alloc_register();
        self.emitter.code.push(Instruction::Compare {
            dst: cond,
            op: CompareOp::Less,
            left: index,
            right: len,
        });
        let jump_to_end = self.push_jump_if_false(cond);

        let current_key = self.alloc_register();
        self.emitter.code.push(Instruction::Index {
            dst: current_key,
            target: keys,
            index,
        });
        if assign || key_binding.is_some() {
            let _ = key_binding;
            self.store_local_binding(key, current_key)?;
        }
        if assign || value_binding.is_some() {
            let current_value = self.alloc_register();
            self.emitter.code.push(Instruction::CallStdlib {
                function: range_value_builtin,
                args: vec![iterable, current_key],
                dst: Some(current_value),
            });
            let _ = value_binding;
            self.store_local_binding(value.expect("range value exists"), current_value)?;
        }

        let label = self.control.pending_label.take();
        self.control.break_scopes.push(BreakContext {
            label: label.clone(),
            break_jumps: Vec::new(),
        });
        self.control.loops.push(LoopContext {
            label,
            continue_jumps: Vec::new(),
        });
        self.compile_block(body)?;
        let loop_context = self.control.loops.pop().expect("loop context should exist");
        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("loop break context should exist");

        let continue_target = self.emitter.code.len();
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(continue_jump, continue_target);
        }

        let one = self.alloc_register();
        self.emitter
            .code
            .push(Instruction::LoadInt { dst: one, value: 1 });
        self.emitter.code.push(Instruction::Add {
            dst: index,
            left: index,
            right: one,
        });
        self.emitter
            .code
            .push(Instruction::Jump { target: loop_start });

        let end_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_to_end, end_target);
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        self.end_scope();
        Ok(())
    }

    fn compile_channel_range_for(
        &mut self,
        key: &str,
        value: Option<&str>,
        assign: bool,
        expr: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        let element_type = self.channel_element_type(expr)?;
        if value.is_some() {
            return Err(CompileError::Unsupported {
                detail: "channel `range` currently supports only one binding in the current subset"
                    .into(),
            });
        }

        self.begin_scope();
        let chan = self.compile_value_expr(expr)?;
        if assign {
            self.validate_range_assign_target(key, Some(&element_type))?;
        }
        let value_binding = if assign {
            None
        } else {
            self.bind_local(key, Some(&element_type))?
        };
        let loop_start = self.emitter.code.len();
        let current_value = self.alloc_register();
        let current_ok = self.alloc_register();
        self.emitter.code.push(Instruction::ChanRecvOk {
            value_dst: current_value,
            ok_dst: current_ok,
            chan,
        });
        let jump_to_end = self.push_jump_if_false(current_ok);

        if assign || value_binding.is_some() {
            self.store_local_binding(key, current_value)?;
        }

        let label = self.control.pending_label.take();
        self.control.break_scopes.push(BreakContext {
            label: label.clone(),
            break_jumps: Vec::new(),
        });
        self.control.loops.push(LoopContext {
            label,
            continue_jumps: Vec::new(),
        });
        self.compile_block(body)?;
        let loop_context = self.control.loops.pop().expect("loop context should exist");
        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("loop break context should exist");

        let continue_target = self.emitter.code.len();
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(continue_jump, continue_target);
        }

        self.emitter
            .code
            .push(Instruction::Jump { target: loop_start });

        let end_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_to_end, end_target);
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        self.end_scope();
        Ok(())
    }

    fn compile_int_range_for(
        &mut self,
        key: &str,
        value: Option<&str>,
        assign: bool,
        expr: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        if value.is_some() {
            return Err(CompileError::Unsupported {
                detail: "range over int does not produce a second value".into(),
            });
        }

        self.begin_scope();
        let limit = self.compile_value_expr(expr)?;
        let index = self.alloc_register();
        self.emitter.code.push(Instruction::LoadInt {
            dst: index,
            value: 0,
        });
        if assign {
            self.validate_range_assign_target(key, Some("int"))?;
        }
        let key_binding = if assign {
            None
        } else {
            self.bind_local(key, Some("int"))?
        };

        let loop_start = self.emitter.code.len();
        let cond = self.alloc_register();
        self.emitter.code.push(Instruction::Compare {
            dst: cond,
            op: CompareOp::Less,
            left: index,
            right: limit,
        });
        let jump_to_end = self.push_jump_if_false(cond);

        if assign || key_binding.is_some() {
            self.store_local_binding(key, index)?;
        }

        let label = self.control.pending_label.take();
        self.control.break_scopes.push(BreakContext {
            label: label.clone(),
            break_jumps: Vec::new(),
        });
        self.control.loops.push(LoopContext {
            label,
            continue_jumps: Vec::new(),
        });
        self.compile_block(body)?;
        let loop_context = self.control.loops.pop().expect("loop context should exist");
        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("loop break context should exist");

        let continue_target = self.emitter.code.len();
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(continue_jump, continue_target);
        }

        let one = self.alloc_register();
        self.emitter
            .code
            .push(Instruction::LoadInt { dst: one, value: 1 });
        self.emitter.code.push(Instruction::Add {
            dst: index,
            left: index,
            right: one,
        });
        self.emitter
            .code
            .push(Instruction::Jump { target: loop_start });

        let end_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_to_end, end_target);
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        self.end_scope();
        Ok(())
    }

    pub(super) fn compile_if(
        &mut self,
        init: Option<&Stmt>,
        condition: &Expr,
        then_body: &[Stmt],
        else_body: Option<&[Stmt]>,
    ) -> Result<(), CompileError> {
        let has_init = init.is_some();
        if has_init {
            self.begin_scope();
            self.compile_stmt(init.unwrap())?;
        }

        let cond = self.compile_value_expr(condition)?;
        let jump_if_false = self.push_jump_if_false(cond);
        self.compile_block(then_body)?;

        if let Some(else_body) = else_body {
            let jump_to_end = self.push_jump();
            let else_target = self.emitter.code.len();
            self.patch_jump_if_false(jump_if_false, else_target);
            self.compile_block(else_body)?;
            let end_target = self.emitter.code.len();
            self.patch_jump(jump_to_end, end_target);
        } else {
            let end_target = self.emitter.code.len();
            self.patch_jump_if_false(jump_if_false, end_target);
        }

        if has_init {
            self.end_scope();
        }

        Ok(())
    }

    pub(super) fn compile_select_case(
        &mut self,
        case: &SelectCase,
        exit_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        match &case.stmt {
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Receive,
                expr,
            }) => {
                let ready = self.alloc_register();
                let value = self.alloc_register();
                self.compile_try_receive_expr(ready, value, expr)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.compile_block(&case.body)?;
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            Stmt::ShortVarDecl {
                name,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                let ready = self.alloc_register();
                let value = self.alloc_register();
                self.compile_try_receive_expr(ready, value, expr)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.begin_scope();
                let is_new = self.validate_short_decl_names(&[name.as_str()])?;
                if is_new[0] {
                    self.bind_local(name, Some(&element_type))?;
                } else if name.as_str() != "_" {
                    self.store_local_binding(name, value)?;
                }
                self.compile_block(&case.body)?;
                self.end_scope();
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            Stmt::ShortVarDeclPair {
                first,
                second,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                let ready = self.alloc_register();
                let value = self.alloc_register();
                let ok = self.alloc_register();
                self.compile_try_receive_ok_expr(ready, value, ok, expr)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.begin_scope();
                let is_new =
                    self.validate_short_decl_names(&[first.as_str(), second.as_str()])?;
                if is_new[0] {
                    self.bind_local(first, Some(&element_type))?;
                } else if first.as_str() != "_" {
                    self.store_local_binding(first, value)?;
                }

                if is_new[1] {
                    self.bind_local(second, Some("bool"))?;
                } else if second.as_str() != "_" {
                    self.store_local_binding(second, ok)?;
                }
                self.compile_block(&case.body)?;
                self.end_scope();
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            Stmt::Assign {
                target,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                self.validate_select_assign_target_type(target, &element_type)?;
                let ready = self.alloc_register();
                let value = self.alloc_register();
                self.compile_try_receive_expr(ready, value, expr)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.compile_assign_from_register(target, value)?;
                self.compile_block(&case.body)?;
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            Stmt::AssignPair {
                first,
                second,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                self.validate_select_assign_target_type(first, &element_type)?;
                self.validate_select_assign_target_type(second, "bool")?;
                let ready = self.alloc_register();
                let value = self.alloc_register();
                let ok = self.alloc_register();
                self.compile_try_receive_ok_expr(ready, value, ok, expr)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.compile_assign_from_register(first, value)?;
                self.compile_assign_from_register(second, ok)?;
                self.compile_block(&case.body)?;
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            Stmt::Send { chan, value } => {
                let ready = self.alloc_register();
                self.compile_try_send_stmt(ready, chan, value)?;
                let jump_if_false = self.push_jump_if_false(ready);
                self.compile_block(&case.body)?;
                exit_jumps.push(self.push_jump());
                let next_case_target = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_case_target);
                Ok(())
            }
            other => Err(CompileError::Unsupported {
                detail: format!(
                    "`select` currently supports only send and receive cases with `default`; found {other:?}"
                ),
            }),
        }
    }

    pub(super) fn validate_select_assign_target_type(
        &self,
        target: &AssignTarget,
        source_type: &str,
    ) -> Result<(), CompileError> {
        let Some(target_type) = self.select_assign_target_type(target) else {
            return Ok(());
        };
        if target_type == source_type {
            return Ok(());
        }
        if function_signatures_match(&target_type, source_type) {
            return Ok(());
        }
        if self
            .env
            .interface_types
            .get(&target_type)
            .is_some_and(|interface_type| {
                self.type_satisfies_interface(&target_type, source_type, interface_type)
            })
        {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "cannot assign `{source_type}` to `{target_type}` in `select` receive case"
            ),
        })
    }

    fn select_assign_target_type(&self, target: &AssignTarget) -> Option<String> {
        match target {
            AssignTarget::Ident(name) => self
                .lookup_local_type(name)
                .or_else(|| self.lookup_global_type(name))
                .map(str::to_string),
            AssignTarget::Deref { target } => self
                .lookup_local_type(target)
                .or_else(|| self.lookup_global_type(target))
                .and_then(parse_pointer_type)
                .map(str::to_string),
            AssignTarget::DerefSelector { .. }
            | AssignTarget::DerefIndex { .. }
            | AssignTarget::Selector { .. }
            | AssignTarget::Index { .. } => None,
        }
    }

    pub(super) fn compile_for(
        &mut self,
        init: Option<&Stmt>,
        condition: Option<&Expr>,
        post: Option<&Stmt>,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        self.begin_scope();
        if let Some(init) = init {
            self.compile_stmt(init)?;
        }

        let loop_start = self.emitter.code.len();
        let jump_to_end = if let Some(condition) = condition {
            let cond = self.compile_value_expr(condition)?;
            Some(self.push_jump_if_false(cond))
        } else {
            None
        };

        let label = self.control.pending_label.take();
        self.control.break_scopes.push(BreakContext {
            label: label.clone(),
            break_jumps: Vec::new(),
        });
        self.control.loops.push(LoopContext {
            label,
            continue_jumps: Vec::new(),
        });
        self.compile_block(body)?;
        let loop_context = self.control.loops.pop().expect("loop context should exist");
        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("loop break context should exist");

        let continue_target = if post.is_some() {
            self.emitter.code.len()
        } else {
            loop_start
        };
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(continue_jump, continue_target);
        }

        if let Some(post) = post {
            self.compile_stmt(post)?;
        }
        self.emitter
            .code
            .push(Instruction::Jump { target: loop_start });

        let end_target = self.emitter.code.len();
        if let Some(jump_to_end) = jump_to_end {
            self.patch_jump_if_false(jump_to_end, end_target);
        }
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        self.end_scope();

        Ok(())
    }

    pub(super) fn compile_labeled(&mut self, label: &str, stmt: &Stmt) -> Result<(), CompileError> {
        self.control.pending_label = Some(label.to_string());
        self.compile_stmt(stmt)?;
        self.control.pending_label = None;
        Ok(())
    }

    pub(super) fn compile_break(&mut self, label: Option<&str>) -> Result<(), CompileError> {
        let jump = self.push_jump();
        let break_context = if let Some(label) = label {
            self.control
                .break_scopes
                .iter_mut()
                .rev()
                .find(|ctx| ctx.label.as_deref() == Some(label))
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!("label `{label}` not found for break"),
                })?
        } else {
            self.control
                .break_scopes
                .last_mut()
                .ok_or(CompileError::BreakOutsideBreakable)?
        };
        break_context.break_jumps.push(jump);
        Ok(())
    }

    pub(super) fn compile_continue(&mut self, label: Option<&str>) -> Result<(), CompileError> {
        let jump = self.push_jump();
        let loop_context = if let Some(label) = label {
            self.control
                .loops
                .iter_mut()
                .rev()
                .find(|ctx| ctx.label.as_deref() == Some(label))
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!("label `{label}` not found for continue"),
                })?
        } else {
            self.control
                .loops
                .last_mut()
                .ok_or(CompileError::ContinueOutsideLoop)?
        };
        loop_context.continue_jumps.push(jump);
        Ok(())
    }

    pub(super) fn push_jump(&mut self) -> usize {
        let index = self.emitter.code.len();
        self.emitter
            .code
            .push(Instruction::Jump { target: usize::MAX });
        index
    }

    pub(super) fn push_jump_if_false(&mut self, cond: usize) -> usize {
        let index = self.emitter.code.len();
        self.emitter.code.push(Instruction::JumpIfFalse {
            cond,
            target: usize::MAX,
        });
        index
    }

    pub(super) fn patch_jump(&mut self, index: usize, target: usize) {
        match self.emitter.code.get_mut(index) {
            Some(Instruction::Jump {
                target: jump_target,
            }) => *jump_target = target,
            _ => panic!("jump placeholder should exist"),
        }
    }

    pub(super) fn patch_jump_if_false(&mut self, index: usize, target: usize) {
        match self.emitter.code.get_mut(index) {
            Some(Instruction::JumpIfFalse {
                target: jump_target,
                ..
            }) => *jump_target = target,
            _ => panic!("conditional jump placeholder should exist"),
        }
    }
}
