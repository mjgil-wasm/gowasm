use super::*;
use gowasm_vm::CompareOp;

impl FunctionBuilder<'_> {
    pub(super) fn compile_switch(
        &mut self,
        init: Option<&Stmt>,
        expr: Option<&Expr>,
        cases: &[SwitchCase],
        default: Option<&[Stmt]>,
        default_index: Option<usize>,
        default_fallthrough: bool,
    ) -> Result<(), CompileError> {
        let has_init = init.is_some();
        if has_init {
            self.begin_scope();
            self.compile_stmt(init.unwrap())?;
        }

        let switch_value = expr.map(|expr| self.compile_value_expr(expr)).transpose()?;
        let mut exit_jumps = Vec::new();

        self.control.break_scopes.push(BreakContext {
            label: self.control.pending_label.take(),
            break_jumps: Vec::new(),
        });
        let mut fallthrough_jumps = Vec::new();
        let clause_count = cases.len() + usize::from(default.is_some());
        let mut case_index = 0usize;
        for clause_index in 0..clause_count {
            if default_index == Some(clause_index) {
                self.compile_switch_default(
                    default.expect("default clause should exist"),
                    clause_index + 1 < clause_count,
                    default_fallthrough,
                    &mut exit_jumps,
                    &mut fallthrough_jumps,
                )?;
                continue;
            }
            self.compile_switch_case(
                switch_value,
                &cases[case_index],
                clause_index + 1 < clause_count,
                &mut exit_jumps,
                &mut fallthrough_jumps,
            )?;
            case_index += 1;
        }

        let end_target = self.emitter.code.len();
        for exit_jump in exit_jumps {
            self.patch_jump(exit_jump, end_target);
        }

        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("switch break context should exist");
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }

        if has_init {
            self.end_scope();
        }
        Ok(())
    }

    pub(super) fn compile_type_switch(
        &mut self,
        init: Option<&Stmt>,
        binding: Option<&str>,
        expr: &Expr,
        cases: &[TypeSwitchCase],
        default: Option<&[Stmt]>,
        default_index: Option<usize>,
    ) -> Result<(), CompileError> {
        let has_init = init.is_some();
        if has_init {
            self.begin_scope();
            self.compile_stmt(init.unwrap())?;
        }

        let source_type = self.infer_expr_type_name(expr);
        let src = self.compile_value_expr(expr)?;
        let mut exit_jumps = Vec::new();

        self.control.break_scopes.push(BreakContext {
            label: self.control.pending_label.take(),
            break_jumps: Vec::new(),
        });

        let clause_count = cases.len() + usize::from(default.is_some());
        let mut case_index = 0usize;
        for clause_index in 0..clause_count {
            if default_index == Some(clause_index) {
                self.compile_type_switch_default(
                    binding,
                    src,
                    source_type.clone(),
                    default,
                    &mut exit_jumps,
                )?;
                continue;
            }
            let case = &cases[case_index];
            let mut body_jumps = Vec::with_capacity(case.types.len());
            for type_name in &case.types {
                let matches_reg = self.alloc_register();
                if type_name == "nil" {
                    self.emitter.code.push(Instruction::IsNil {
                        dst: matches_reg,
                        src,
                    });
                } else {
                    self.ensure_runtime_visible_type(type_name)?;
                    let target = self.lower_type_assert_target(type_name)?;
                    self.emitter.code.push(Instruction::TypeMatches {
                        dst: matches_reg,
                        src,
                        target,
                    });
                }
                let jump_if_false = self.push_jump_if_false(matches_reg);
                let jump_to_body = self.push_jump();
                body_jumps.push(jump_to_body);
                let next_test = self.emitter.code.len();
                self.patch_jump_if_false(jump_if_false, next_test);
            }

            let skip_body = self.push_jump();
            let body_target = self.emitter.code.len();
            for body_jump in body_jumps {
                self.patch_jump(body_jump, body_target);
            }

            self.begin_scope();
            if let Some(binding) = binding {
                let binding_reg = self.alloc_register();
                let binding_type = if case.types.len() == 1 {
                    let binding_type = case.types[0].clone();
                    if binding_type == "nil" {
                        self.emitter.code.push(Instruction::Move {
                            dst: binding_reg,
                            src,
                        });
                        source_type.clone()
                    } else {
                        let target = self.lower_type_assert_target(&binding_type)?;
                        self.emitter.code.push(Instruction::AssertType {
                            dst: binding_reg,
                            src,
                            target,
                        });
                        Some(binding_type)
                    }
                } else {
                    self.emitter.code.push(Instruction::Move {
                        dst: binding_reg,
                        src,
                    });
                    source_type.clone()
                };
                self.scopes
                    .last_mut()
                    .expect("scope should exist")
                    .insert(binding.to_string(), binding_reg);
                if let Some(binding_type) = binding_type {
                    self.current_type_scope_mut()
                        .insert(binding.to_string(), binding_type);
                }
            }
            self.compile_block(&case.body)?;
            self.end_scope();

            exit_jumps.push(self.push_jump());
            let next_clause = self.emitter.code.len();
            self.patch_jump(skip_body, next_clause);
            case_index += 1;
        }

        let end_target = self.emitter.code.len();
        for exit_jump in exit_jumps {
            self.patch_jump(exit_jump, end_target);
        }
        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("type switch break context should exist");
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        if has_init {
            self.end_scope();
        }
        Ok(())
    }

    fn compile_switch_case(
        &mut self,
        switch_value: Option<usize>,
        case: &SwitchCase,
        has_next_clause: bool,
        exit_jumps: &mut Vec<usize>,
        fallthrough_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let mut body_jumps = Vec::with_capacity(case.expressions.len());
        for case_expr in &case.expressions {
            let cond = self.compile_switch_case_condition(switch_value, case_expr)?;
            let jump_if_false = self.push_jump_if_false(cond);
            let jump_to_body = self.push_jump();
            body_jumps.push(jump_to_body);
            let next_test_target = self.emitter.code.len();
            self.patch_jump_if_false(jump_if_false, next_test_target);
        }

        let skip_body = self.push_jump();
        let body_target = self.emitter.code.len();
        for body_jump in body_jumps {
            self.patch_jump(body_jump, body_target);
        }
        for ft_jump in fallthrough_jumps.drain(..) {
            self.patch_jump(ft_jump, body_target);
        }
        self.compile_block(&case.body)?;
        if case.fallthrough {
            if !has_next_clause {
                return Err(CompileError::Unsupported {
                    detail: "`fallthrough` cannot appear in the final `switch` clause".into(),
                });
            }
            fallthrough_jumps.push(self.push_jump());
        } else {
            exit_jumps.push(self.push_jump());
        }

        let next_clause_target = self.emitter.code.len();
        self.patch_jump(skip_body, next_clause_target);
        Ok(())
    }

    fn compile_switch_case_condition(
        &mut self,
        switch_value: Option<usize>,
        case_expr: &Expr,
    ) -> Result<usize, CompileError> {
        match switch_value {
            Some(switch_value) => {
                let case_value = self.compile_value_expr(case_expr)?;
                let cond = self.alloc_register();
                self.emitter.code.push(Instruction::Compare {
                    dst: cond,
                    op: CompareOp::Equal,
                    left: switch_value,
                    right: case_value,
                });
                Ok(cond)
            }
            None => self.compile_value_expr(case_expr),
        }
    }

    fn compile_switch_default(
        &mut self,
        body: &[Stmt],
        has_next_clause: bool,
        fallthrough: bool,
        exit_jumps: &mut Vec<usize>,
        fallthrough_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let body_target = self.emitter.code.len();
        for ft_jump in fallthrough_jumps.drain(..) {
            self.patch_jump(ft_jump, body_target);
        }
        self.compile_block(body)?;
        if fallthrough {
            if !has_next_clause {
                return Err(CompileError::Unsupported {
                    detail: "`fallthrough` cannot appear in the final `switch` clause".into(),
                });
            }
            fallthrough_jumps.push(self.push_jump());
        } else {
            exit_jumps.push(self.push_jump());
        }
        Ok(())
    }

    fn compile_type_switch_default(
        &mut self,
        binding: Option<&str>,
        src: usize,
        source_type: Option<String>,
        default: Option<&[Stmt]>,
        exit_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let Some(default) = default else {
            return Ok(());
        };
        self.begin_scope();
        if let Some(binding) = binding {
            let binding_reg = self.alloc_register();
            self.emitter.code.push(Instruction::Move {
                dst: binding_reg,
                src,
            });
            self.scopes
                .last_mut()
                .expect("scope should exist")
                .insert(binding.to_string(), binding_reg);
            if let Some(binding_type) = source_type {
                self.current_type_scope_mut()
                    .insert(binding.to_string(), binding_type);
            }
        }
        self.compile_block(default)?;
        self.end_scope();
        exit_jumps.push(self.push_jump());
        Ok(())
    }
}
