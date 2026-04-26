use super::*;
use crate::types::format_function_type;
use gowasm_parser::Parameter;

impl FunctionBuilder<'_> {
    pub(super) fn selector_is_method_expression(&self, receiver: &Expr, field: &str) -> bool {
        self.selector_method_expression_type(receiver, field)
            .is_some()
    }

    pub(super) fn selector_method_expression_type(
        &self,
        receiver: &Expr,
        field: &str,
    ) -> Option<String> {
        let receiver_type = self.selector_method_expression_receiver_type(receiver)?;
        let method = self.method_decl_for_receiver_type(&receiver_type, field)?;
        let mut param_types = vec![receiver_type];
        param_types.extend(method.params.iter().map(|param| param.typ.clone()));
        Some(format_function_type(&param_types, &method.result_types))
    }

    pub(super) fn compile_method_expression_value(
        &mut self,
        dst: usize,
        receiver: &Expr,
        field: &str,
    ) -> Result<(), CompileError> {
        let receiver_type = self
            .selector_method_expression_receiver_type(receiver)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!("unsupported method expression selector for `{field}`"),
            })?;
        let method = self
            .method_decl_for_receiver_type(&receiver_type, field)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "unknown method expression `{receiver_type}.{field}` in the current subset"
                ),
            })?;

        let receiver_name = "__gowasm_method_recv".to_string();
        let mut params = self.wrapper_params(&method.params);
        params.insert(
            0,
            Parameter {
                name: receiver_name.clone(),
                typ: receiver_type,
                variadic: false,
            },
        );

        self.compile_method_wrapper_literal(
            dst,
            &receiver_name,
            field,
            &params[1..],
            &params,
            &method.result_types,
        )
    }

    pub(super) fn compile_interface_method_value(
        &mut self,
        dst: usize,
        receiver: &Expr,
        field: &str,
    ) -> Result<(), CompileError> {
        let (interface_name, interface_type) = self
            .interface_type_for_expr(receiver)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "interface selector `{field}` cannot be used in value position in the current subset"
                ),
            })?;
        let method = interface_type
            .methods
            .iter()
            .find(|method| method.name == *field)
            .cloned()
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "method `{field}` is not part of interface `{interface_name}` in the current subset"
                ),
            })?;
        let receiver_reg = self.compile_value_expr(receiver)?;
        self.compile_bound_method_value(dst, receiver_reg, &interface_name, field, &method)
    }

    pub(super) fn compile_stdlib_method_value(
        &mut self,
        dst: usize,
        receiver: &Expr,
        field: &str,
    ) -> Result<(), CompileError> {
        let (receiver_type, function) = self
            .resolved_stdlib_value_method(receiver, field)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "selector `{field}` cannot be used in value position in the current subset"
                ),
            })?;
        let receiver_reg = self.compile_stdlib_method_receiver(receiver, field)?;
        let hidden_name = format!("__gowasm_stdlib_method_recv${}", self.emitter.next_register);
        self.current_scope_mut()
            .insert(hidden_name.clone(), receiver_reg);
        self.current_type_scope_mut()
            .insert(hidden_name.clone(), receiver_type);

        let param_types = stdlib_function_param_types(function)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!("unsupported stdlib method value `{field}`"),
            })?
            .iter()
            .skip(1)
            .map(|typ| (*typ).to_string())
            .collect::<Vec<_>>();
        let result_types = stdlib_function_result_types(function)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!("unsupported stdlib method value `{field}`"),
            })?
            .iter()
            .map(|typ| (*typ).to_string())
            .collect::<Vec<_>>();
        let params = param_types
            .iter()
            .enumerate()
            .map(|(index, typ)| Parameter {
                name: format!("__gowasm_arg{index}"),
                typ: typ.clone(),
                variadic: false,
            })
            .collect::<Vec<_>>();
        let result = self.compile_method_wrapper_literal(
            dst,
            &hidden_name,
            field,
            &params,
            &params,
            &result_types,
        );

        self.current_scope_mut().remove(&hidden_name);
        self.current_type_scope_mut().remove(&hidden_name);
        result
    }

    pub(super) fn compile_method_expression_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if let Some(result_types) = self.expr_function_result_types(callee) {
            if dst.is_some() && result_types.len() != 1 {
                return Err(self.function_value_result_count_error(callee, result_types.len(), 1));
            }
        }
        self.validate_function_value_call_arity(callee, args)?;
        self.validate_function_value_call_types(callee, args)?;
        let callee_reg = self.compile_value_expr(callee)?;
        let registers = self.compile_function_value_args(callee, args)?;
        self.emitter.code.push(Instruction::CallClosure {
            callee: callee_reg,
            args: registers,
            dst,
        });
        Ok(())
    }

    pub(super) fn compile_method_expression_multi_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        dsts: &[usize],
    ) -> Result<(), CompileError> {
        let Some(result_types) = self.expr_function_result_types(callee) else {
            return Err(CompileError::Unsupported {
                detail: "unsupported method-expression multi-result call target".into(),
            });
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

    pub(super) fn compile_method_expression_go_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<(), CompileError> {
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

    pub(super) fn compile_method_expression_defer_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let callee_reg = self.compile_value_expr(callee)?;
        let mut registers = Vec::with_capacity(args.len());
        for arg in args {
            registers.push(self.compile_value_expr(arg)?);
        }
        self.emitter.code.push(Instruction::DeferClosure {
            callee: callee_reg,
            args: registers,
        });
        Ok(())
    }

    fn compile_bound_method_value(
        &mut self,
        dst: usize,
        receiver_reg: usize,
        receiver_type: &str,
        field: &str,
        method: &InterfaceMethodDecl,
    ) -> Result<(), CompileError> {
        let hidden_name = format!("__gowasm_method_recv${}", self.emitter.next_register);
        self.current_scope_mut()
            .insert(hidden_name.clone(), receiver_reg);
        self.current_type_scope_mut()
            .insert(hidden_name.clone(), receiver_type.to_string());

        let params = self.wrapper_params(&method.params);
        let result = self.compile_method_wrapper_literal(
            dst,
            &hidden_name,
            field,
            &params,
            &params,
            &method.result_types,
        );

        self.current_scope_mut().remove(&hidden_name);
        self.current_type_scope_mut().remove(&hidden_name);
        result
    }

    fn compile_method_wrapper_literal(
        &mut self,
        dst: usize,
        receiver_name: &str,
        field: &str,
        arg_params: &[Parameter],
        params: &[Parameter],
        result_types: &[String],
    ) -> Result<(), CompileError> {
        let call = Expr::Call {
            callee: Box::new(Expr::Selector {
                receiver: Box::new(Expr::Ident(receiver_name.to_string())),
                field: field.to_string(),
            }),
            type_args: Vec::new(),
            args: arg_params
                .iter()
                .map(|param| {
                    if param.variadic {
                        Expr::Spread {
                            expr: Box::new(Expr::Ident(param.name.clone())),
                        }
                    } else {
                        Expr::Ident(param.name.clone())
                    }
                })
                .collect(),
        };
        let body = if result_types.is_empty() {
            vec![Stmt::Expr(call)]
        } else {
            vec![Stmt::Return(vec![call])]
        };
        self.compile_function_literal(dst, params, result_types, &body)
    }

    fn wrapper_params(&self, params: &[Parameter]) -> Vec<Parameter> {
        params
            .iter()
            .enumerate()
            .map(|(index, param)| Parameter {
                name: format!("__gowasm_arg{index}"),
                typ: param.typ.clone(),
                variadic: param.variadic,
            })
            .collect()
    }

    fn method_decl_for_receiver_type(
        &self,
        receiver_type: &str,
        field: &str,
    ) -> Option<InterfaceMethodDecl> {
        if let Some(interface_type) = self.instantiated_interface_type(receiver_type) {
            return interface_type
                .methods
                .iter()
                .find(|method| method.name == *field)
                .cloned();
        }
        self.instantiated_method_set(receiver_type)
            .and_then(|methods| methods.iter().find(|method| method.name == *field).cloned())
    }

    fn selector_method_expression_receiver_type(&self, receiver: &Expr) -> Option<String> {
        self.receiver_type_expr_name(receiver)
    }

    fn receiver_type_expr_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some()
                    || self.scopes.captured_by_ref.contains(name)
                    || self.lookup_global(name).is_some()
                    || self.env.function_ids.contains_key(name)
                    || self.env.imported_packages.contains_key(name)
                {
                    return None;
                }
                (self.instantiated_named_type(name)
                    || self.instantiated_interface_type(name).is_some()
                    || self.instantiated_alias_type(name).is_some())
                .then(|| name.clone())
            }
            Expr::Selector { receiver, field } => {
                let Expr::Ident(package_name) = receiver.as_ref() else {
                    return None;
                };
                self.env
                    .imported_packages
                    .contains_key(package_name)
                    .then(|| {
                        let candidate = format!("{package_name}.{field}");
                        (self.instantiated_named_type(&candidate)
                            || self.instantiated_interface_type(&candidate).is_some()
                            || self.instantiated_alias_type(&candidate).is_some())
                        .then_some(candidate)
                    })?
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => self
                .receiver_type_expr_name(expr)
                .map(|inner| format!("*{inner}")),
            Expr::Index { target, index } => {
                let base = self.receiver_type_expr_name(target)?;
                let type_arg = self.receiver_type_expr_name(index)?;
                let candidate = format!("{base}[{type_arg}]");
                (self.instantiated_named_type(&candidate)
                    || self.instantiated_interface_type(&candidate).is_some()
                    || self.instantiated_alias_type(&candidate).is_some())
                .then_some(candidate)
            }
            _ => None,
        }
    }
}
