use super::diagnostics::CallableContext;
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_defer(&mut self, call: &Expr) -> Result<(), CompileError> {
        let Expr::Call { callee, args, .. } = call else {
            return Err(CompileError::Unsupported {
                detail: "`defer` currently requires a call expression".into(),
            });
        };

        match callee.as_ref() {
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() {
                    if self.expr_function_result_types(callee).is_some() {
                        let callee = self.compile_value_expr(callee)?;
                        let mut registers = Vec::with_capacity(args.len());
                        for arg in args {
                            registers.push(self.compile_value_expr(arg)?);
                        }
                        self.emitter.code.push(Instruction::DeferClosure {
                            callee,
                            args: registers,
                        });
                        return Ok(());
                    }
                    return Err(self.local_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::Defer,
                    ));
                }
                if self.lookup_global(name).is_some() {
                    if self.expr_function_result_types(callee).is_some() {
                        let callee = self.compile_value_expr(callee)?;
                        let mut registers = Vec::with_capacity(args.len());
                        for arg in args {
                            registers.push(self.compile_value_expr(arg)?);
                        }
                        self.emitter.code.push(Instruction::DeferClosure {
                            callee,
                            args: registers,
                        });
                        return Ok(());
                    }
                    return Err(self.global_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::Defer,
                    ));
                }
                let function = self
                    .env
                    .function_ids
                    .get(name)
                    .copied()
                    .ok_or_else(|| CompileError::UnknownFunction { name: name.clone() })?;
                let mut registers = Vec::with_capacity(args.len());
                for arg in args {
                    registers.push(self.compile_value_expr(arg)?);
                }
                self.emitter.code.push(Instruction::DeferFunction {
                    function,
                    args: registers,
                });
                Ok(())
            }
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(package_path) =
                        self.env.imported_packages.get(receiver_name).cloned()
                    {
                        if self.try_compile_imported_selector_defer_call(
                            receiver_name,
                            &package_path,
                            field,
                            args,
                        )? {
                            return Ok(());
                        }
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "unsupported deferred selector call `{receiver_name}.{field}`"
                            ),
                        });
                    }
                }
                if self.selector_is_method_expression(receiver, field) {
                    return self.compile_method_expression_defer_call(callee, args);
                }

                if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                    let mut registers = Vec::with_capacity(args.len() + 1);
                    registers.push(self.compile_stdlib_method_receiver(receiver, field)?);
                    for arg in args {
                        registers.push(self.compile_value_expr(arg)?);
                    }
                    self.emitter.code.push(Instruction::DeferStdlib {
                        function,
                        args: registers,
                    });
                    return Ok(());
                }

                if let Some(interface_type) = self
                    .infer_expr_type_name(receiver)
                    .and_then(|typ| self.instantiated_interface_type(&typ))
                {
                    if !interface_type
                        .methods
                        .iter()
                        .any(|method| method.name == *field)
                    {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` is not part of the receiver interface in the current subset"
                            ),
                        });
                    }
                    let receiver = self.compile_value_expr(receiver)?;
                    let mut registers = Vec::with_capacity(args.len());
                    for arg in args {
                        registers.push(self.compile_value_expr(arg)?);
                    }
                    self.emitter.code.push(Instruction::DeferMethod {
                        receiver,
                        method: field.clone(),
                        args: registers,
                    });
                    return Ok(());
                }

                if let Some(function) = self.lookup_concrete_method_function(receiver, field) {
                    let receiver = self.compile_method_receiver(receiver, field)?;
                    let mut registers = Vec::with_capacity(args.len() + 1);
                    registers.push(receiver);
                    for arg in args {
                        registers.push(self.compile_value_expr(arg)?);
                    }
                    self.emitter.code.push(Instruction::DeferFunction {
                        function,
                        args: registers,
                    });
                    return Ok(());
                }

                let receiver_type =
                    self.infer_expr_type_name(receiver)
                        .ok_or_else(|| CompileError::Unsupported {
                            detail: format!(
                                "deferred method receiver for `{field}` must have a known type in the current subset"
                            ),
                        })?;
                Err(CompileError::Unsupported {
                    detail: self
                        .missing_concrete_method_detail(receiver, field)
                        .unwrap_or_else(|| {
                            format!(
                                "unknown method `{receiver_type}.{field}` in the current subset"
                            )
                        }),
                })
            }
            _ => {
                if self.expr_function_result_types(callee).is_some() {
                    let callee = self.compile_value_expr(callee)?;
                    let mut registers = Vec::with_capacity(args.len());
                    for arg in args {
                        registers.push(self.compile_value_expr(arg)?);
                    }
                    self.emitter.code.push(Instruction::DeferClosure {
                        callee,
                        args: registers,
                    });
                    return Ok(());
                }
                Err(self.unsupported_call_target_error(callee, CallableContext::Defer))
            }
        }
    }
}
