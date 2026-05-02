use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn declare_named_results(&mut self) -> Result<(), CompileError> {
        if !self.has_named_results() {
            return Ok(());
        }
        let names: Vec<_> = self.control.current_result_names.clone();
        let types: Vec<_> = self.control.current_result_types.clone();
        for (name, typ) in names.iter().zip(types.iter()) {
            if name.is_empty() {
                continue;
            }
            let register = self.alloc_register();
            self.compile_zero_value(register, typ)?;
            self.bind_initialized_local(name, register, Some(typ.clone()));
        }
        Ok(())
    }

    pub(super) fn emit_implicit_return(&mut self) {
        if self.has_named_results() {
            let names = self.control.current_result_names.clone();
            let mut srcs = Vec::new();
            for name in &names {
                if let Some(reg) = self.lookup_local(name) {
                    srcs.push(reg);
                }
            }
            if srcs.len() == 1 {
                self.emitter
                    .code
                    .push(Instruction::Return { src: Some(srcs[0]) });
            } else if srcs.len() > 1 {
                self.emitter.code.push(Instruction::ReturnMulti { srcs });
            } else {
                self.emitter.code.push(Instruction::Return { src: None });
            }
        } else {
            self.emitter.code.push(Instruction::Return { src: None });
        }
    }

    fn has_named_results(&self) -> bool {
        !self.control.current_result_names.is_empty()
            && self
                .control
                .current_result_names
                .iter()
                .any(|n| !n.is_empty())
    }

    pub(super) fn compile_return(&mut self, values: &[Expr]) -> Result<(), CompileError> {
        if values.is_empty() && self.has_named_results() {
            let names = self.control.current_result_names.clone();
            let mut srcs = Vec::new();
            for name in &names {
                if let Some(reg) = self.lookup_local(name) {
                    srcs.push(reg);
                }
            }
            return match srcs.len() {
                0 => {
                    self.emitter.code.push(Instruction::Return { src: None });
                    Ok(())
                }
                1 => {
                    self.emitter
                        .code
                        .push(Instruction::Return { src: Some(srcs[0]) });
                    Ok(())
                }
                _ => {
                    self.emitter.code.push(Instruction::ReturnMulti { srcs });
                    Ok(())
                }
            };
        }

        if values.len() == 1 && self.control.current_result_types.len() > 1 {
            if let Expr::Call {
                callee,
                type_args,
                args,
            } = &values[0]
            {
                if self
                    .infer_expr_result_types(&values[0])
                    .is_some_and(|result_types| {
                        result_types.len() == self.control.current_result_types.len()
                    })
                {
                    let mut srcs = Vec::with_capacity(self.control.current_result_types.len());
                    for _ in 0..self.control.current_result_types.len() {
                        srcs.push(self.alloc_register());
                    }
                    self.compile_multi_call(callee, type_args, args, &srcs)?;
                    self.emitter.code.push(Instruction::ReturnMulti { srcs });
                    return Ok(());
                }
            }
        }

        if values.len() != self.control.current_result_types.len() {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "return value count {} does not match {} declared result(s) in the current subset",
                    values.len(),
                    self.control.current_result_types.len()
                ),
            });
        }

        match values {
            [] => self.emitter.code.push(Instruction::Return { src: None }),
            [value] => {
                let result_type = self.control.current_result_types.first().cloned();
                self.validate_assignable_type(result_type.as_deref(), value)?;
                let register = self.alloc_register();
                self.compile_expr_into_with_hint(register, value, result_type.as_deref())?;
                self.emitter.code.push(Instruction::Return {
                    src: Some(register),
                });
            }
            _ => {
                let result_types = self.control.current_result_types.clone();
                let mut srcs = Vec::with_capacity(values.len());
                for (value, result_type) in values.iter().zip(result_types.iter()) {
                    self.validate_assignable_type(Some(result_type), value)?;
                    let register = self.alloc_register();
                    self.compile_expr_into_with_hint(register, value, Some(result_type))?;
                    srcs.push(register);
                }
                self.emitter.code.push(Instruction::ReturnMulti { srcs });
            }
        }
        Ok(())
    }
}
