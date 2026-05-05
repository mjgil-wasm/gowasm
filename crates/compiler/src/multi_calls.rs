use super::*;

use super::diagnostics::CallableContext;

impl FunctionBuilder<'_> {
    pub(super) fn compile_multi_call(
        &mut self,
        callee: &Expr,
        type_args: &[String],
        args: &[Expr],
        dsts: &[usize],
    ) -> Result<(), CompileError> {
        if let Some(generic_call) = self.resolve_generic_call(callee, type_args, args) {
            let generic_call = generic_call?;
            return self.compile_generic_multi_call(&generic_call, args, dsts);
        }
        match callee {
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(package_path) =
                        self.env.imported_packages.get(receiver_name).cloned()
                    {
                        if self.try_compile_imported_selector_multi_call(
                            receiver_name,
                            &package_path,
                            field,
                            args,
                            dsts,
                        )? {
                            return Ok(());
                        }
                        return Err(CompileError::Unsupported {
                            detail: format!("unsupported selector call `{receiver_name}.{field}`"),
                        });
                    }
                }
                if self.selector_is_method_expression(receiver, field) {
                    return self.compile_method_expression_multi_call(callee, args, dsts);
                }

                self.validate_selector_call_arity(receiver, field, args)?;
                self.validate_selector_call_types(receiver, field, args)?;

                if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                    let result_count = stdlib_function_result_count(function);
                    if result_count != dsts.len() {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` returns {result_count} value(s), not {}",
                                dsts.len()
                            ),
                        });
                    }
                    if stdlib_function_mutates_first_arg(function) {
                        return self
                            .compile_mutating_method_multi_call(receiver, field, args, dsts, 0);
                    }
                    let mut registers = vec![self.compile_stdlib_method_receiver(receiver, field)?];
                    registers.extend(self.compile_stdlib_call_args(function, args, 1)?);
                    self.emitter.code.push(Instruction::CallStdlibMulti {
                        function,
                        args: registers,
                        dsts: dsts.to_vec(),
                    });
                    return Ok(());
                }

                let result_count = if let Some((interface_name, interface_type)) =
                    self.interface_type_for_expr(receiver)
                {
                    let Some(method) = interface_type
                        .methods
                        .iter()
                        .find(|method| method.name == *field)
                    else {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` is not part of interface `{interface_name}` in the current subset"
                            ),
                        });
                    };
                    method.result_types.len()
                } else {
                    let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` multi-result call requires a known receiver type"
                            ),
                        });
                    };
                    let Some(result_count) = self
                        .lookup_concrete_method(receiver, field)
                        .map(|method| method.result_types.len())
                    else {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "unknown method `{receiver_type}.{field}` in the current subset"
                            ),
                        });
                    };
                    result_count
                };

                if let Some((_, _)) = self.interface_type_for_expr(receiver) {
                    if result_count != dsts.len() {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` returns {result_count} value(s), not {}",
                                dsts.len()
                            ),
                        });
                    }

                    let is_mutating_read = self.interface_type_for_expr(receiver).is_some_and(
                        |(_, interface_type)| {
                            interface_type.methods.iter().any(|method| {
                                method.name == "Read"
                                    && method.name == *field
                                    && method.params.len() == 1
                                    && method.params[0].typ == "[]byte"
                                    && method.result_types == ["int", "error"]
                            })
                        },
                    );

                    if is_mutating_read {
                        return self
                            .compile_mutating_method_multi_call(receiver, field, args, dsts, 0);
                    }

                    let params = self
                        .interface_type_for_expr(receiver)
                        .and_then(|(_, interface_type)| {
                            interface_type
                                .methods
                                .iter()
                                .find(|method| method.name == *field)
                                .map(|method| method.params.clone())
                        })
                        .unwrap_or_default();
                    let receiver = self.compile_value_expr(receiver)?;
                    let registers = self.compile_method_call_args(&params, args)?;
                    self.emitter.code.push(Instruction::CallMethodMulti {
                        receiver,
                        method: field.clone(),
                        args: registers,
                        dsts: dsts.to_vec(),
                    });
                    return Ok(());
                }

                if result_count != dsts.len() {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "method `{field}` returns {result_count} value(s), not {}",
                            dsts.len()
                        ),
                    });
                }

                let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "method `{field}` multi-result call requires a known concrete receiver type"
                        ),
                    });
                };
                let Some(function) = self.lookup_concrete_method_function(receiver, field) else {
                    return Err(CompileError::Unsupported {
                        detail: self
                            .missing_concrete_method_detail(receiver, field)
                            .unwrap_or_else(|| {
                                format!(
                                    "unknown method `{receiver_type}.{field}` in the current subset"
                                )
                            }),
                    });
                };

                let params = self
                    .lookup_concrete_method(receiver, field)
                    .map(|method| method.params.clone())
                    .unwrap_or_default();
                let receiver = self.compile_method_receiver(receiver, field)?;
                let mut registers = vec![receiver];
                registers.extend(self.compile_method_call_args(&params, args)?);
                self.emitter.code.push(Instruction::CallFunctionMulti {
                    function,
                    args: registers,
                    dsts: dsts.to_vec(),
                });
                Ok(())
            }
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() {
                    if let Some(result_types) = self.expr_function_result_types(callee) {
                        if result_types.len() != dsts.len() {
                            return Err(self.function_value_result_count_error(
                                callee,
                                result_types.len(),
                                dsts.len(),
                            ));
                        }
                        self.validate_function_value_call_arity(callee, args)?;
                        self.validate_function_value_call_types(callee, args)?;
                        let callee_reg = self.compile_value_expr(callee)?;
                        let registers = self.compile_function_value_args(callee, args)?;
                        self.emitter.code.push(Instruction::CallClosureMulti {
                            callee: callee_reg,
                            args: registers,
                            dsts: dsts.to_vec(),
                        });
                        return Ok(());
                    }
                    return Err(self.local_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::MultiCall {
                            expected_results: dsts.len(),
                        },
                    ));
                }
                if self.lookup_global(name).is_some() {
                    if let Some(result_types) = self.expr_function_result_types(callee) {
                        if result_types.len() != dsts.len() {
                            return Err(self.function_value_result_count_error(
                                callee,
                                result_types.len(),
                                dsts.len(),
                            ));
                        }
                        self.validate_function_value_call_arity(callee, args)?;
                        self.validate_function_value_call_types(callee, args)?;
                        let callee_reg = self.compile_value_expr(callee)?;
                        let registers = self.compile_function_value_args(callee, args)?;
                        self.emitter.code.push(Instruction::CallClosureMulti {
                            callee: callee_reg,
                            args: registers,
                            dsts: dsts.to_vec(),
                        });
                        return Ok(());
                    }
                    return Err(self.global_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::MultiCall {
                            expected_results: dsts.len(),
                        },
                    ));
                }
                if name == "copy" || name == "delete" {
                    return Err(CompileError::Unsupported {
                        detail: format!("multi-result builtin call `{name}` is not supported yet"),
                    });
                }
                if name == "panic" {
                    return Err(CompileError::Unsupported {
                        detail: "multi-result builtin call `panic` is not supported".into(),
                    });
                }
                if name == "recover" {
                    return Err(CompileError::Unsupported {
                        detail: "multi-result builtin call `recover` is not supported".into(),
                    });
                }
                if resolve_stdlib_function("builtin", name).is_some() {
                    return Err(CompileError::Unsupported {
                        detail: format!("multi-result builtin call `{name}` is not supported yet"),
                    });
                }

                let function = self
                    .env
                    .function_ids
                    .get(name)
                    .copied()
                    .ok_or_else(|| CompileError::UnknownFunction { name: name.clone() })?;
                self.validate_named_function_call_arity(name, args)?;
                self.validate_named_function_call_types(name, args)?;
                let result_count = self
                    .env
                    .function_result_types
                    .get(name)
                    .map(Vec::len)
                    .unwrap_or(0);
                if result_count != dsts.len() {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "`{name}` returns {result_count} value(s), not {}",
                            dsts.len()
                        ),
                    });
                }

                let registers = self.compile_call_args(name, args)?;
                self.emitter.code.push(Instruction::CallFunctionMulti {
                    function,
                    args: registers,
                    dsts: dsts.to_vec(),
                });
                Ok(())
            }
            _ => {
                let Some(result_types) = self.expr_function_result_types(callee) else {
                    return Err(self.unsupported_call_target_error(
                        callee,
                        CallableContext::MultiCall {
                            expected_results: dsts.len(),
                        },
                    ));
                };
                if result_types.len() != dsts.len() {
                    return Err(self.function_value_result_count_error(
                        callee,
                        result_types.len(),
                        dsts.len(),
                    ));
                }
                self.validate_function_value_call_arity(callee, args)?;
                self.validate_function_value_call_types(callee, args)?;
                let callee_reg = self.compile_value_expr(callee)?;
                let registers = self.compile_function_value_args(callee, args)?;
                self.emitter.code.push(Instruction::CallClosureMulti {
                    callee: callee_reg,
                    args: registers,
                    dsts: dsts.to_vec(),
                });
                Ok(())
            }
        }
    }
}
