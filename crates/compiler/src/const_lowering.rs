use super::*;
use crate::const_eval::{ConstValue, ConstValueInfo};

impl FunctionBuilder<'_> {
    pub(super) fn compile_const_expr_value(
        &mut self,
        dst: usize,
        expr: &Expr,
        target_type: Option<&str>,
    ) -> Result<bool, CompileError> {
        let Some(info) = self.try_eval_const_expr(expr) else {
            return Ok(false);
        };
        self.compile_const_value_info(dst, &info, target_type)?;
        Ok(true)
    }

    pub(super) fn compile_const_value_info(
        &mut self,
        dst: usize,
        info: &ConstValueInfo,
        target_type: Option<&str>,
    ) -> Result<(), CompileError> {
        let coerced = match target_type {
            Some(target_type) => self.coerce_const_value_info(info, target_type)?,
            None => ConstValueInfo {
                value: info.value.clone(),
                typ: Some(info.visible_type_name()),
            },
        };
        let emit_type = coerced
            .typ
            .clone()
            .expect("compiled constant should have an emission type");
        let base_type = self.const_runtime_base_type(&coerced.value, &emit_type)?;
        match &coerced.value {
            ConstValue::Int(value) => {
                self.emitter
                    .code
                    .push(Instruction::LoadInt { dst, value: *value });
            }
            ConstValue::Float(bits) => {
                self.emitter.code.push(Instruction::LoadFloat {
                    dst,
                    value: gowasm_vm::Float64(f64::from_bits(*bits)),
                });
            }
            ConstValue::Bool(value) => {
                self.emitter
                    .code
                    .push(Instruction::LoadBool { dst, value: *value });
            }
            ConstValue::String(value) => {
                self.emitter.code.push(Instruction::LoadString {
                    dst,
                    value: value.clone(),
                });
            }
        }

        if emit_type != base_type {
            let Some(base_runtime_type) = self.runtime_type_for_value_target(base_type) else {
                return Ok(());
            };
            let Some(target_runtime_type) = self.runtime_type_for_value_target(&emit_type) else {
                return Ok(());
            };
            if base_runtime_type != target_runtime_type {
                self.emitter.code.push(Instruction::Retag {
                    dst,
                    src: dst,
                    typ: target_runtime_type,
                });
            }
        }

        if let Some(target_type) = target_type {
            if target_type == "interface{}"
                || target_type == "any"
                || self.instantiated_interface_type(target_type).is_some()
            {
                if !self.type_allows_nil(&emit_type) {
                    return Ok(());
                }
                let Some(source_runtime_type) = self.runtime_type_for_value_target(&emit_type)
                else {
                    return Ok(());
                };
                let Some(target_runtime_type) = self.runtime_type_for_value_target(target_type)
                else {
                    return Ok(());
                };
                if source_runtime_type != target_runtime_type {
                    self.emitter.code.push(Instruction::Retag {
                        dst,
                        src: dst,
                        typ: target_runtime_type,
                    });
                }
            }
        }

        Ok(())
    }

    pub(super) fn validate_const_assignable_type(
        &self,
        target_type: &str,
        info: &ConstValueInfo,
    ) -> Result<(), CompileError> {
        self.coerce_const_value_info(info, target_type).map(|_| ())
    }

    pub(super) fn known_int_const_value(&self, expr: &Expr) -> Option<i64> {
        let info = self.try_eval_const_expr(expr)?;
        self.coerce_const_value_info(&info, "int")
            .ok()
            .and_then(|coerced| match coerced.value {
                ConstValue::Int(value) => Some(value),
                _ => None,
            })
    }

    pub(super) fn coerce_const_value_info(
        &self,
        info: &ConstValueInfo,
        target_type: &str,
    ) -> Result<ConstValueInfo, CompileError> {
        if target_type == "interface{}"
            || target_type == "any"
            || self
                .instantiated_interface_type(target_type)
                .is_some_and(|interface| interface.methods.is_empty())
        {
            return Ok(ConstValueInfo {
                value: info.value.clone(),
                typ: Some(info.visible_type_name()),
            });
        }
        if let Some(source_type) = info.typ.as_deref() {
            self.validate_assignable_source_type(target_type, source_type)?;
        }
        let value = self.convert_const_value_for_target(&info.value, target_type)?;
        Ok(ConstValueInfo::typed(value, target_type))
    }

    fn convert_const_value_for_target(
        &self,
        value: &ConstValue,
        target_type: &str,
    ) -> Result<ConstValue, CompileError> {
        match self.terminal_scalar_type(target_type).as_str() {
            "int" => self
                .const_value_to_int(value, target_type)
                .map(ConstValue::Int),
            "byte" => self
                .const_value_to_int(value, target_type)
                .and_then(|value| {
                    (0..=255)
                        .contains(&value)
                        .then_some(value)
                        .ok_or_else(|| self.const_representability_error(target_type, value))
                })
                .map(ConstValue::Int),
            "rune" => self
                .const_value_to_int(value, target_type)
                .and_then(|value| {
                    ((i32::MIN as i64)..=(i32::MAX as i64))
                        .contains(&value)
                        .then_some(value)
                        .ok_or_else(|| self.const_representability_error(target_type, value))
                })
                .map(ConstValue::Int),
            "float64" => self
                .const_value_to_float(value, target_type)
                .map(ConstValue::Float),
            "bool" => match value {
                ConstValue::Bool(value) => Ok(ConstValue::Bool(*value)),
                _ => self.unsupported_const_assignment(target_type, value),
            },
            "string" => match value {
                ConstValue::String(value) => Ok(ConstValue::String(value.clone())),
                ConstValue::Int(value) => Ok(ConstValue::String(
                    char::from_u32(*value as u32)
                        .unwrap_or(char::REPLACEMENT_CHARACTER)
                        .to_string(),
                )),
                _ => self.unsupported_const_assignment(target_type, value),
            },
            _ => self.unsupported_const_assignment(target_type, value),
        }
    }

    fn const_value_to_int(
        &self,
        value: &ConstValue,
        target_type: &str,
    ) -> Result<i64, CompileError> {
        match value {
            ConstValue::Int(value) => Ok(*value),
            ConstValue::Float(bits) => {
                let value = f64::from_bits(*bits);
                if !value.is_finite() || value.fract() != 0.0 {
                    return Err(self.const_type_error(target_type, "float64"));
                }
                let int = value as i64;
                if (int as f64).to_bits() != value.to_bits() {
                    return Err(self.const_type_error(target_type, "float64"));
                }
                Ok(int)
            }
            ConstValue::Bool(_) => Err(self.const_type_error(target_type, "bool")),
            ConstValue::String(_) => Err(self.const_type_error(target_type, "string")),
        }
    }

    fn const_value_to_float(
        &self,
        value: &ConstValue,
        target_type: &str,
    ) -> Result<u64, CompileError> {
        match value {
            ConstValue::Float(bits) => Ok(*bits),
            ConstValue::Int(value) => {
                let float = *value as f64;
                ((float as i64) == *value)
                    .then_some(float.to_bits())
                    .ok_or_else(|| self.const_representability_error(target_type, *value))
            }
            ConstValue::Bool(_) => Err(self.const_type_error(target_type, "bool")),
            ConstValue::String(_) => Err(self.const_type_error(target_type, "string")),
        }
    }

    fn const_runtime_base_type<'a>(
        &self,
        value: &ConstValue,
        emit_type: &'a str,
    ) -> Result<&'a str, CompileError> {
        match value {
            ConstValue::Int(_) => {
                if self.int_like_const_type(emit_type) {
                    Ok("int")
                } else {
                    self.unsupported_const_assignment(emit_type, value)
                }
            }
            ConstValue::Float(_) => {
                if self.float_const_type(emit_type) {
                    Ok("float64")
                } else {
                    self.unsupported_const_assignment(emit_type, value)
                }
            }
            ConstValue::Bool(_) => {
                if self.bool_const_type(emit_type) {
                    Ok("bool")
                } else {
                    self.unsupported_const_assignment(emit_type, value)
                }
            }
            ConstValue::String(_) => {
                if self.string_const_type(emit_type) {
                    Ok("string")
                } else {
                    self.unsupported_const_assignment(emit_type, value)
                }
            }
        }
    }
}
