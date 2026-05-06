use super::*;
use crate::const_eval::{ConstValue, ConstValueInfo};

pub(super) fn infer_package_const_value(
    expr: &Expr,
    const_values: &HashMap<String, ConstValueInfo>,
    imported_packages: &HashMap<String, String>,
) -> Result<ConstValueInfo, CompileError> {
    match expr {
        Expr::IntLiteral(value) => Ok(ConstValueInfo::untyped(ConstValue::Int(*value))),
        Expr::FloatLiteral(bits) => Ok(ConstValueInfo::untyped(ConstValue::Float(*bits))),
        Expr::BoolLiteral(value) => Ok(ConstValueInfo::untyped(ConstValue::Bool(*value))),
        Expr::StringLiteral(value) => {
            Ok(ConstValueInfo::untyped(ConstValue::String(value.clone())))
        }
        Expr::Ident(name) => const_values.get(name).cloned().ok_or_else(|| {
            CompileError::Unsupported {
                detail: format!(
                    "package const initializers can only reference earlier consts in the current subset: `{name}`"
                ),
            }
        }),
        Expr::Selector { receiver, field } => {
            let Expr::Ident(receiver_name) = receiver.as_ref() else {
                return Err(CompileError::Unsupported {
                    detail: "const initializers currently require supported constant expressions"
                        .into(),
                });
            };
            let Some(package_path) = imported_packages.get(receiver_name) else {
                return Err(CompileError::Unsupported {
                    detail: "const initializers currently require supported constant expressions"
                        .into(),
                });
            };
            if let Some(constant) = resolve_stdlib_constant(package_path, field) {
                if let Some(value) = stdlib_const_value(&constant.value) {
                    return Ok(ConstValueInfo::typed(value, constant.typ.to_string()));
                }
                return Err(CompileError::Unsupported {
                    detail: "const initializers currently require supported constant expressions"
                        .into(),
                });
            }
            if let Some(value) = resolve_stdlib_value(package_path, field) {
                if let StdlibValueInit::Constant(constant) = value.value {
                    if let Some(constant) = stdlib_const_value(&constant) {
                        return Ok(ConstValueInfo::typed(constant, value.typ.to_string()));
                    }
                    return Err(CompileError::Unsupported {
                        detail:
                            "const initializers currently require supported constant expressions"
                                .into(),
                    });
                }
            }
            Err(CompileError::Unsupported {
                detail: "const initializers currently require supported constant expressions"
                    .into(),
            })
        }
        Expr::Unary { op, expr } => {
            let inner = infer_package_const_value(expr, const_values, imported_packages)?;
            let value = match (&inner.value, op) {
                (ConstValue::Bool(value), UnaryOp::Not) => ConstValue::Bool(!value),
                (ConstValue::Int(value), UnaryOp::Negate) => {
                    ConstValue::Int(value.checked_neg().ok_or_else(|| CompileError::Unsupported {
                        detail: "constant expression overflows supported int range".into(),
                    })?)
                }
                (ConstValue::Float(bits), UnaryOp::Negate) => {
                    ConstValue::Float((-f64::from_bits(*bits)).to_bits())
                }
                (ConstValue::Int(value), UnaryOp::BitNot) => ConstValue::Int(!value),
                _ => {
                    return Err(CompileError::Unsupported {
                        detail: "const initializers currently require supported constant expressions"
                            .into(),
                    });
                }
            };
            Ok(ConstValueInfo {
                value,
                typ: inner.typ,
            })
        }
        Expr::Binary { left, op, right } => {
            let left = infer_package_const_value(left, const_values, imported_packages)?;
            let right = infer_package_const_value(right, const_values, imported_packages)?;
            eval_package_const_binary(left, right, *op)
        }
        _ => Err(CompileError::Unsupported {
            detail: "const initializers currently require supported constant expressions".into(),
        }),
    }
}

