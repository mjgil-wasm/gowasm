use super::*;
use crate::types::display_type;
use gowasm_parser::Parameter;
use gowasm_vm::{
    StdlibFunctionId, TypeId, TYPE_ARRAY, TYPE_BOOL, TYPE_CHANNEL, TYPE_FLOAT64, TYPE_FUNCTION,
    TYPE_INT, TYPE_INT64, TYPE_MAP, TYPE_SLICE, TYPE_STRING,
};

pub(crate) fn predeclared_conversion_target(name: &str) -> Option<&'static str> {
    match name {
        "int" => Some("int"),
        "rune" => Some("rune"),
        "float64" => Some("float64"),
        "string" => Some("string"),
        "byte" => Some("byte"),
        _ => None,
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn imported_selector_alias_name(
        &self,
        receiver_name: &str,
        field: &str,
    ) -> Option<String> {
        self.env
            .imported_packages
            .contains_key(receiver_name)
            .then(|| {
                let alias_name = format!("{receiver_name}.{field}");
                self.instantiated_alias_type(alias_name.as_str())
                    .is_some()
                    .then_some(alias_name)
            })?
    }

    pub(super) fn compile_alias_conversion_call(
        &mut self,
        alias_name: &str,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if self.instantiated_alias_type(alias_name).is_none() {
            return Err(CompileError::Unsupported {
                detail: format!("unknown alias type `{alias_name}`"),
            });
        }
        self.compile_named_conversion_call(alias_name, args, dst)
    }

    pub(super) fn compile_named_conversion_call(
        &mut self,
        target_type: &str,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        if args.len() != 1 {
            return Err(CompileError::Unsupported {
                detail: format!("`{target_type}` type conversion expects exactly one argument"),
            });
        }
        let Some((runtime_target, retag_type)) = self.named_conversion_target(target_type) else {
            return self.compile_passthrough_alias_conversion(target_type, &args[0], dst);
        };
        let dst = dst.unwrap_or_else(|| self.alloc_register());
        if let Some(info) = self.try_eval_const_expr(&args[0]) {
            self.validate_const_conversion_source(target_type, runtime_target, &info)?;
            let info = self.maybe_truncate_explicit_const_float_conversion(
                target_type,
                runtime_target,
                info,
            )?;
            self.compile_const_value_info(dst, &info, Some(target_type))?;
            return Ok(());
        }

        self.validate_runtime_conversion_source(target_type, runtime_target, &args[0])?;
        let src = self.compile_value_expr(&args[0])?;
        let instruction = match runtime_target {
            "int" | "rune" => Instruction::ConvertToInt { dst, src },
            "float64" => Instruction::ConvertToFloat64 { dst, src },
            "string" => {
                let source_type = self
                    .infer_expr_type_name(&args[0])
                    .expect("validated runtime conversions should have a source type");
                let source_underlying = self.instantiated_underlying_type_name(&source_type);
                if source_underlying == "[]rune" {
                    Instruction::ConvertRuneSliceToString { dst, src }
                } else {
                    Instruction::ConvertToString { dst, src }
                }
            }
            "byte" => Instruction::ConvertToByte { dst, src },
            _ => unreachable!("unsupported named conversion target"),
        };
        self.emitter.code.push(instruction);

        if let Some(retag_type) = retag_type {
            self.emitter.code.push(Instruction::Retag {
                dst,
                src: dst,
                typ: retag_type,
            });
        }
        Ok(())
    }

    fn compile_passthrough_alias_conversion(
        &mut self,
        alias_name: &str,
        value: &Expr,
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        let Some(alias) = self.instantiated_alias_type(alias_name) else {
            return Err(CompileError::Unsupported {
                detail: format!("unknown alias type `{alias_name}`"),
            });
        };
        let dst = dst.unwrap_or_else(|| self.alloc_register());
        if self.compile_const_expr_value(dst, value, Some(alias_name))? {
            return Ok(());
        }
        let src = self.compile_value_expr(value)?;
        self.emitter.code.push(Instruction::Retag {
            dst,
            src,
            typ: alias.type_id,
        });
        Ok(())
    }

    fn named_conversion_target(&self, target_type: &str) -> Option<(&'static str, Option<TypeId>)> {
        if let Some(target) = predeclared_conversion_target(target_type) {
            return Some((target, None));
        }

        let alias = self.instantiated_alias_type(target_type)?;
        let underlying = self.instantiated_underlying_type_name(target_type);
        let target = predeclared_conversion_target(underlying.as_str())?;
        Some((target, Some(alias.type_id)))
    }

    fn validate_runtime_conversion_source(
        &self,
        target_type: &str,
        runtime_target: &str,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let Some(source_type) = self.infer_expr_type_name(expr) else {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "cannot convert expression to `{}` in the current subset",
                    display_type(target_type)
                ),
            });
        };
        let source_underlying = self.instantiated_underlying_type_name(&source_type);
        let allowed = match runtime_target {
            "int" | "rune" | "byte" => {
                self.int_like_const_type(&source_underlying)
                    || self.float_const_type(&source_underlying)
            }
            "float64" => {
                self.int_like_const_type(&source_underlying)
                    || self.float_const_type(&source_underlying)
            }
            "string" => {
                self.string_const_type(&source_underlying)
                    || self.int_like_const_type(&source_underlying)
                    || source_underlying == "[]byte"
                    || source_underlying == "[]rune"
            }
            _ => false,
        };
        if allowed {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "cannot convert expression of type `{}` to `{}` in the current subset",
                display_type(&source_type),
                display_type(target_type)
            ),
        })
    }

    fn validate_const_conversion_source(
        &self,
        target_type: &str,
        runtime_target: &str,
        info: &crate::const_eval::ConstValueInfo,
    ) -> Result<(), CompileError> {
        let allowed = match runtime_target {
            "int" | "rune" | "byte" => matches!(
                info.value,
                crate::const_eval::ConstValue::Int(_) | crate::const_eval::ConstValue::Float(_)
            ),
            "float64" => matches!(
                info.value,
                crate::const_eval::ConstValue::Int(_) | crate::const_eval::ConstValue::Float(_)
            ),
            "string" => matches!(
                info.value,
                crate::const_eval::ConstValue::String(_) | crate::const_eval::ConstValue::Int(_)
            ),
            _ => false,
        };
        if allowed {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "cannot convert expression of type `{}` to `{}` in the current subset",
                display_type(&info.visible_type_name()),
                display_type(target_type)
            ),
        })
    }

    fn maybe_truncate_explicit_const_float_conversion(
        &self,
        target_type: &str,
        runtime_target: &str,
        info: crate::const_eval::ConstValueInfo,
    ) -> Result<crate::const_eval::ConstValueInfo, CompileError> {
        if !matches!(runtime_target, "int" | "rune" | "byte") {
            return Ok(info);
        }
        let crate::const_eval::ConstValue::Float(bits) = info.value else {
            return Ok(info);
        };
        let value = f64::from_bits(bits);
        if !value.is_finite() {
            return Err(self.const_type_error(target_type, "float64"));
        }
        let truncated = value.trunc();
        if truncated < i64::MIN as f64 || truncated > i64::MAX as f64 {
            return Err(CompileError::Unsupported {
                detail: "constant expression overflows supported int range".into(),
            });
        }
        Ok(crate::const_eval::ConstValueInfo::typed(
            crate::const_eval::ConstValue::Int(truncated as i64),
            runtime_target,
        ))
    }

    pub(super) fn compile_typed_value_expr(
        &mut self,
        expr: &Expr,
        target_type: Option<&str>,
    ) -> Result<usize, CompileError> {
        let dst = self.alloc_register();
        self.compile_expr_into_with_hint(dst, expr, target_type)?;
        Ok(dst)
    }

    pub(super) fn compile_param_typed_args(
        &mut self,
        param_types: &[String],
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let mut registers = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            registers.push(
                self.compile_typed_value_expr(arg, param_types.get(index).map(String::as_str))?,
            );
        }
        Ok(registers)
    }

    pub(super) fn compile_method_call_args(
        &mut self,
        params: &[Parameter],
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let param_types = params
            .iter()
            .map(|param| param.typ.clone())
            .collect::<Vec<_>>();
        self.compile_param_typed_args(&param_types, args)
    }

    pub(super) fn compile_function_value_args(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let Some(function_type) = self.expr_function_type_name(callee) else {
            return self.compile_param_typed_args(&[], args);
        };
        let Some((param_types, _)) = parse_function_type(&function_type) else {
            return self.compile_param_typed_args(&[], args);
        };
        self.compile_param_typed_args(&param_types, args)
    }

    pub(super) fn compile_stdlib_call_args(
        &mut self,
        function: StdlibFunctionId,
        args: &[Expr],
        skip_receiver: usize,
    ) -> Result<Vec<usize>, CompileError> {
        let fixed_param_types = stdlib_function_param_types(function).unwrap_or(&[]);
        let fixed_param_types = fixed_param_types.get(skip_receiver..).unwrap_or(&[]);
        if let Some(variadic_param_type) = stdlib_function_variadic_param_type(function) {
            let mut registers = Vec::with_capacity(args.len());
            for (index, arg) in args.iter().enumerate().take(fixed_param_types.len()) {
                registers.push(self.compile_typed_value_expr(arg, Some(fixed_param_types[index]))?);
            }
            let variadic_args = &args[fixed_param_types.len()..];
            if variadic_args.len() == 1 {
                if let Expr::Spread { expr } = &variadic_args[0] {
                    let slice_type = format!("[]{variadic_param_type}");
                    registers.push(self.compile_typed_value_expr(expr, Some(&slice_type))?);
                    return Ok(registers);
                }
            }
            for arg in variadic_args {
                registers.push(self.compile_typed_value_expr(arg, Some(variadic_param_type))?);
            }
            return Ok(registers);
        }

        let param_types = fixed_param_types
            .iter()
            .map(|typ| (*typ).to_string())
            .collect::<Vec<_>>();
        self.compile_param_typed_args(&param_types, args)
    }

    pub(super) fn compile_named_call_args(
        &mut self,
        name: &str,
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let Some((param_types, _)) = self
            .env
            .function_types
            .get(name)
            .and_then(|function_type| parse_function_type(function_type))
        else {
            return self.compile_param_typed_args(&[], args);
        };

        if !self.env.variadic_functions.contains(name) {
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

    pub(super) fn maybe_retag_assignable_value(
        &mut self,
        dst: usize,
        expr: &Expr,
        target_type: &str,
    ) -> Result<(), CompileError> {
        let Some(source_type) = self.infer_expr_type_name(expr) else {
            return Ok(());
        };
        if source_type == target_type {
            return Ok(());
        }
        if !self.types_assignable(target_type, &source_type)
            && !self.literal_assignable_to(target_type, expr)
        {
            return Ok(());
        }

        let Some(target_runtime_type) = self.runtime_type_for_value_target(target_type) else {
            return Ok(());
        };
        if self.target_is_interface_type(target_type) && self.target_is_interface_type(&source_type)
        {
            return Ok(());
        }
        if self.target_is_interface_type(target_type) && !self.type_allows_nil(&source_type) {
            return Ok(());
        }
        if self.runtime_type_for_value_target(&source_type) == Some(target_runtime_type) {
            return Ok(());
        }

        self.emitter.code.push(Instruction::Retag {
            dst,
            src: dst,
            typ: target_runtime_type,
        });
        Ok(())
    }

    pub(super) fn runtime_type_for_value_target(&self, typ: &str) -> Option<TypeId> {
        if let Some(alias) = self.instantiated_alias_type(typ) {
            return Some(alias.type_id);
        }
        if let Some(struct_type) = self.instantiated_struct_type(typ) {
            return Some(struct_type.type_id);
        }

        match typ {
            "int" | "byte" | "rune" => Some(TYPE_INT),
            "int64" => Some(TYPE_INT64),
            "float64" => Some(TYPE_FLOAT64),
            "string" => Some(TYPE_STRING),
            "bool" => Some(TYPE_BOOL),
            "interface{}" | "any" => Some(gowasm_vm::TYPE_ANY),
            "error" => Some(gowasm_vm::TYPE_ERROR),
            typ if self.instantiated_interface_type(typ).is_some() => self
                .instantiated_interface_type(typ)
                .map(|interface| interface.type_id),
            typ if parse_array_type(typ).is_some() => Some(TYPE_ARRAY),
            typ if typ.starts_with("[]") => Some(TYPE_SLICE),
            typ if parse_map_type(typ).is_some() => Some(TYPE_MAP),
            typ if parse_function_type(typ).is_some() => Some(TYPE_FUNCTION),
            typ if parse_channel_type(typ).is_some() => Some(TYPE_CHANNEL),
            typ => parse_pointer_type(typ).map(|inner| self.pointer_runtime_type(inner)),
        }
    }

    fn target_is_interface_type(&self, typ: &str) -> bool {
        matches!(typ, "interface{}" | "any" | "error")
            || self.instantiated_interface_type(typ).is_some()
    }
}
