use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_new_expr(
        &mut self,
        dst: usize,
        type_name: &str,
    ) -> Result<(), CompileError> {
        let zero = self.alloc_register();
        self.compile_zero_value(zero, type_name)?;
        let typ = self.pointer_runtime_type(type_name);
        self.emitter.code.push(Instruction::BoxHeap {
            dst,
            src: zero,
            typ,
        });
        Ok(())
    }

    pub(super) fn compile_make_expr(
        &mut self,
        dst: usize,
        type_name: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        if self.lookup_local("make").is_some() {
            return Err(CompileError::Unsupported {
                detail: "calling local variable `make` is not supported".into(),
            });
        }
        if self.lookup_global("make").is_some() {
            return Err(CompileError::Unsupported {
                detail: "calling package variable `make` is not supported".into(),
            });
        }

        if let Some(alias) = self.instantiated_alias_type(type_name) {
            self.compile_make_expr(dst, &alias.underlying, args)?;
            self.emitter.code.push(Instruction::Retag {
                dst,
                src: dst,
                typ: alias.type_id,
            });
            return Ok(());
        }

        if let Some((_key_type, value_type)) = parse_map_type(type_name) {
            if args.len() > 1 {
                return Err(CompileError::Unsupported {
                    detail: "`make(map[...], hint)` supports at most one size hint in the current subset".into(),
                });
            }
            if let Some(size_hint) = args.first() {
                let _ = self.compile_value_expr(size_hint)?;
            }
            let zero = self.alloc_register();
            self.compile_zero_value(zero, value_type)?;
            self.emitter.code.push(Instruction::MakeMap {
                dst,
                concrete_type: Some(self.lower_runtime_concrete_type(type_name)?),
                entries: vec![],
                zero,
            });
            return Ok(());
        }

        if let Some(element_type) = type_name.strip_prefix("[]") {
            if !(1..=2).contains(&args.len()) {
                return Err(CompileError::Unsupported {
                    detail:
                        "`make([]T, len)` and `make([]T, len, cap)` are the supported slice make forms in the current subset"
                            .into(),
                });
            }
            self.validate_slice_make_constant_bounds(args)?;
            let len = self.compile_value_expr(&args[0])?;
            let cap = if args.len() == 2 {
                Some(self.compile_value_expr(&args[1])?)
            } else {
                None
            };
            let zero = self.alloc_register();
            self.compile_zero_value(zero, element_type)?;
            let function = resolve_stdlib_function("builtin", "__make_slice").ok_or_else(|| {
                CompileError::Unsupported {
                    detail: "`make([]T, len[, cap])` is not registered in the current runtime"
                        .into(),
                }
            })?;
            let mut builtin_args = vec![len];
            if let Some(cap) = cap {
                builtin_args.push(cap);
            }
            builtin_args.push(zero);
            self.emitter.code.push(Instruction::CallStdlib {
                function,
                args: builtin_args,
                dst: Some(dst),
            });
            return Ok(());
        }

        if let Some(channel_type) = parse_channel_type(type_name) {
            if args.len() > 1 {
                return Err(CompileError::Unsupported {
                    detail: "`make(chan T)` and `make(chan T, cap)` are the supported channel make forms in the current subset"
                        .into(),
                });
            }
            let zero = self.alloc_register();
            self.compile_zero_value(zero, channel_type.element_type)?;
            let cap = if let Some(cap) = args.first() {
                Some(self.compile_value_expr(cap)?)
            } else {
                None
            };
            self.emitter.code.push(Instruction::MakeChannel {
                dst,
                concrete_type: Some(self.lower_runtime_concrete_type(type_name)?),
                cap,
                zero,
            });
            return Ok(());
        }

        Err(CompileError::Unsupported {
            detail: format!("unsupported make target type `{type_name}`"),
        })
    }

    fn validate_slice_make_constant_bounds(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(len) = self.known_int_constant_expr(&args[0]) else {
            return Ok(());
        };
        if len < 0 {
            return Err(CompileError::Unsupported {
                detail: format!("`make` length {len} must not be negative"),
            });
        }

        let Some(cap_expr) = args.get(1) else {
            return Ok(());
        };
        let Some(cap) = self.known_int_constant_expr(cap_expr) else {
            return Ok(());
        };
        if cap < 0 {
            return Err(CompileError::Unsupported {
                detail: format!("`make` capacity {cap} must not be negative"),
            });
        }
        if cap < len {
            return Err(CompileError::Unsupported {
                detail: format!("`make` capacity {cap} must be >= length {len}"),
            });
        }
        Ok(())
    }

    fn known_int_constant_expr(&self, expr: &Expr) -> Option<i64> {
        self.known_int_const_value(expr)
    }

    pub(super) fn compile_clear_call(
        &mut self,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if dst.is_some() {
            return Err(CompileError::Unsupported {
                detail: "`clear` cannot be used in value position".into(),
            });
        }
        if args.len() != 1 {
            return Err(CompileError::Unsupported {
                detail: "`clear` expects exactly one argument".into(),
            });
        }

        let target_name = match &args[0] {
            Expr::Ident(name) => name,
            _ => {
                return Err(CompileError::Unsupported {
                    detail: "`clear` currently requires a local or package-level identifier as its first argument".into(),
                });
            }
        };

        let (target, global) = if let Some(target) = self.lookup_local(target_name) {
            (target, None)
        } else if let Some(global) = self.lookup_global(target_name) {
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::LoadGlobal {
                dst: target,
                global,
            });
            (target, Some(global))
        } else {
            return Err(CompileError::UnknownIdentifier {
                name: target_name.clone(),
            });
        };

        let function = resolve_stdlib_function("builtin", "clear").ok_or_else(|| {
            CompileError::Unsupported {
                detail: "`clear` is not registered in the current runtime".into(),
            }
        })?;
        self.emitter.code.push(Instruction::CallStdlib {
            function,
            args: vec![target],
            dst: Some(target),
        });
        if let Some(global) = global {
            self.emitter.code.push(Instruction::StoreGlobal {
                global,
                src: target,
            });
        }
        Ok(())
    }

    pub(super) fn compile_delete_call(
        &mut self,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if dst.is_some() {
            return Err(CompileError::Unsupported {
                detail: "`delete` cannot be used in value position".into(),
            });
        }
        if args.len() != 2 {
            return Err(CompileError::Unsupported {
                detail: "`delete` expects exactly two arguments in the current subset".into(),
            });
        }

        let mut pointer_target = None;
        let pointer_reg = self.alloc_register();
        let pointer_result = self.compile_expr_into(
            pointer_reg,
            &Expr::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(args[0].clone()),
            },
        );
        match pointer_result {
            Ok(()) => pointer_target = Some(pointer_reg),
            Err(CompileError::Unsupported { .. }) => {}
            Err(err) => return Err(err),
        }

        let target = if let Some(pointer_target) = pointer_target {
            let value_reg = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: value_reg,
                src: pointer_target,
            });
            value_reg
        } else {
            return Err(CompileError::Unsupported {
                detail: "`delete` currently requires an addressable map expression as its first argument".into(),
            });
        };

        let function = resolve_stdlib_function("builtin", "delete").ok_or_else(|| {
            CompileError::Unsupported {
                detail: "`delete` is not registered in the current runtime".into(),
            }
        })?;
        let key = self.compile_value_expr(&args[1])?;
        self.emitter.code.push(Instruction::CallStdlib {
            function,
            args: vec![target, key],
            dst: Some(target),
        });
        if let Some(pointer_target) = pointer_target {
            self.emitter.code.push(Instruction::StoreIndirect {
                target: pointer_target,
                src: target,
            });
        }
        Ok(())
    }

    pub(super) fn compile_copy_call(
        &mut self,
        args: &[Expr],
        count_dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if args.len() != 2 {
            return Err(CompileError::Unsupported {
                detail: "`copy` expects exactly two arguments in the current subset".into(),
            });
        }

        let target = self.compile_value_expr(&args[0])?;
        let src = self.compile_value_expr(&args[1])?;
        self.emitter.code.push(Instruction::Copy {
            target,
            src,
            count_dst,
        });
        Ok(())
    }

    pub(super) fn compile_close_call(
        &mut self,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if dst.is_some() {
            return Err(CompileError::Unsupported {
                detail: "`close` cannot be used in value position".into(),
            });
        }
        if args.len() != 1 {
            return Err(CompileError::Unsupported {
                detail: "`close` expects exactly one argument in the current subset".into(),
            });
        }

        let Some(channel_type) = self.infer_expr_type_name(&args[0]) else {
            return Err(CompileError::Unsupported {
                detail: "close requires a known channel type in the current subset".into(),
            });
        };
        let underlying_type = self.instantiated_underlying_type_name(&channel_type);
        let Some(channel) = parse_channel_type(&underlying_type) else {
            return Err(CompileError::Unsupported {
                detail: format!("cannot close non-channel type `{channel_type}`"),
            });
        };
        if !channel.direction.accepts_send() {
            return Err(CompileError::Unsupported {
                detail: format!("cannot close receive-only channel type `{channel_type}`"),
            });
        }

        let chan = self.compile_value_expr(&args[0])?;
        self.emitter.code.push(Instruction::CloseChannel { chan });
        Ok(())
    }

    pub(super) fn compile_mutating_stdlib_call(
        &mut self,
        receiver_name: &str,
        field: &str,
        function: gowasm_vm::StdlibFunctionId,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if dst.is_some() {
            return Err(CompileError::Unsupported {
                detail: format!("`{receiver_name}.{field}` cannot be used in value position"),
            });
        }
        let Some(target_name) = args.first().and_then(|arg| match arg {
            Expr::Ident(name) => Some(name.as_str()),
            _ => None,
        }) else {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{receiver_name}.{field}` requires an identifier as its first argument in the current subset"
                ),
            });
        };

        let (target, global, is_by_ref) = if let Some(target) = self.lookup_local(target_name) {
            let is_by_ref = self.scopes.captured_by_ref.contains(target_name);
            (target, None, is_by_ref)
        } else if let Some(global) = self.lookup_global(target_name) {
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::LoadGlobal {
                dst: target,
                global,
            });
            (target, Some(global), false)
        } else {
            return Err(CompileError::UnknownIdentifier {
                name: target_name.to_string(),
            });
        };

        let value_reg = if is_by_ref {
            let dereffed = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: dereffed,
                src: target,
            });
            dereffed
        } else {
            target
        };

        let mut registers = vec![value_reg];
        for arg in &args[1..] {
            registers.push(self.compile_value_expr(arg)?);
        }
        self.emitter.code.push(Instruction::CallStdlib {
            function,
            args: registers,
            dst: Some(value_reg),
        });
        if is_by_ref {
            self.emitter.code.push(Instruction::StoreIndirect {
                target,
                src: value_reg,
            });
        }
        if let Some(global) = global {
            self.emitter.code.push(Instruction::StoreGlobal {
                global,
                src: value_reg,
            });
        }
        Ok(())
    }

    pub(super) fn compile_mutating_method_multi_call(
        &mut self,
        receiver: &Expr,
        field: &str,
        args: &[Expr],
        dsts: &[usize],
        mutated_arg: usize,
    ) -> Result<(), CompileError> {
        let Some(target_name) = args.get(mutated_arg).and_then(|arg| match arg {
            Expr::Ident(name) => Some(name.as_str()),
            _ => None,
        }) else {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "method `{field}` requires an identifier as argument {} in the current subset",
                    mutated_arg + 1
                ),
            });
        };

        let (target, global, is_by_ref) = if let Some(target) = self.lookup_local(target_name) {
            let is_by_ref = self.scopes.captured_by_ref.contains(target_name);
            (target, None, is_by_ref)
        } else if let Some(global) = self.lookup_global(target_name) {
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::LoadGlobal {
                dst: target,
                global,
            });
            (target, Some(global), false)
        } else {
            return Err(CompileError::UnknownIdentifier {
                name: target_name.to_string(),
            });
        };

        let value_reg = if is_by_ref {
            let dereffed = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: dereffed,
                src: target,
            });
            dereffed
        } else {
            target
        };

        let receiver = self.compile_value_expr(receiver)?;
        let mut registers = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            if index == mutated_arg {
                registers.push(value_reg);
            } else {
                registers.push(self.compile_value_expr(arg)?);
            }
        }
        self.emitter
            .code
            .push(Instruction::CallMethodMultiMutatingArg {
                receiver,
                method: field.to_string(),
                args: registers,
                dsts: dsts.to_vec(),
                mutated_arg,
            });
        if is_by_ref {
            self.emitter.code.push(Instruction::StoreIndirect {
                target,
                src: value_reg,
            });
        }
        if let Some(global) = global {
            self.emitter.code.push(Instruction::StoreGlobal {
                global,
                src: value_reg,
            });
        }
        Ok(())
    }

    pub(super) fn compile_mutating_stdlib_method_call(
        &mut self,
        receiver: &Expr,
        field: &str,
        function: gowasm_vm::StdlibFunctionId,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if dst.is_some() {
            return Err(CompileError::Unsupported {
                detail: format!("method `{field}` cannot be used in value position"),
            });
        }

        let mut pointer_target = None;
        let target = self.alloc_register();
        let pointer_result = self.compile_expr_into(
            target,
            &Expr::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(receiver.clone()),
            },
        );
        match pointer_result {
            Ok(()) => pointer_target = Some(target),
            Err(CompileError::Unsupported { .. }) => {}
            Err(err) => return Err(err),
        }

        let receiver_reg = if let Some(target) = pointer_target {
            let value_reg = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: value_reg,
                src: target,
            });
            value_reg
        } else {
            self.compile_stdlib_method_receiver(receiver, field)?
        };

        let mut registers = Vec::with_capacity(args.len() + 1);
        registers.push(receiver_reg);
        for arg in args {
            registers.push(self.compile_value_expr(arg)?);
        }
        self.emitter.code.push(Instruction::CallStdlib {
            function,
            args: registers,
            dst: Some(receiver_reg),
        });
        if let Some(target) = pointer_target {
            self.emitter.code.push(Instruction::StoreIndirect {
                target,
                src: receiver_reg,
            });
        }
        Ok(())
    }
}