pub(super) fn coerce_package_const_value(
    value: &ConstValueInfo,
    target_type: &str,
) -> Result<ConstValueInfo, CompileError> {
    let coerced = match (value.value.clone(), target_type) {
        (ConstValue::Int(value), "int") => ConstValue::Int(value),
        (ConstValue::Int(value), "byte") => {
            if !(0..=255).contains(&value) {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "constant {value} is not representable as `{target_type}` in the current subset"
                    ),
                });
            }
            ConstValue::Int(value)
        }
        (ConstValue::Int(value), "rune") => {
            if !((i32::MIN as i64)..=(i32::MAX as i64)).contains(&value) {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "constant {value} is not representable as `{target_type}` in the current subset"
                    ),
                });
            }
            ConstValue::Int(value)
        }
        (ConstValue::Int(value), "float64") => {
            let float = value as f64;
            if (float as i64) != value {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "constant {value} is not representable as `{target_type}` in the current subset"
                    ),
                });
            }
            ConstValue::Float(float.to_bits())
        }
        (ConstValue::Float(bits), "float64") => ConstValue::Float(bits),
        (ConstValue::Float(bits), "int" | "byte" | "rune") => {
            let float = f64::from_bits(bits);
            if !float.is_finite()
                || float.fract() != 0.0
                || (float as i64 as f64).to_bits() != float.to_bits()
            {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "constant of type `float64` is not assignable to `{target_type}` in the current subset"
                    ),
                });
            }
            return coerce_package_const_value(
                &ConstValueInfo::untyped(ConstValue::Int(float as i64)),
                target_type,
            );
        }
        (ConstValue::Bool(value), "bool") => ConstValue::Bool(value),
        (ConstValue::String(value), "string") => ConstValue::String(value),
        (value, _) => {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "constant of type `{}` is not assignable to `{target_type}` in the current subset",
                    match value {
                        ConstValue::Int(_) => "int",
                        ConstValue::Float(_) => "float64",
                        ConstValue::Bool(_) => "bool",
                        ConstValue::String(_) => "string",
                    }
                ),
            });
        }
    };

    Ok(ConstValueInfo::typed(coerced, target_type))
}

