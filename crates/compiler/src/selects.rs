use super::*;
use gowasm_vm::{CompareOp, SelectCaseOp, SelectCaseOpKind};

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockingSelectCasePlan {
    op: SelectCaseOp,
    kind: BlockingSelectCaseKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockingSelectCaseKind {
    Recv {
        value_dst: usize,
        ok_dst: Option<usize>,
        element_type: String,
    },
    Send,
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_select(
        &mut self,
        cases: &[SelectCase],
        default: Option<&[Stmt]>,
    ) -> Result<(), CompileError> {
        let mut exit_jumps = Vec::new();
        self.control.break_scopes.push(BreakContext {
            label: self.control.pending_label.take(),
            break_jumps: Vec::new(),
        });
        if let Some(default) = default {
            if cases.is_empty() {
                let choice_dst = self.alloc_register();
                self.emitter.code.push(Instruction::Select {
                    choice_dst,
                    cases: Vec::new(),
                    default_case: Some(0),
                });
                self.compile_block(default)?;
            } else if cases
                .iter()
                .all(|case| Self::select_case_uses_vm_path(&case.stmt))
            {
                self.compile_select_with_default_vm(cases, default, &mut exit_jumps)?;
            } else {
                for case in cases {
                    self.compile_select_case(case, &mut exit_jumps)?;
                }
                self.compile_block(default)?;
            }
        } else {
            self.compile_blocking_select(cases, &mut exit_jumps)?;
        }

        let end_target = self.emitter.code.len();
        for exit_jump in exit_jumps {
            self.patch_jump(exit_jump, end_target);
        }

        let break_context = self
            .control
            .break_scopes
            .pop()
            .expect("select break context should exist");
        for break_jump in break_context.break_jumps {
            self.patch_jump(break_jump, end_target);
        }
        Ok(())
    }

    fn select_case_uses_vm_path(stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Receive,
                ..
            }) | Stmt::ShortVarDecl {
                value: Expr::Unary {
                    op: UnaryOp::Receive,
                    ..
                },
                ..
            } | Stmt::ShortVarDeclPair {
                value: Expr::Unary {
                    op: UnaryOp::Receive,
                    ..
                },
                ..
            } | Stmt::Assign {
                value: Expr::Unary {
                    op: UnaryOp::Receive,
                    ..
                },
                ..
            } | Stmt::AssignPair {
                value: Expr::Unary {
                    op: UnaryOp::Receive,
                    ..
                },
                ..
            } | Stmt::Send { .. }
        )
    }

    fn compile_select_with_default_vm(
        &mut self,
        cases: &[SelectCase],
        default: &[Stmt],
        exit_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let choice_dst = self.alloc_register();
        let mut plans = Vec::with_capacity(cases.len());
        let mut ops = Vec::with_capacity(cases.len());
        for case in cases {
            let plan = self.plan_blocking_select_case(case)?;
            ops.push(plan.op.clone());
            plans.push(plan);
        }
        self.emitter.code.push(Instruction::Select {
            choice_dst,
            cases: ops,
            default_case: Some(cases.len()),
        });
        for (index, (case, plan)) in cases.iter().zip(plans.iter()).enumerate() {
            self.compile_blocking_select_case_dispatch(choice_dst, index, case, plan, exit_jumps)?;
        }
        self.compile_block(default)?;
        Ok(())
    }

    fn compile_blocking_select(
        &mut self,
        cases: &[SelectCase],
        exit_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let choice_dst = self.alloc_register();
        let mut plans = Vec::with_capacity(cases.len());
        let mut ops = Vec::with_capacity(cases.len());
        for case in cases {
            let plan = self.plan_blocking_select_case(case)?;
            ops.push(plan.op.clone());
            plans.push(plan);
        }
        self.emitter.code.push(Instruction::Select {
            choice_dst,
            cases: ops,
            default_case: None,
        });
        for (index, (case, plan)) in cases.iter().zip(plans.iter()).enumerate() {
            self.compile_blocking_select_case_dispatch(choice_dst, index, case, plan, exit_jumps)?;
        }
        Ok(())
    }

    fn plan_blocking_select_case(
        &mut self,
        case: &SelectCase,
    ) -> Result<BlockingSelectCasePlan, CompileError> {
        match &case.stmt {
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Receive,
                expr,
            })
            | Stmt::ShortVarDecl {
                name: _,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                let chan = self.compile_value_expr(expr)?;
                let value_dst = self.alloc_register();
                Ok(BlockingSelectCasePlan {
                    op: SelectCaseOp {
                        chan,
                        kind: SelectCaseOpKind::Recv {
                            value_dst,
                            ok_dst: None,
                        },
                    },
                    kind: BlockingSelectCaseKind::Recv {
                        value_dst,
                        ok_dst: None,
                        element_type,
                    },
                })
            }
            Stmt::ShortVarDeclPair {
                first: _,
                second: _,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr,
                    },
            } => {
                let element_type = self.channel_element_type(expr)?;
                let chan = self.compile_value_expr(expr)?;
                let value_dst = self.alloc_register();
                let ok_dst = self.alloc_register();
                Ok(BlockingSelectCasePlan {
                    op: SelectCaseOp {
                        chan,
                        kind: SelectCaseOpKind::Recv {
                            value_dst,
                            ok_dst: Some(ok_dst),
                        },
                    },
                    kind: BlockingSelectCaseKind::Recv {
                        value_dst,
                        ok_dst: Some(ok_dst),
                        element_type,
                    },
                })
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
                let chan = self.compile_value_expr(expr)?;
                let value_dst = self.alloc_register();
                Ok(BlockingSelectCasePlan {
                    op: SelectCaseOp {
                        chan,
                        kind: SelectCaseOpKind::Recv {
                            value_dst,
                            ok_dst: None,
                        },
                    },
                    kind: BlockingSelectCaseKind::Recv {
                        value_dst,
                        ok_dst: None,
                        element_type,
                    },
                })
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
                let chan = self.compile_value_expr(expr)?;
                let value_dst = self.alloc_register();
                let ok_dst = self.alloc_register();
                Ok(BlockingSelectCasePlan {
                    op: SelectCaseOp {
                        chan,
                        kind: SelectCaseOpKind::Recv {
                            value_dst,
                            ok_dst: Some(ok_dst),
                        },
                    },
                    kind: BlockingSelectCaseKind::Recv {
                        value_dst,
                        ok_dst: Some(ok_dst),
                        element_type,
                    },
                })
            }
            Stmt::Send { chan, value } => {
                let element_type = self.send_channel_element_type(chan)?;
                self.validate_assignable_type(Some(&element_type), value)?;
                let chan = self.compile_value_expr(chan)?;
                let value = self.compile_value_expr(value)?;
                Ok(BlockingSelectCasePlan {
                    op: SelectCaseOp {
                        chan,
                        kind: SelectCaseOpKind::Send { value },
                    },
                    kind: BlockingSelectCaseKind::Send,
                })
            }
            other => Err(CompileError::Unsupported {
                detail: format!(
                    "blocking `select` currently supports only send and receive cases; found {other:?}"
                ),
            }),
        }
    }

    fn compile_blocking_select_case_dispatch(
        &mut self,
        choice_dst: usize,
        case_index: usize,
        case: &SelectCase,
        plan: &BlockingSelectCasePlan,
        exit_jumps: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        let expected = self.alloc_register();
        self.emitter.code.push(Instruction::LoadInt {
            dst: expected,
            value: case_index as i64,
        });
        let cond = self.alloc_register();
        self.emitter.code.push(Instruction::Compare {
            dst: cond,
            op: CompareOp::Equal,
            left: choice_dst,
            right: expected,
        });
        let jump_if_false = self.push_jump_if_false(cond);

        match &case.stmt {
            Stmt::Send { .. } => {
                debug_assert!(matches!(plan.kind, BlockingSelectCaseKind::Send));
                self.compile_block(&case.body)?;
            }
            Stmt::Expr(Expr::Unary {
                op: UnaryOp::Receive,
                expr: _,
            }) => {
                debug_assert!(matches!(plan.kind, BlockingSelectCaseKind::Recv { .. }));
                self.compile_block(&case.body)?;
            }
            Stmt::ShortVarDecl {
                name,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr: _,
                    },
            } => {
                let BlockingSelectCaseKind::Recv {
                    value_dst,
                    element_type,
                    ..
                } = &plan.kind
                else {
                    unreachable!("blocking receive case should keep receive metadata");
                };
                self.begin_scope();
                let is_new = self.validate_short_decl_names(&[name.as_str()])?;
                if is_new[0] {
                    self.bind_initialized_local(name, *value_dst, Some(element_type.clone()));
                } else if name.as_str() != "_" {
                    self.store_local_binding(name, *value_dst)?;
                }
                self.compile_block(&case.body)?;
                self.end_scope();
            }
            Stmt::ShortVarDeclPair {
                first,
                second,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr: _,
                    },
            } => {
                let BlockingSelectCaseKind::Recv {
                    value_dst,
                    ok_dst,
                    element_type,
                } = &plan.kind
                else {
                    unreachable!("blocking receive pair should keep receive metadata");
                };
                let ok_dst = ok_dst.expect("select pair receive should have ok register");
                self.begin_scope();
                let is_new = self.validate_short_decl_names(&[first.as_str(), second.as_str()])?;
                if is_new[0] {
                    self.bind_initialized_local(first, *value_dst, Some(element_type.clone()));
                } else if first.as_str() != "_" {
                    self.store_local_binding(first, *value_dst)?;
                }

                if is_new[1] {
                    self.bind_initialized_local(second, ok_dst, Some("bool".into()));
                } else if second.as_str() != "_" {
                    self.store_local_binding(second, ok_dst)?;
                }
                self.compile_block(&case.body)?;
                self.end_scope();
            }
            Stmt::Assign {
                target,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr: _,
                    },
            } => {
                let BlockingSelectCaseKind::Recv { value_dst, .. } = &plan.kind else {
                    unreachable!("blocking receive assignment should keep receive metadata");
                };
                self.compile_assign_from_register(target, *value_dst)?;
                self.compile_block(&case.body)?;
            }
            Stmt::AssignPair {
                first,
                second,
                value:
                    Expr::Unary {
                        op: UnaryOp::Receive,
                        expr: _,
                    },
            } => {
                let BlockingSelectCaseKind::Recv {
                    value_dst, ok_dst, ..
                } = &plan.kind
                else {
                    unreachable!("blocking receive pair assignment should keep receive metadata");
                };
                let ok_dst = ok_dst.expect("select pair receive should have ok register");
                self.compile_assign_from_register(first, *value_dst)?;
                self.compile_assign_from_register(second, ok_dst)?;
                self.compile_block(&case.body)?;
            }
            _ => unreachable!("blocking select should reject unsupported cases during planning"),
        }

        exit_jumps.push(self.push_jump());
        let next_case_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_if_false, next_case_target);
        Ok(())
    }
}
