use super::*;
use crate::types::display_type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ConstValue {
    Int(i64),
    Float(u64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConstValueInfo {
    pub(crate) value: ConstValue,
    pub(crate) typ: Option<String>,
}

impl ConstValueInfo {
    pub(crate) fn untyped(value: ConstValue) -> Self {
        Self { value, typ: None }
    }

    pub(crate) fn typed(value: ConstValue, typ: impl Into<String>) -> Self {
        Self {
            value,
            typ: Some(typ.into()),
        }
    }

    pub(crate) fn default_type_name(&self) -> String {
        match self.value {
            ConstValue::Int(_) => "int".into(),
            ConstValue::Float(_) => "float64".into(),
            ConstValue::Bool(_) => "bool".into(),
            ConstValue::String(_) => "string".into(),
        }
    }

    pub(crate) fn visible_type_name(&self) -> String {
        self.typ.clone().unwrap_or_else(|| self.default_type_name())
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn try_eval_const_expr(&self, expr: &Expr) -> Option<ConstValueInfo> {
        self.eval_const_expr(expr).ok()
    }

    pub(super) fn eval_const_expr(&self, expr: &Expr) -> Result<ConstValueInfo, CompileError> {
        match expr {
            Expr::IntLiteral(value) => Ok(ConstValueInfo::untyped(ConstValue::Int(*value))),
            Expr::FloatLiteral(bits) => Ok(ConstValueInfo::untyped(ConstValue::Float(*bits))),
            Expr::BoolLiteral(value) => Ok(ConstValueInfo::untyped(ConstValue::Bool(*value))),
            Expr::StringLiteral(value) => {
                Ok(ConstValueInfo::untyped(ConstValue::String(value.clone())))
            }
            Expr::Ident(name) => self.lookup_visible_const_info(name).ok_or_else(|| {
                CompileError::Unsupported {
                    detail: format!(
                        "const initializers can only reference const bindings in the current subset: `{name}`"
                    ),
                }
            }),
            Expr::Selector { receiver, field } => self.lookup_selector_const_info(receiver, field),
            Expr::Unary { op, expr } => self.eval_const_unary(*op, expr),
            Expr::Binary { left, op, right } => self.eval_const_binary(left, *op, right),
            _ => Err(CompileError::Unsupported {
                detail: "const initializers currently require supported constant expressions"
                    .into(),
            }),
        }
    }
    fn eval_const_unary(&self, op: UnaryOp, expr: &Expr) -> Result<ConstValueInfo, CompileError> {
        let info = self.eval_const_expr(expr)?;
        let result = match op {
            UnaryOp::Not => match info.value {
                ConstValue::Bool(value) => ConstValue::Bool(!value),
                _ => return self.unsupported_const_expr(),
            },
            UnaryOp::Negate => match info.value {
                ConstValue::Int(value) => {
                    ConstValue::Int(value.checked_neg().ok_or_else(|| {
                        CompileError::Unsupported {
                            detail: "constant expression overflows supported int range".into(),
                        }
                    })?)
                }
                ConstValue::Float(bits) => ConstValue::Float((-f64::from_bits(bits)).to_bits()),
                _ => return self.unsupported_const_expr(),
            },
            UnaryOp::BitNot => match info.value {
                ConstValue::Int(value) => ConstValue::Int(!value),
                _ => return self.unsupported_const_expr(),
            },
            UnaryOp::AddressOf | UnaryOp::Deref | UnaryOp::Receive => {
                return self.unsupported_const_expr()
            }
        };
        Ok(ConstValueInfo {
            value: result,
            typ: info.typ,
        })
    }

    fn eval_const_binary(
        &self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<ConstValueInfo, CompileError> {
        let left = self.eval_const_expr(left)?;
        let right = self.eval_const_expr(right)?;
        let result_type = self.const_binary_result_type(&left, &right, op)?;
        let (left, right) = if let Some(result_type) = result_type.as_deref() {
            (
                self.coerce_const_value_info(&left, result_type)?,
                self.coerce_const_value_info(&right, result_type)?,
            )
        } else {
            (left, right)
        };

        let value = match op {
            BinaryOp::Add => self.eval_const_add(&left.value, &right.value)?,
            BinaryOp::Subtract => self.eval_const_numeric(&left.value, &right.value, op)?,
            BinaryOp::Multiply => self.eval_const_numeric(&left.value, &right.value, op)?,
            BinaryOp::Divide => self.eval_const_numeric(&left.value, &right.value, op)?,
            BinaryOp::Modulo => self.eval_const_numeric(&left.value, &right.value, op)?,
            BinaryOp::ShiftLeft => self.eval_const_shift(&left.value, &right.value, true)?,
            BinaryOp::ShiftRight => self.eval_const_shift(&left.value, &right.value, false)?,
            BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::BitAnd | BinaryOp::BitClear => {
                self.eval_const_int_binary(&left.value, &right.value, op)?
            }
            BinaryOp::And => match (&left.value, &right.value) {
                (ConstValue::Bool(left), ConstValue::Bool(right)) => {
                    ConstValue::Bool(*left && *right)
                }
                _ => return self.unsupported_const_expr(),
            },
            BinaryOp::Or => match (&left.value, &right.value) {
                (ConstValue::Bool(left), ConstValue::Bool(right)) => {
                    ConstValue::Bool(*left || *right)
                }
                _ => return self.unsupported_const_expr(),
            },
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => self.eval_const_compare(&left.value, &right.value, op)?,
        };

        Ok(ConstValueInfo {
            value,
            typ: const_result_type_for_op(op, result_type),
        })
    }

    fn const_binary_result_type(
        &self,
        left: &ConstValueInfo,
        right: &ConstValueInfo,
        op: BinaryOp,
    ) -> Result<Option<String>, CompileError> {
        if matches!(
            op,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::And
                | BinaryOp::Or
        ) {
            return Ok(None);
        }

        if matches!(op, BinaryOp::ShiftLeft | BinaryOp::ShiftRight) {
            if let Some(typ) = left.typ.as_deref() {
                return Ok(Some(typ.to_string()));
            }
            return Ok(None);
        }

        let left_type = left.typ.as_deref();
        let right_type = right.typ.as_deref();
        match (left_type, right_type) {
            (Some(left_type), Some(right_type)) => {
                if left_type == right_type {
                    return Ok(Some(left_type.to_string()));
                }
                if self.int_like_const_type(left_type) && self.int_like_const_type(right_type) {
                    return Ok(Some(left_type.to_string()));
                }
                self.unsupported_const_expr()
            }
            (Some(typ), None) | (None, Some(typ)) => Ok(Some(typ.to_string())),
            (None, None) => Ok(None),
        }
    }

    fn eval_const_add(
        &self,
        left: &ConstValue,
        right: &ConstValue,
    ) -> Result<ConstValue, CompileError> {
        match (left, right) {
            (ConstValue::String(left), ConstValue::String(right)) => {
                Ok(ConstValue::String(format!("{left}{right}")))
            }
            _ => self.eval_const_numeric(left, right, BinaryOp::Add),
        }
    }

    fn eval_const_numeric(
        &self,
        left: &ConstValue,
        right: &ConstValue,
        op: BinaryOp,
    ) -> Result<ConstValue, CompileError> {
        match (left, right) {
            (ConstValue::Int(left), ConstValue::Int(right)) => {
                let value = match op {
                    BinaryOp::Add => left.checked_add(*right),
                    BinaryOp::Subtract => left.checked_sub(*right),
                    BinaryOp::Multiply => left.checked_mul(*right),
                    BinaryOp::Divide => {
                        if *right == 0 {
                            return Err(CompileError::Unsupported {
                                detail: "constant division by zero".into(),
                            });
                        }
                        Some(left / right)
                    }
                    BinaryOp::Modulo => {
                        if *right == 0 {
                            return Err(CompileError::Unsupported {
                                detail: "constant modulo by zero".into(),
                            });
                        }
                        Some(left % right)
                    }
                    _ => None,
                }
                .ok_or_else(|| CompileError::Unsupported {
                    detail: "constant expression overflows supported int range".into(),
                })?;
                Ok(ConstValue::Int(value))
            }
            (ConstValue::Float(left), ConstValue::Float(right)) => {
                let left = f64::from_bits(*left);
                let right = f64::from_bits(*right);
                let value = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Subtract => left - right,
                    BinaryOp::Multiply => left * right,
                    BinaryOp::Divide => left / right,
                    _ => return self.unsupported_const_expr(),
                };
                Ok(ConstValue::Float(value.to_bits()))
            }
            (ConstValue::Int(left), ConstValue::Float(right_bits)) => {
                let left = exact_float_from_int(*left)
                    .ok_or_else(|| self.const_representability_error("float64", *left))?;
                self.eval_const_numeric(
                    &ConstValue::Float(left),
                    &ConstValue::Float(*right_bits),
                    op,
                )
            }
            (ConstValue::Float(left_bits), ConstValue::Int(right)) => {
                let right = exact_float_from_int(*right)
                    .ok_or_else(|| self.const_representability_error("float64", *right))?;
                self.eval_const_numeric(
                    &ConstValue::Float(*left_bits),
                    &ConstValue::Float(right),
                    op,
                )
            }
            _ => self.unsupported_const_expr(),
        }
    }

    fn eval_const_shift(
        &self,
        left: &ConstValue,
        right: &ConstValue,
        left_shift: bool,
    ) -> Result<ConstValue, CompileError> {
        let ConstValue::Int(left) = left else {
            return self.unsupported_const_expr();
        };
        let ConstValue::Int(right) = right else {
            return self.unsupported_const_expr();
        };
        let shift = u32::try_from(*right).map_err(|_| CompileError::Unsupported {
            detail: "shift count must be non-negative".into(),
        })?;
        let value = if left_shift {
            if shift >= 127 {
                None
            } else {
                let shifted = (i128::from(*left)) << shift;
                i64::try_from(shifted).ok()
            }
        } else {
            Some(if shift >= 63 {
                if *left >= 0 {
                    0
                } else {
                    -1
                }
            } else {
                left >> shift
            })
        }
        .ok_or_else(|| CompileError::Unsupported {
            detail: "constant expression overflows supported int range".into(),
        })?;
        Ok(ConstValue::Int(value))
    }

    fn eval_const_int_binary(
        &self,
        left: &ConstValue,
        right: &ConstValue,
        op: BinaryOp,
    ) -> Result<ConstValue, CompileError> {
        let (ConstValue::Int(left), ConstValue::Int(right)) = (left, right) else {
            return self.unsupported_const_expr();
        };
        let value = match op {
            BinaryOp::BitOr => *left | *right,
            BinaryOp::BitXor => *left ^ *right,
            BinaryOp::BitAnd => *left & *right,
            BinaryOp::BitClear => *left & !*right,
            _ => unreachable!("non-int op routed through int const evaluator"),
        };
        Ok(ConstValue::Int(value))
    }

    fn eval_const_compare(
        &self,
        left: &ConstValue,
        right: &ConstValue,
        op: BinaryOp,
    ) -> Result<ConstValue, CompileError> {
        let value = match (left, right) {
            (ConstValue::Int(left), ConstValue::Int(right)) => {
                compare_ordering(left.cmp(right), op)
            }
            (ConstValue::Float(left), ConstValue::Float(right)) => {
                let left = f64::from_bits(*left);
                let right = f64::from_bits(*right);
                let ordering =
                    left.partial_cmp(&right)
                        .ok_or_else(|| CompileError::Unsupported {
                            detail: "cannot compare NaN constant values in the current subset"
                                .into(),
                        })?;
                compare_ordering(ordering, op)
            }
            (ConstValue::Int(left), ConstValue::Float(right_bits)) => {
                let left = exact_float_from_int(*left)
                    .ok_or_else(|| self.const_representability_error("float64", *left))?;
                return self.eval_const_compare(
                    &ConstValue::Float(left),
                    &ConstValue::Float(*right_bits),
                    op,
                );
            }
            (ConstValue::Float(left_bits), ConstValue::Int(right)) => {
                let right = exact_float_from_int(*right)
                    .ok_or_else(|| self.const_representability_error("float64", *right))?;
                return self.eval_const_compare(
                    &ConstValue::Float(*left_bits),
                    &ConstValue::Float(right),
                    op,
                );
            }
            (ConstValue::Bool(left), ConstValue::Bool(right)) => match op {
                BinaryOp::Equal => *left == *right,
                BinaryOp::NotEqual => *left != *right,
                _ => return self.unsupported_const_expr(),
            },
            (ConstValue::String(left), ConstValue::String(right)) => {
                compare_ordering(left.cmp(right), op)
            }
            _ => return self.unsupported_const_expr(),
        };
        Ok(ConstValue::Bool(value))
    }

    fn lookup_visible_const_info(&self, name: &str) -> Option<ConstValueInfo> {
        for (((scope, consts), types), values) in self
            .scopes
            .scopes
            .iter()
            .rev()
            .zip(self.scopes.const_scopes.iter().rev())
            .zip(self.scopes.type_scopes.iter().rev())
            .zip(self.scopes.const_value_scopes.iter().rev())
        {
            if scope.contains_key(name) {
                if !consts.contains(name) {
                    return None;
                }
                if let Some(info) = values.get(name) {
                    return Some(info.clone());
                }
                return types
                    .get(name)
                    .cloned()
                    .map(|typ| ConstValueInfo::typed(ConstValue::Int(0), typ));
            }
        }
        if self.control.in_package_init {
            return None;
        }
        self.env.globals.get(name).and_then(|binding| {
            binding
                .is_const
                .then_some(binding.const_value.clone())
                .flatten()
        })
    }

    fn lookup_selector_const_info(
        &self,
        receiver: &Expr,
        field: &str,
    ) -> Result<ConstValueInfo, CompileError> {
        let Expr::Ident(receiver_name) = receiver else {
            return self.unsupported_const_expr();
        };
        let Some(package_path) = self.env.imported_packages.get(receiver_name) else {
            return self.unsupported_const_expr();
        };

        if let Some(constant) = resolve_stdlib_constant(package_path, field) {
            if let Some(value) = stdlib_const_value(&constant.value) {
                return Ok(ConstValueInfo::typed(value, constant.typ.to_string()));
            }
            return self.unsupported_const_expr();
        }
        if let Some(value) = resolve_stdlib_value(package_path, field) {
            if let StdlibValueInit::Constant(constant) = value.value {
                if let Some(constant) = stdlib_const_value(&constant) {
                    return Ok(ConstValueInfo::typed(constant, value.typ.to_string()));
                }
                return self.unsupported_const_expr();
            }
        }
        if let Some(binding) = self.lookup_imported_global_binding(package_path, field) {
            if binding.is_const {
                if let Some(info) = &binding.const_value {
                    return Ok(info.clone());
                }
            }
        }

        self.unsupported_const_expr()
    }

    pub(super) fn int_like_const_type(&self, typ: &str) -> bool {
        matches!(
            self.terminal_scalar_type(typ).as_str(),
            "int" | "byte" | "rune"
        )
    }

    pub(super) fn float_const_type(&self, typ: &str) -> bool {
        self.terminal_scalar_type(typ) == "float64"
    }

    pub(super) fn bool_const_type(&self, typ: &str) -> bool {
        self.terminal_scalar_type(typ) == "bool"
    }

    pub(super) fn string_const_type(&self, typ: &str) -> bool {
        self.terminal_scalar_type(typ) == "string"
    }

    pub(super) fn terminal_scalar_type(&self, typ: &str) -> String {
        let mut current = typ.to_string();
        while let Some(alias) = self.instantiated_alias_type(&current) {
            current = alias.underlying.clone();
        }
        current
    }

    pub(super) fn const_type_error(&self, target_type: &str, source_type: &str) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "constant of type `{source_type}` is not assignable to `{}` in the current subset",
                display_type(target_type)
            ),
        }
    }

    pub(super) fn const_representability_error(
        &self,
        target_type: &str,
        value: i64,
    ) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "constant {value} is not representable as `{}` in the current subset",
                display_type(target_type)
            ),
        }
    }

    pub(super) fn unsupported_const_expr<T>(&self) -> Result<T, CompileError> {
        Err(CompileError::Unsupported {
            detail: "const initializers currently require supported constant expressions".into(),
        })
    }

    pub(super) fn unsupported_const_assignment<T>(
        &self,
        target_type: &str,
        value: &ConstValue,
    ) -> Result<T, CompileError> {
        Err(CompileError::Unsupported {
            detail: format!(
                "constant of type `{}` is not assignable to `{}` in the current subset",
                const_value_type_name(value),
                display_type(target_type)
            ),
        })
    }
}

