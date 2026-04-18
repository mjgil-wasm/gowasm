use super::*;
use crate::stdlib_function_values::{
    is_registered_stdlib_package, unsupported_stdlib_selector_detail,
};

impl FunctionBuilder<'_> {
    fn compile_imported_function_call_args(
        &mut self,
        package: &str,
        symbol: &str,
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let Some((param_types, _)) = self
            .lookup_imported_function_type(package, symbol)
            .and_then(parse_function_type)
        else {
            return self.compile_param_typed_args(&[], args);
        };

        if !self.imported_function_is_variadic(package, symbol) {
            return self.compile_param_typed_args(&param_types, args);
        }

        let fixed_count = param_types.len().saturating_sub(1);
        let mut registers =
            self.compile_param_typed_args(&param_types[..fixed_count], &args[..fixed_count])?;
        let variadic_args = &args[fixed_count..];
        if variadic_args.len() == 1 {
            if let Expr::Spread { expr } = &variadic_args[0] {
                registers.push(self.compile_typed_value_expr(
                    expr,
                    param_types.get(fixed_count).map(String::as_str),
                )?);
                return Ok(registers);
            }
        }

        let variadic_element_type = param_types
            .get(fixed_count)
            .and_then(|typ| typ.strip_prefix("[]"));
        let variadic_items = variadic_args
            .iter()
            .map(|arg| self.compile_typed_value_expr(arg, variadic_element_type))
            .collect::<Result<Vec<_>, _>>()?;
        let slice_reg = self.alloc_register();
        self.emitter.code.push(Instruction::MakeSlice {
            dst: slice_reg,
            concrete_type: param_types
                .get(fixed_count)
                .map(|typ| self.lower_runtime_concrete_type(typ))
                .transpose()?,
            items: variadic_items,
        });
        registers.push(slice_reg);
        Ok(registers)
    }

    fn imported_function_result_count(&self, package: &str, symbol: &str) -> usize {
        self.lookup_imported_function_result_types(package, symbol)
            .map(Vec::len)
            .unwrap_or(0)
    }

    pub(super) fn try_compile_imported_selector_call(
        &mut self,
        receiver_name: &str,
        package_path: &str,
        field: &str,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<bool, CompileError> {
        if let Some(function) = resolve_stdlib_function(package_path, field) {
            self.validate_stdlib_call_signature(package_path, field, args)?;
            let result_count = stdlib_function_result_count(function);
            if dst.is_some() && result_count != 1 {
                return Err(CompileError::Unsupported {
                    detail: format!("`{receiver_name}.{field}` cannot be used in value position"),
                });
            }

            if stdlib_function_mutates_first_arg(function) {
                self.compile_mutating_stdlib_call(receiver_name, field, function, args, dst)?;
                return Ok(true);
            }

            let uses_stringer =
                matches!(field, "Println" | "Sprint" | "Sprintln") && package_path == "fmt";
            let mut registers = self.compile_stdlib_call_args(function, args, 0)?;
            if uses_stringer {
                for (index, arg) in args.iter().enumerate() {
                    if let Some(reg) = self.maybe_apply_stringer(arg, registers[index])? {
                        registers[index] = reg;
                    }
                }
            }
            if result_count > 1 {
                self.emitter.code.push(Instruction::CallStdlibMulti {
                    function,
                    args: registers,
                    dsts: Vec::new(),
                });
            } else {
                self.emitter.code.push(Instruction::CallStdlib {
                    function,
                    args: registers,
                    dst,
                });
            }
            return Ok(true);
        }

        let Some(function) = self.lookup_imported_function_id(package_path, field) else {
            if is_registered_stdlib_package(package_path) {
                return Err(CompileError::Unsupported {
                    detail: unsupported_stdlib_selector_detail(receiver_name, field),
                });
            }
            return Ok(false);
        };
        self.validate_imported_function_call_arity(package_path, field, args)?;
        self.validate_imported_function_call_types(package_path, field, args)?;
        let result_count = self.imported_function_result_count(package_path, field);
        if dst.is_some() && result_count != 1 {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{receiver_name}.{field}` returns {result_count} value(s) and cannot be used in single-value position"
                ),
            });
        }
        let registers = self.compile_imported_function_call_args(package_path, field, args)?;
        self.emitter.code.push(Instruction::CallFunction {
            function,
            args: registers,
            dst,
        });
        Ok(true)
    }

    pub(super) fn try_compile_imported_selector_multi_call(
        &mut self,
        receiver_name: &str,
        package_path: &str,
        field: &str,
        args: &[Expr],
        dsts: &[usize],
    ) -> Result<bool, CompileError> {
        if let Some(function) = resolve_stdlib_function(package_path, field) {
            self.validate_stdlib_call_signature(package_path, field, args)?;
            let result_count = stdlib_function_result_count(function);
            if result_count != dsts.len() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "`{receiver_name}.{field}` returns {result_count} value(s), not {}",
                        dsts.len()
                    ),
                });
            }
            let registers = self.compile_stdlib_call_args(function, args, 0)?;
            self.emitter.code.push(Instruction::CallStdlibMulti {
                function,
                args: registers,
                dsts: dsts.to_vec(),
            });
            return Ok(true);
        }

        let Some(function) = self.lookup_imported_function_id(package_path, field) else {
            if is_registered_stdlib_package(package_path) {
                return Err(CompileError::Unsupported {
                    detail: unsupported_stdlib_selector_detail(receiver_name, field),
                });
            }
            return Ok(false);
        };
        self.validate_imported_function_call_arity(package_path, field, args)?;
        self.validate_imported_function_call_types(package_path, field, args)?;
        let result_count = self.imported_function_result_count(package_path, field);
        if result_count != dsts.len() {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{receiver_name}.{field}` returns {result_count} value(s), not {}",
                    dsts.len()
                ),
            });
        }
        let registers = self.compile_imported_function_call_args(package_path, field, args)?;
        self.emitter.code.push(Instruction::CallFunctionMulti {
            function,
            args: registers,
            dsts: dsts.to_vec(),
        });
        Ok(true)
    }

    pub(super) fn try_compile_imported_selector_go_call(
        &mut self,
        _receiver_name: &str,
        package_path: &str,
        field: &str,
        args: &[Expr],
    ) -> Result<bool, CompileError> {
        if let Some(function) = resolve_stdlib_function(package_path, field) {
            self.validate_stdlib_call_signature(package_path, field, args)?;
            let uses_stringer = matches!(field, "Println" | "Sprint" | "Sprintln" | "Print")
                && package_path == "fmt";
            let mut registers = self.compile_stdlib_call_args(function, args, 0)?;
            if uses_stringer {
                for (index, arg) in args.iter().enumerate() {
                    if let Some(reg) = self.maybe_apply_stringer(arg, registers[index])? {
                        registers[index] = reg;
                    }
                }
            }
            self.emitter.code.push(Instruction::GoCallStdlib {
                function,
                args: registers,
            });
            return Ok(true);
        }

        let Some(function) = self.lookup_imported_function_id(package_path, field) else {
            if is_registered_stdlib_package(package_path) {
                return Err(CompileError::Unsupported {
                    detail: unsupported_stdlib_selector_detail(_receiver_name, field),
                });
            }
            return Ok(false);
        };
        self.validate_imported_function_call_arity(package_path, field, args)?;
        self.validate_imported_function_call_types(package_path, field, args)?;
        let registers = self.compile_imported_function_call_args(package_path, field, args)?;
        self.emitter.code.push(Instruction::GoCall {
            function,
            args: registers,
        });
        Ok(true)
    }

    pub(super) fn try_compile_imported_selector_defer_call(
        &mut self,
        _receiver_name: &str,
        package_path: &str,
        field: &str,
        args: &[Expr],
    ) -> Result<bool, CompileError> {
        if let Some(function) = resolve_stdlib_function(package_path, field) {
            let mut registers = Vec::with_capacity(args.len());
            for arg in args {
                registers.push(self.compile_value_expr(arg)?);
            }
            self.emitter.code.push(Instruction::DeferStdlib {
                function,
                args: registers,
            });
            return Ok(true);
        }

        let Some(function) = self.lookup_imported_function_id(package_path, field) else {
            if is_registered_stdlib_package(package_path) {
                return Err(CompileError::Unsupported {
                    detail: unsupported_stdlib_selector_detail(_receiver_name, field),
                });
            }
            return Ok(false);
        };
        self.validate_imported_function_call_arity(package_path, field, args)?;
        self.validate_imported_function_call_types(package_path, field, args)?;
        let registers = self.compile_imported_function_call_args(package_path, field, args)?;
        self.emitter.code.push(Instruction::DeferFunction {
            function,
            args: registers,
        });
        Ok(true)
    }
}