fn eval_package_const_binary(
    left: ConstValueInfo,
    right: ConstValueInfo,
    op: BinaryOp,
) -> Result<ConstValueInfo, CompileError> {
    let value = match (&left.value, &right.value, op) {
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::Add) => {
            ConstValue::String(format!("{left}{right}"))
        }
        (ConstValue::Bool(left), ConstValue::Bool(right), BinaryOp::And) => {
            ConstValue::Bool(*left && *right)
        }
        (ConstValue::Bool(left), ConstValue::Bool(right), BinaryOp::Or) => {
            ConstValue::Bool(*left || *right)
        }
        (ConstValue::Bool(left), ConstValue::Bool(right), BinaryOp::Equal) => {
            ConstValue::Bool(*left == *right)
        }
        (ConstValue::Bool(left), ConstValue::Bool(right), BinaryOp::NotEqual) => {
            ConstValue::Bool(*left != *right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::Equal) => {
            ConstValue::Bool(left == right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::NotEqual) => {
            ConstValue::Bool(left != right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::Less) => {
            ConstValue::Bool(left < right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::LessEqual) => {
            ConstValue::Bool(left <= right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::Greater) => {
            ConstValue::Bool(left > right)
        }
        (ConstValue::String(left), ConstValue::String(right), BinaryOp::GreaterEqual) => {
            ConstValue::Bool(left >= right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Add) => ConstValue::Int(
            left.checked_add(*right)
                .ok_or_else(|| CompileError::Unsupported {
                    detail: "constant expression overflows supported int range".into(),
                })?,
        ),
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Subtract) => ConstValue::Int(
            left.checked_sub(*right)
                .ok_or_else(|| CompileError::Unsupported {
                    detail: "constant expression overflows supported int range".into(),
                })?,
        ),
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Multiply) => ConstValue::Int(
            left.checked_mul(*right)
                .ok_or_else(|| CompileError::Unsupported {
                    detail: "constant expression overflows supported int range".into(),
                })?,
        ),
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Divide) => {
            if *right == 0 {
                return Err(CompileError::Unsupported {
                    detail: "constant division by zero".into(),
                });
            }
            ConstValue::Int(left / right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Modulo) => {
            if *right == 0 {
                return Err(CompileError::Unsupported {
                    detail: "constant modulo by zero".into(),
                });
            }
            ConstValue::Int(left % right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::ShiftLeft) => {
            let shift = u32::try_from(*right).map_err(|_| CompileError::Unsupported {
                detail: "shift count must be non-negative".into(),
            })?;
            let shifted = if shift >= 127 {
                None
            } else {
                let shifted = (i128::from(*left)) << shift;
                i64::try_from(shifted).ok()
            };
            ConstValue::Int(shifted.ok_or_else(|| CompileError::Unsupported {
                detail: "constant expression overflows supported int range".into(),
            })?)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::ShiftRight) => {
            let shift = u32::try_from(*right).map_err(|_| CompileError::Unsupported {
                detail: "shift count must be non-negative".into(),
            })?;
            ConstValue::Int(if shift >= 63 {
                if *left >= 0 {
                    0
                } else {
                    -1
                }
            } else {
                left >> shift
            })
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::BitOr) => {
            ConstValue::Int(*left | *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::BitXor) => {
            ConstValue::Int(*left ^ *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::BitAnd) => {
            ConstValue::Int(*left & *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::BitClear) => {
            ConstValue::Int(*left & !*right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Equal) => {
            ConstValue::Bool(*left == *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::NotEqual) => {
            ConstValue::Bool(*left != *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Less) => {
            ConstValue::Bool(*left < *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::LessEqual) => {
            ConstValue::Bool(*left <= *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::Greater) => {
            ConstValue::Bool(*left > *right)
        }
        (ConstValue::Int(left), ConstValue::Int(right), BinaryOp::GreaterEqual) => {
            ConstValue::Bool(*left >= *right)
        }
        (ConstValue::Float(left), ConstValue::Float(right), BinaryOp::Add) => {
            ConstValue::Float((f64::from_bits(*left) + f64::from_bits(*right)).to_bits())
        }
        (ConstValue::Float(left), ConstValue::Float(right), BinaryOp::Subtract) => {
            ConstValue::Float((f64::from_bits(*left) - f64::from_bits(*right)).to_bits())
        }
        (ConstValue::Float(left), ConstValue::Float(right), BinaryOp::Multiply) => {
            ConstValue::Float((f64::from_bits(*left) * f64::from_bits(*right)).to_bits())
        }
        (ConstValue::Float(left), ConstValue::Float(right), BinaryOp::Divide) => {
            ConstValue::Float((f64::from_bits(*left) / f64::from_bits(*right)).to_bits())
        }
        (ConstValue::Int(left), ConstValue::Float(right_bits), op)
            if matches!(
                op,
                BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
            ) =>
        {
            let left = exact_float_from_int(*left).ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "constant {left} is not representable as `float64` in the current subset"
                ),
            })?;
            return eval_package_const_binary(
                ConstValueInfo::untyped(ConstValue::Float(left)),
                ConstValueInfo::untyped(ConstValue::Float(*right_bits)),
                op,
            );
        }
        (ConstValue::Float(left_bits), ConstValue::Int(right), op)
            if matches!(
                op,
                BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
            ) =>
        {
            let right = exact_float_from_int(*right).ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "constant {right} is not representable as `float64` in the current subset"
                ),
            })?;
            return eval_package_const_binary(
                ConstValueInfo::untyped(ConstValue::Float(*left_bits)),
                ConstValueInfo::untyped(ConstValue::Float(right)),
                op,
            );
        }
        _ => {
            return Err(CompileError::Unsupported {
                detail: "const initializers currently require supported constant expressions"
                    .into(),
            });
        }
    };

    Ok(ConstValueInfo {
        value,
        typ: match op {
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::And
            | BinaryOp::Or => None,
            _ => left.typ.or(right.typ),
        },
    })
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

fn exact_float_from_int(value: i64) -> Option<u64> {
    let float = value as f64;
    ((float as i64) == value).then_some(float.to_bits())
}