fn const_result_type_for_op(op: BinaryOp, result_type: Option<String>) -> Option<String> {
    match op {
        BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
        | BinaryOp::And
        | BinaryOp::Or => None,
        _ => result_type,
    }
}

fn compare_ordering(ordering: std::cmp::Ordering, op: BinaryOp) -> bool {
    match op {
        BinaryOp::Equal => ordering == std::cmp::Ordering::Equal,
        BinaryOp::NotEqual => ordering != std::cmp::Ordering::Equal,
        BinaryOp::Less => ordering == std::cmp::Ordering::Less,
        BinaryOp::LessEqual => ordering != std::cmp::Ordering::Greater,
        BinaryOp::Greater => ordering == std::cmp::Ordering::Greater,
        BinaryOp::GreaterEqual => ordering != std::cmp::Ordering::Less,
        _ => unreachable!("non-comparison op routed through const compare"),
    }
}

fn stdlib_const_value(value: &StdlibConstantValue) -> Option<ConstValue> {
    match value {
        StdlibConstantValue::Int(value) => Some(ConstValue::Int(*value)),
        StdlibConstantValue::Float(value) => Some(ConstValue::Float(value.0.to_bits())),
        StdlibConstantValue::Bool(value) => Some(ConstValue::Bool(*value)),
        StdlibConstantValue::String(value) => Some(ConstValue::String((*value).to_string())),
        StdlibConstantValue::Error(_) => None,
    }
}

fn const_value_type_name(value: &ConstValue) -> &'static str {
    match value {
        ConstValue::Int(_) => "int",
        ConstValue::Float(_) => "float64",
        ConstValue::Bool(_) => "bool",
        ConstValue::String(_) => "string",
    }
}

fn exact_float_from_int(value: i64) -> Option<u64> {
    let float = value as f64;
    ((float as i64) == value).then_some(float.to_bits())
}
