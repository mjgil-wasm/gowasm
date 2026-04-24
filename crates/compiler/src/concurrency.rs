use super::diagnostics::CallableContext;
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn send_channel_element_type(&self, expr: &Expr) -> Result<String, CompileError> {
        let Some(channel_type) = self.infer_expr_type_name(expr) else {
            return Err(self.unsupported_with_active_span(
                "channel send requires a known channel type in the current subset",
            ));
        };
        let Some(channel) = parse_channel_type(&channel_type) else {
            return Err(self.unsupported_with_active_span(format!(
                "cannot send on non-channel type `{channel_type}`"
            )));
        };
        if !channel.direction.accepts_send() {
            return Err(self.unsupported_with_active_span(format!(
                "cannot send on receive-only channel type `{channel_type}`"
            )));
        }
        Ok(channel.element_type.to_string())
    }

    fn receive_channel_element_type(&self, expr: &Expr) -> Result<String, CompileError> {
        let Some(channel_type) = self.infer_expr_type_name(expr) else {
            return Err(self.unsupported_with_active_span(
                "channel receive requires a known channel type in the current subset",
            ));
        };
        let Some(channel) = parse_channel_type(&channel_type) else {
            return Err(self.unsupported_with_active_span(format!(
                "cannot receive from non-channel type `{channel_type}`"
            )));
        };
        if !channel.direction.accepts_recv() {
            return Err(self.unsupported_with_active_span(format!(
                "cannot receive from send-only channel type `{channel_type}`"
            )));
        }
        Ok(channel.element_type.to_string())
    }

    pub(super) fn compile_go_stmt(&mut self, call: &Expr) -> Result<(), CompileError> {
        let Expr::Call { callee, args, .. } = call else {
            return Err(CompileError::Unsupported {
                detail: "`go` currently requires a call expression".into(),
            });
        };

        match callee.as_ref() {
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() || self.lookup_global(name).is_some() {
                    if self.expr_function_result_types(callee).is_some() {
                        self.validate_function_value_call_arity(callee, args)?;
                        self.validate_function_value_call_types(callee, args)?;
                        let callee_reg = self.compile_value_expr(callee)?;
                        let registers = self.compile_function_value_args(callee, args)?;
                        self.emitter.code.push(Instruction::GoCallClosure {
                            callee: callee_reg,
                            args: registers,
                        });
                        return Ok(());
                    }
                    return if self.lookup_local(name).is_some() {
                        Err(self.local_non_callable_call_error(name, callee, CallableContext::Go))
                    } else {
                        Err(self.global_non_callable_call_error(name, callee, CallableContext::Go))
                    };
                }
                if resolve_stdlib_function("builtin", name).is_some() {
                    return Err(CompileError::Unsupported {
                        detail: format!("builtin goroutine call `{name}` is not supported yet"),
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
                let registers = self.compile_call_args(name, args)?;
                self.emitter.code.push(Instruction::GoCall {
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
                        if self.try_compile_imported_selector_go_call(
                            receiver_name,
                            &package_path,
                            field,
                            args,
                        )? {
                            return Ok(());
                        }
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "unsupported goroutine selector call `{receiver_name}.{field}`"
                            ),
                        });
                    }
                }
                if self.selector_is_method_expression(receiver, field) {
                    return self.compile_method_expression_go_call(callee, args);
                }
                self.validate_selector_call_arity(receiver, field, args)?;
                self.validate_selector_call_types(receiver, field, args)?;
                if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                    let mut registers = vec![self.compile_stdlib_method_receiver(receiver, field)?];
                    registers.extend(self.compile_stdlib_call_args(function, args, 1)?);
                    self.emitter.code.push(Instruction::GoCallStdlib {
                        function,
                        args: registers,
                    });
                    return Ok(());
                }
                if let Some((_, interface_type)) = self.interface_type_for_expr(receiver) {
                    let receiver = self.compile_value_expr(receiver)?;
                    let params = interface_type
                        .methods
                        .iter()
                        .find(|method| method.name == *field)
                        .map(|method| method.params.clone())
                        .unwrap_or_default();
                    let registers = self.compile_method_call_args(&params, args)?;
                    self.emitter.code.push(Instruction::GoCallMethod {
                        receiver,
                        method: field.clone(),
                        args: registers,
                    });
                    return Ok(());
                }
                let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "goroutine method `{field}` requires a known concrete receiver type"
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
                self.emitter.code.push(Instruction::GoCall {
                    function,
                    args: registers,
                });
                Ok(())
            }
            _ => {
                let Some(_result_types) = self.expr_function_result_types(callee) else {
                    return Err(self.unsupported_call_target_error(callee, CallableContext::Go));
                };
                self.validate_function_value_call_arity(callee, args)?;
                self.validate_function_value_call_types(callee, args)?;
                let callee_reg = self.compile_value_expr(callee)?;
                let registers = self.compile_function_value_args(callee, args)?;
                self.emitter.code.push(Instruction::GoCallClosure {
                    callee: callee_reg,
                    args: registers,
                });
                Ok(())
            }
        }
    }

    pub(super) fn compile_send_stmt(
        &mut self,
        chan: &Expr,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let element_type = self.send_channel_element_type(chan)?;
        self.validate_assignable_type(Some(&element_type), value)?;
        let chan = self.compile_value_expr(chan)?;
        let value = self.compile_typed_value_expr(value, Some(&element_type))?;
        self.emitter
            .code
            .push(Instruction::ChanSend { chan, value });
        Ok(())
    }

    pub(super) fn compile_receive_expr(
        &mut self,
        dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let _ = self.receive_channel_element_type(expr)?;
        let chan = self.compile_value_expr(expr)?;
        self.emitter.code.push(Instruction::ChanRecv { dst, chan });
        Ok(())
    }

    pub(super) fn compile_receive_ok_expr(
        &mut self,
        value_dst: usize,
        ok_dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let _ = self.receive_channel_element_type(expr)?;
        let chan = self.compile_value_expr(expr)?;
        self.emitter.code.push(Instruction::ChanRecvOk {
            value_dst,
            ok_dst,
            chan,
        });
        Ok(())
    }

    pub(super) fn channel_element_type(&self, expr: &Expr) -> Result<String, CompileError> {
        self.receive_channel_element_type(expr)
    }

    pub(super) fn compile_try_receive_expr(
        &mut self,
        ready_dst: usize,
        value_dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let _ = self.channel_element_type(expr)?;
        let chan = self.compile_value_expr(expr)?;
        self.emitter.code.push(Instruction::ChanTryRecv {
            ready_dst,
            value_dst,
            chan,
        });
        Ok(())
    }

    pub(super) fn compile_try_receive_ok_expr(
        &mut self,
        ready_dst: usize,
        value_dst: usize,
        ok_dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let _ = self.channel_element_type(expr)?;
        let chan = self.compile_value_expr(expr)?;
        self.emitter.code.push(Instruction::ChanTryRecvOk {
            ready_dst,
            value_dst,
            ok_dst,
            chan,
        });
        Ok(())
    }

    pub(super) fn compile_try_send_stmt(
        &mut self,
        ready_dst: usize,
        chan: &Expr,
        value: &Expr,
    ) -> Result<(), CompileError> {
        let element_type = self.send_channel_element_type(chan)?;
        self.validate_assignable_type(Some(&element_type), value)?;
        let chan = self.compile_value_expr(chan)?;
        let value = self.compile_value_expr(value)?;
        self.emitter.code.push(Instruction::ChanTrySend {
            ready_dst,
            chan,
            value,
        });
        Ok(())
    }
}
