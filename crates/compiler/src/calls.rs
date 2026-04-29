use super::diagnostics::CallableContext;
use super::*;
use crate::typed_lowering::predeclared_conversion_target;
use gowasm_vm::PromotedFieldAccess;

impl FunctionBuilder<'_> {
    pub(super) fn lookup_stdlib_value_method(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> Option<StdlibFunctionId> {
        self.resolved_stdlib_value_method(receiver, method)
            .map(|(_, function)| function)
    }

    pub(super) fn resolved_stdlib_value_method(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> Option<(String, StdlibFunctionId)> {
        let receiver_type = self.infer_expr_type_name(receiver)?;
        if let Some(function) = resolve_stdlib_method(&receiver_type, method) {
            return Some((receiver_type, function));
        }
        if let Some(inner) = parse_pointer_type(&receiver_type) {
            if let Some(function) = resolve_stdlib_method(inner, method) {
                return Some((inner.to_string(), function));
            }
        }
        if !self.stdlib_method_uses_implicit_address_of(receiver, method) {
            return None;
        }
        let pointer_type = format!("*{receiver_type}");
        resolve_stdlib_method(&pointer_type, method).map(|function| (pointer_type, function))
    }

    pub(super) fn lookup_concrete_method_function(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> Option<usize> {
        let receiver_type = self.concrete_method_receiver_type(receiver, method)?;
        self.instantiated_method_function_id(&format!("{receiver_type}.{method}"))
    }

    pub(super) fn lookup_concrete_method(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> Option<InterfaceMethodDecl> {
        let receiver_type = self.concrete_method_receiver_type(receiver, method)?;
        self.instantiated_method_set(&receiver_type)
            .and_then(|methods| {
                methods
                    .into_iter()
                    .find(|candidate| candidate.name == method)
            })
    }

    fn field_has_function_type(&self, receiver_type: &str, field: &str) -> bool {
        self.resolve_field_selector(receiver_type, field)
            .ok()
            .flatten()
            .is_some_and(|field| parse_function_type(&field.typ).is_some())
    }

    pub(super) fn compile_method_receiver(
        &mut self,
        receiver: &Expr,
        method: &str,
    ) -> Result<usize, CompileError> {
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return self.compile_value_expr(receiver);
        };
        let Some(resolved_receiver_type) = self.concrete_method_receiver_type(receiver, method)
        else {
            return self.compile_value_expr(receiver);
        };
        if let Some(binding) = self
            .instantiated_promoted_method_binding(&format!("{resolved_receiver_type}.{method}"))
            .as_ref()
        {
            return self.compile_promoted_method_receiver(
                receiver,
                &receiver_type,
                &resolved_receiver_type,
                binding,
            );
        }
        if resolved_receiver_type == receiver_type {
            return self.compile_value_expr(receiver);
        }
        if parse_pointer_type(&receiver_type).is_some_and(|inner| inner == resolved_receiver_type) {
            let pointer = self.compile_value_expr(receiver)?;
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: target,
                src: pointer,
            });
            return Ok(target);
        }
        if resolved_receiver_type == format!("*{receiver_type}") {
            return self.compile_addressable_expr(receiver);
        }
        self.compile_value_expr(receiver)
    }

    pub(super) fn compile_stdlib_method_receiver(
        &mut self,
        receiver: &Expr,
        method: &str,
    ) -> Result<usize, CompileError> {
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return self.compile_value_expr(receiver);
        };
        if resolve_stdlib_method(&receiver_type, method).is_some() {
            return self.compile_value_expr(receiver);
        }
        if parse_pointer_type(&receiver_type)
            .and_then(|inner| resolve_stdlib_method(inner, method))
            .is_some()
        {
            let pointer = self.compile_value_expr(receiver)?;
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: target,
                src: pointer,
            });
            return Ok(target);
        }
        if self.stdlib_method_uses_implicit_address_of(receiver, method) {
            return self.compile_addressable_expr(receiver);
        }
        self.compile_value_expr(receiver)
    }

    pub(super) fn receiver_uses_implicit_address_of(&self, receiver: &Expr, method: &str) -> bool {
        if !self.receiver_is_addressable(receiver) {
            return false;
        }
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return false;
        };
        self.instantiated_method_set(&format!("*{receiver_type}"))
            .and_then(|methods| {
                methods
                    .into_iter()
                    .find(|candidate| candidate.name == method)
            })
            .is_some()
    }

    fn stdlib_method_uses_implicit_address_of(&self, receiver: &Expr, method: &str) -> bool {
        if !self.receiver_is_addressable(receiver) {
            return false;
        }
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return false;
        };
        resolve_stdlib_method(&format!("*{receiver_type}"), method).is_some()
    }

    fn receiver_is_addressable(&self, receiver: &Expr) -> bool {
        self.expr_is_addressable(receiver)
    }

    pub(super) fn missing_concrete_method_detail(
        &self,
        receiver: &Expr,
        field: &str,
    ) -> Option<String> {
        if let Some(detail) = self.ambiguous_method_selector_detail(receiver, field) {
            return Some(detail);
        }
        let receiver_type = self.infer_expr_type_name(receiver)?;
        if !self.receiver_is_addressable(receiver) {
            let pointer_type = format!("*{receiver_type}");
            if self
                .instantiated_method_function_id(&format!("{pointer_type}.{field}"))
                .is_some()
            {
                return Some(format!(
                    "method `{field}` requires an addressable receiver in the current subset"
                ));
            }
        }
        Some(format!(
            "unknown method `{receiver_type}.{field}` in the current subset"
        ))
    }

    fn concrete_method_receiver_type(&self, receiver: &Expr, method: &str) -> Option<String> {
        let receiver_type = self.infer_expr_type_name(receiver)?;
        if self
            .instantiated_method_function_id(&format!("{receiver_type}.{method}"))
            .is_some()
        {
            return Some(receiver_type);
        }
        if self.receiver_uses_implicit_address_of(receiver, method) {
            let pointer_type = format!("*{receiver_type}");
            if self
                .instantiated_method_function_id(&format!("{pointer_type}.{method}"))
                .is_some()
            {
                return Some(pointer_type);
            }
        }
        let inner = parse_pointer_type(&receiver_type)?;
        self.instantiated_method_function_id(&format!("{inner}.{method}"))
            .is_some()
            .then(|| inner.to_string())
    }

    pub(super) fn compile_addressable_expr(
        &mut self,
        receiver: &Expr,
    ) -> Result<usize, CompileError> {
        self.compile_value_expr(&Expr::Unary {
            op: UnaryOp::AddressOf,
            expr: Box::new(receiver.clone()),
        })
    }

    fn compile_promoted_method_receiver(
        &mut self,
        receiver: &Expr,
        receiver_type: &str,
        resolved_receiver_type: &str,
        binding: &symbols::PromotedMethodBindingInfo,
    ) -> Result<usize, CompileError> {
        let (mut current, mut current_type) = if resolved_receiver_type == receiver_type {
            (
                self.compile_value_expr(receiver)?,
                receiver_type.to_string(),
            )
        } else if resolved_receiver_type == format!("*{receiver_type}") {
            (
                self.compile_addressable_expr(receiver)?,
                resolved_receiver_type.to_string(),
            )
        } else {
            (
                self.compile_value_expr(receiver)?,
                receiver_type.to_string(),
            )
        };

        for (index, step) in binding.path.iter().enumerate() {
            let last = index + 1 == binding.path.len();
            match step.access {
                PromotedFieldAccess::Value => {
                    if let Some(inner) = parse_pointer_type(&current_type) {
                        let dereferenced = self.alloc_register();
                        self.emitter.code.push(Instruction::Deref {
                            dst: dereferenced,
                            src: current,
                        });
                        current = dereferenced;
                        current_type = inner.to_string();
                    }
                    let field_type = self.struct_field_type(&current_type, &step.field)?;
                    let field_value = self.alloc_register();
                    self.emitter.code.push(Instruction::GetField {
                        dst: field_value,
                        target: current,
                        field: step.field.clone(),
                    });
                    current = field_value;
                    current_type = field_type;
                }
                PromotedFieldAccess::Pointer => {
                    let Some(inner) = parse_pointer_type(&current_type) else {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "promoted method `{resolved_receiver_type}` requires an addressable embedded receiver in the current subset"
                            ),
                        });
                    };
                    let field_type = self.struct_field_type(inner, &step.field)?;
                    let field_pointer = self.alloc_register();
                    self.emitter.code.push(Instruction::ProjectFieldPointer {
                        dst: field_pointer,
                        src: current,
                        field: step.field.clone(),
                        typ: if last {
                            self.pointer_runtime_type(&field_type)
                        } else {
                            TYPE_POINTER
                        },
                    });
                    current = field_pointer;
                    current_type = format!("*{field_type}");
                }
            }
        }

        if current_type == binding.target_receiver_type {
            return Ok(current);
        }
        if parse_pointer_type(&current_type)
            .is_some_and(|inner| inner == binding.target_receiver_type)
        {
            let dereferenced = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: dereferenced,
                src: current,
            });
            return Ok(dereferenced);
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "promoted method `{resolved_receiver_type}` resolves to unsupported receiver path `{}` in the current subset",
                binding.target_receiver_type
            ),
        })
    }

    fn struct_field_type(
        &self,
        struct_type_name: &str,
        field: &str,
    ) -> Result<String, CompileError> {
        let Some(struct_type) = self.instantiated_struct_type(struct_type_name) else {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "unknown promoted field `{struct_type_name}.{field}` in the current subset"
                ),
            });
        };
        struct_type
            .fields
            .iter()
            .find(|candidate| candidate.name == field)
            .map(|candidate| candidate.typ.clone())
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "unknown promoted field `{struct_type_name}.{field}` in the current subset"
                ),
            })
    }

    pub(super) fn compile_expr_stmt(&mut self, expr: &Expr) -> Result<(), CompileError> {
        if let Expr::Call {
            callee,
            type_args,
            args,
        } = expr
        {
            return self.compile_call(callee, type_args, args, None);
        }

        if let Expr::Unary {
            op: UnaryOp::Receive,
            expr: channel_expr,
        } = expr
        {
            let dst = self.alloc_register();
            return self.compile_receive_expr(dst, channel_expr);
        }

        Err(CompileError::Unsupported {
            detail: "only call expressions are supported in statement position".into(),
        })
    }

    pub(super) fn compile_call(
        &mut self,
        callee: &Expr,
        type_args: &[String],
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        self.ensure_expr_runtime_types(callee)?;
        for arg in args {
            self.ensure_expr_runtime_types(arg)?;
        }
        if let Some(generic_call) = self.resolve_generic_call(callee, type_args, args) {
            let generic_call = generic_call?;
            return self.compile_generic_call(&generic_call, args, dst);
        }
        match callee {
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(alias_name) =
                        self.imported_selector_alias_name(receiver_name, field)
                    {
                        return self.compile_alias_conversion_call(&alias_name, args, dst);
                    }
                    if let Some(package_path) =
                        self.env.imported_packages.get(receiver_name).cloned()
                    {
                        if self.try_compile_imported_selector_call(
                            receiver_name,
                            &package_path,
                            field,
                            args,
                            dst,
                        )? {
                            return Ok(());
                        }
                        return Err(CompileError::Unsupported {
                            detail: format!("unsupported selector call `{receiver_name}.{field}`"),
                        });
                    }
                }
                if self.selector_is_method_expression(receiver, field) {
                    return self.compile_method_expression_call(callee, args, dst);
                }

                self.validate_selector_call_arity(receiver, field, args)?;
                self.validate_selector_call_types(receiver, field, args)?;

                if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                    let result_count = stdlib_function_result_count(function);
                    if dst.is_some() && result_count != 1 {
                        return Err(CompileError::Unsupported {
                            detail: format!("method `{field}` cannot be used in value position"),
                        });
                    }
                    if stdlib_function_mutates_first_arg(function) {
                        return self.compile_mutating_stdlib_method_call(
                            receiver, field, function, args, dst,
                        );
                    }
                    let mut registers = vec![self.compile_stdlib_method_receiver(receiver, field)?];
                    registers.extend(self.compile_stdlib_call_args(function, args, 1)?);
                    self.emitter.code.push(Instruction::CallStdlib {
                        function,
                        args: registers,
                        dst,
                    });
                    return Ok(());
                }

                if let Some((interface_name, interface_type)) =
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
                    if dst.is_some() {
                        let result_count = method.result_types.len();
                        if result_count != 1 {
                            return Err(CompileError::Unsupported {
                                detail: format!(
                                    "method `{field}` returns {result_count} value(s) and cannot be used in single-value position"
                                ),
                            });
                        }
                    }
                    let receiver = self.compile_value_expr(receiver)?;
                    let registers = self.compile_method_call_args(&method.params, args)?;
                    self.emitter.code.push(Instruction::CallMethod {
                        receiver,
                        method: field.clone(),
                        args: registers,
                        dst,
                    });
                    return Ok(());
                }

                if dst.is_some() {
                    let Some(_receiver_type) = self.infer_expr_type_name(receiver) else {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "method `{field}` cannot be used in single-value position without a known receiver type"
                            ),
                        });
                    };
                    if let Some(result_count) = self
                        .lookup_concrete_method(receiver, field)
                        .map(|method| method.result_types.len())
                    {
                        if result_count != 1 {
                            return Err(CompileError::Unsupported {
                                detail: format!(
                                    "method `{field}` returns {result_count} value(s) and cannot be used in single-value position"
                                ),
                            });
                        }
                    }
                }

                let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "method `{field}` requires a known concrete receiver type in the current subset"
                        ),
                    });
                };
                if let Some(function) = self.lookup_concrete_method_function(receiver, field) {
                    let params = self
                        .lookup_concrete_method(receiver, field)
                        .map(|method| method.params.clone())
                        .unwrap_or_default();
                    let receiver = self.compile_method_receiver(receiver, field)?;
                    let mut registers = vec![receiver];
                    registers.extend(self.compile_method_call_args(&params, args)?);
                    self.emitter.code.push(Instruction::CallFunction {
                        function,
                        args: registers,
                        dst,
                    });
                    return Ok(());
                }

                if self.field_has_function_type(&receiver_type, field) {
                    let callee_reg = self.alloc_register();
                    self.compile_selector_expr(callee_reg, receiver, field)?;
                    let registers = self.compile_function_value_args(callee, args)?;
                    self.emitter.code.push(Instruction::CallClosure {
                        callee: callee_reg,
                        args: registers,
                        dst,
                    });
                    return Ok(());
                }

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
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() {
                    if let Some(result_types) = self.expr_function_result_types(callee) {
                        if dst.is_some() && result_types.len() != 1 {
                            return Err(self.function_value_result_count_error(
                                callee,
                                result_types.len(),
                                1,
                            ));
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
                        return Ok(());
                    }
                    return Err(self.local_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::PlainCall,
                    ));
                }
                if self.lookup_global(name).is_some() {
                    if let Some(result_types) = self.expr_function_result_types(callee) {
                        if dst.is_some() && result_types.len() != 1 {
                            return Err(self.function_value_result_count_error(
                                callee,
                                result_types.len(),
                                1,
                            ));
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
                        return Ok(());
                    }
                    return Err(self.global_non_callable_call_error(
                        name,
                        callee,
                        CallableContext::PlainCall,
                    ));
                }
                if predeclared_conversion_target(name.as_str()).is_some() {
                    return self.compile_named_conversion_call(name, args, dst);
                }
                if self.instantiated_alias_type(name.as_str()).is_some() {
                    return self.compile_named_conversion_call(name, args, dst);
                }
                if name == "copy" {
                    return self.compile_copy_call(args, dst);
                }
                if name == "delete" {
                    return self.compile_delete_call(args, dst);
                }
                if name == "clear" {
                    return self.compile_clear_call(args, dst);
                }
                if name == "close" {
                    self.validate_builtin_call_arity(name, args)?;
                    self.validate_builtin_call_types(name, args)?;
                    return self.compile_close_call(args, dst);
                }
                if name == "panic" {
                    if dst.is_some() {
                        return Err(CompileError::Unsupported {
                            detail: "`panic` cannot be used in value position".into(),
                        });
                    }
                    let [arg] = args else {
                        return Err(CompileError::Unsupported {
                            detail: "`panic` currently requires exactly one argument".into(),
                        });
                    };
                    let src = self.compile_value_expr(arg)?;
                    self.emitter.code.push(Instruction::Panic { src });
                    return Ok(());
                }
                if name == "recover" {
                    if !args.is_empty() {
                        return Err(CompileError::Unsupported {
                            detail: "`recover` currently requires no arguments".into(),
                        });
                    }
                    let dst = dst.unwrap_or_else(|| self.alloc_register());
                    self.emitter.code.push(Instruction::Recover { dst });
                    return Ok(());
                }
                if name == "append" && args.len() == 2 {
                    if let Expr::Spread { expr } = &args[1] {
                        let function = resolve_stdlib_function("builtin", "__append_spread")
                            .expect("__append_spread should be registered");
                        let target = self.compile_value_expr(&args[0])?;
                        let source = self.compile_value_expr(expr)?;
                        self.emitter.code.push(Instruction::CallStdlib {
                            function,
                            args: vec![target, source],
                            dst,
                        });
                        return Ok(());
                    }
                }
                if let Some(function) = resolve_stdlib_function("builtin", name) {
                    if dst.is_some() && !stdlib_function_returns_value(function) {
                        return Err(CompileError::Unsupported {
                            detail: format!("`{name}` cannot be used in value position"),
                        });
                    }
                    self.validate_builtin_call_arity(name, args)?;
                    self.validate_builtin_call_types(name, args)?;
                    let mut registers = Vec::with_capacity(args.len());
                    for arg in args {
                        registers.push(self.compile_value_expr(arg)?);
                    }
                    self.emitter.code.push(Instruction::CallStdlib {
                        function,
                        args: registers,
                        dst,
                    });
                    return Ok(());
                }
                let function = self
                    .env
                    .function_ids
                    .get(name)
                    .copied()
                    .ok_or_else(|| CompileError::UnknownFunction { name: name.clone() })?;
                self.validate_named_function_call_arity(name, args)?;
                self.validate_named_function_call_types(name, args)?;
                if dst.is_some() {
                    let result_count = self
                        .env
                        .function_result_types
                        .get(name)
                        .map(Vec::len)
                        .unwrap_or(0);
                    if result_count != 1 {
                        return Err(CompileError::Unsupported {
                            detail: format!(
                                "`{name}` returns {result_count} value(s) and cannot be used in single-value position"
                            ),
                        });
                    }
                }
                let registers = self.compile_call_args(name, args)?;
                self.emitter.code.push(Instruction::CallFunction {
                    function,
                    args: registers,
                    dst,
                });
                Ok(())
            }
            _ => {
                let Some(result_types) = self.expr_function_result_types(callee) else {
                    return Err(
                        self.unsupported_call_target_error(callee, CallableContext::PlainCall)
                    );
                };
                if dst.is_some() && result_types.len() != 1 {
                    return Err(self.function_value_result_count_error(
                        callee,
                        result_types.len(),
                        1,
                    ));
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
        }
    }

    pub(super) fn interface_type_for_expr(
        &self,
        expr: &Expr,
    ) -> Option<(String, InterfaceTypeDef)> {
        let type_name = self.infer_expr_type_name(expr)?;
        self.instantiated_interface_type(&type_name)
            .map(|interface| (type_name, interface))
    }

    pub(super) fn maybe_apply_stringer(
        &mut self,
        expr: &Expr,
        compiled_reg: usize,
    ) -> Result<Option<usize>, CompileError> {
        let Some(type_name) = self.infer_expr_type_name(expr) else {
            return Ok(None);
        };
        let has_stringer = self
            .instantiated_method_set(&type_name)
            .map(|methods| {
                methods.iter().any(|m| {
                    m.name == "String" && m.params.is_empty() && m.result_types == ["string"]
                })
            })
            .unwrap_or(false);
        if !has_stringer {
            return Ok(None);
        }
        let dst = self.alloc_register();
        self.emitter.code.push(Instruction::CallMethod {
            receiver: compiled_reg,
            method: "String".into(),
            args: vec![],
            dst: Some(dst),
        });
        Ok(Some(dst))
    }
}
