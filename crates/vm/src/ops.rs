use super::{
    describe_value, explicit_concrete_type_for_value, program_type_inventory, CompareOp,
    ConcreteType, Float64, Program, RuntimeTypeKind, Value, ValueData, Vm, VmError, TYPE_ANY,
    TYPE_ERROR,
};
use crate::value::{compare_eq, compare_op_symbol, compare_ord};

impl Vm {
    pub(super) fn add_values(
        &self,
        program: &Program,
        left: &Value,
        right: &Value,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Int(left), ValueData::Int(right)) => {
                Ok(Value::int(left.wrapping_add(*right)))
            }
            (ValueData::Float(Float64(left)), ValueData::Float(Float64(right))) => {
                Ok(Value::float(left + right))
            }
            (ValueData::String(left), ValueData::String(right)) => {
                Ok(Value::string(format!("{left}{right}")))
            }
            _ => Err(VmError::InvalidAddOperands {
                function: self.current_function_name(program)?,
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn not_value(&self, program: &Program, value: &Value) -> Result<Value, VmError> {
        match value.data {
            ValueData::Bool(value) => Ok(Value::bool(!value)),
            _ => Err(VmError::InvalidNotOperand {
                function: self.current_function_name(program)?,
                operand: describe_value(value),
            }),
        }
    }

    pub(super) fn negate_value(&self, program: &Program, value: &Value) -> Result<Value, VmError> {
        match value.data {
            ValueData::Int(value) => Ok(Value::int(value.wrapping_neg())),
            ValueData::Float(Float64(value)) => Ok(Value::float(-value)),
            _ => Err(VmError::InvalidNegateOperand {
                function: self.current_function_name(program)?,
                operand: describe_value(value),
            }),
        }
    }

    pub(super) fn bit_not_value(&self, program: &Program, value: &Value) -> Result<Value, VmError> {
        match value.data {
            ValueData::Int(value) => Ok(Value::int(!value)),
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: "^".into(),
                left: describe_value(value),
                right: "missing operand".into(),
            }),
        }
    }

    pub(super) fn int_arithmetic(
        &self,
        program: &Program,
        op: &str,
        left: &Value,
        right: &Value,
        apply: impl FnOnce(i64, i64) -> i64,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Int(left), ValueData::Int(right)) => Ok(Value::int(apply(*left, *right))),
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: op.into(),
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn float_arithmetic(
        &self,
        program: &Program,
        op: &str,
        left: &Value,
        right: &Value,
        apply: impl FnOnce(f64, f64) -> f64,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Float(Float64(left)), ValueData::Float(Float64(right))) => {
                Ok(Value::float(apply(*left, *right)))
            }
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: op.into(),
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn divide_values(
        &self,
        program: &Program,
        left: &Value,
        right: &Value,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Int(_), ValueData::Int(0)) => Err(VmError::DivisionByZero {
                function: self.current_function_name(program)?,
                left: describe_value(left),
                right: describe_value(right),
            }),
            (ValueData::Int(left), ValueData::Int(right)) => Ok(Value::int(left / right)),
            (ValueData::Float(Float64(left)), ValueData::Float(Float64(right))) => {
                Ok(Value::float(left / right))
            }
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: "/".into(),
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn shift_left_values(
        &self,
        program: &Program,
        left: &Value,
        right: &Value,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Int(left), ValueData::Int(right)) if *right >= 0 => {
                Ok(Value::int(left.wrapping_shl(*right as u32)))
            }
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: "<<".into(),
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn shift_right_values(
        &self,
        program: &Program,
        left: &Value,
        right: &Value,
    ) -> Result<Value, VmError> {
        match (&left.data, &right.data) {
            (ValueData::Int(left), ValueData::Int(right)) if *right >= 0 => {
                Ok(Value::int(left.wrapping_shr(*right as u32)))
            }
            _ => Err(VmError::InvalidArithmeticOperands {
                function: self.current_function_name(program)?,
                op: ">>".into(),
                left: describe_value(left),
                right: describe_value(right),
            }),
        }
    }

    pub(super) fn execute_int_instruction(
        &mut self,
        program: &Program,
        dst: usize,
        left: usize,
        right: usize,
        op: &str,
    ) -> Result<(), VmError> {
        let left = self.read_register(program, left)?;
        let right = self.read_register(program, right)?;
        let result = match op {
            "-" => self
                .int_arithmetic(program, op, &left, &right, |l, r| l.wrapping_sub(r))
                .or_else(|_| self.float_arithmetic(program, op, &left, &right, |l, r| l - r))?,
            "*" => self
                .int_arithmetic(program, op, &left, &right, |l, r| l.wrapping_mul(r))
                .or_else(|_| self.float_arithmetic(program, op, &left, &right, |l, r| l * r))?,
            "%" => self
                .int_arithmetic(program, op, &left, &right, |l, r| l.wrapping_rem(r))
                .or_else(|_| self.float_arithmetic(program, op, &left, &right, |l, r| l % r))?,
            "^" => self.int_arithmetic(program, op, &left, &right, |l, r| l ^ r)?,
            "&" => self.int_arithmetic(program, op, &left, &right, |l, r| l & r)?,
            "&^" => self.int_arithmetic(program, op, &left, &right, |l, r| l & !r)?,
            "|" => self.int_arithmetic(program, op, &left, &right, |l, r| l | r)?,
            "/" => self.divide_values(program, &left, &right)?,
            "<<" => self.shift_left_values(program, &left, &right)?,
            ">>" => self.shift_right_values(program, &left, &right)?,
            _ => unreachable!("unexpected integer instruction op"),
        };
        self.set_register(program, dst, result)?;
        Ok(())
    }

    pub(super) fn compare_values(
        &self,
        program: &Program,
        op: CompareOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, VmError> {
        let function = self.current_function_name(program)?;
        if let Some(result) =
            self.try_compare_interface_wrapped_values(program, op, left, right, &function)?
        {
            return Ok(Value::bool(result));
        }
        let result = if matches!(&left.data, ValueData::Nil)
            && interface_wrapper_type(program, right.typ)
        {
            compare_eq(matches!(&right.data, ValueData::Nil), op)
        } else if matches!(&right.data, ValueData::Nil) && interface_wrapper_type(program, left.typ)
        {
            compare_eq(matches!(&left.data, ValueData::Nil), op)
        } else {
            match (&left.data, &right.data) {
                (ValueData::Int(left), ValueData::Int(right)) => compare_ord(left.cmp(right), op),
                (ValueData::Float(Float64(left)), ValueData::Float(Float64(right))) => {
                    left.partial_cmp(right).and_then(|ord| compare_ord(ord, op))
                }
                (ValueData::String(left), ValueData::String(right)) => {
                    compare_ord(left.cmp(right), op)
                }
                (ValueData::Bool(left), ValueData::Bool(right)) => compare_eq(*left == *right, op),
                (ValueData::Nil, ValueData::Nil) => compare_eq(true, op),
                (ValueData::Array(left), ValueData::Array(right)) => compare_eq(left == right, op),
                (ValueData::Nil, ValueData::Slice(slice))
                | (ValueData::Slice(slice), ValueData::Nil) => compare_eq(slice.is_nil, op),
                (ValueData::Slice(left), ValueData::Slice(right)) => {
                    compare_eq(left.is_nil && right.is_nil, op)
                }
                (ValueData::Nil, ValueData::Map(map)) | (ValueData::Map(map), ValueData::Nil) => {
                    compare_eq(map.entries.is_none(), op)
                }
                (ValueData::Nil, ValueData::Channel(channel))
                | (ValueData::Channel(channel), ValueData::Nil) => compare_eq(channel.is_nil(), op),
                (ValueData::Channel(left), ValueData::Channel(right)) => {
                    compare_eq(left == right, op)
                }
                (ValueData::Nil, ValueData::Function(_))
                | (ValueData::Function(_), ValueData::Nil) => compare_eq(false, op),
                (ValueData::Nil, ValueData::Error(_)) | (ValueData::Error(_), ValueData::Nil) => {
                    compare_eq(false, op)
                }
                (ValueData::Error(left), ValueData::Error(right)) => {
                    compare_eq(left.message == right.message, op)
                }
                (ValueData::Nil, ValueData::Pointer(pointer))
                | (ValueData::Pointer(pointer), ValueData::Nil) => compare_eq(pointer.is_nil(), op),
                (ValueData::Pointer(left), ValueData::Pointer(right)) => {
                    compare_eq(left == right, op)
                }
                (ValueData::Struct(left), ValueData::Struct(right)) => {
                    compare_eq(left == right, op)
                }
                _ => {
                    return Err(VmError::InvalidComparisonOperands {
                        function,
                        op: compare_op_symbol(op).into(),
                        left: describe_value(left),
                        right: describe_value(right),
                    });
                }
            }
        };

        result
            .map(Value::bool)
            .ok_or_else(|| VmError::InvalidComparisonOperands {
                function: function.clone(),
                op: compare_op_symbol(op).into(),
                left: describe_value(left),
                right: describe_value(right),
            })
    }

    fn try_compare_interface_wrapped_values(
        &self,
        program: &Program,
        op: CompareOp,
        left: &Value,
        right: &Value,
        function: &str,
    ) -> Result<Option<bool>, VmError> {
        if !matches!(op, CompareOp::Equal | CompareOp::NotEqual) {
            return Ok(None);
        }

        let left_interface = interface_wrapper_type(program, left.typ);
        let right_interface = interface_wrapper_type(program, right.typ);
        if !left_interface && !right_interface {
            return Ok(None);
        }

        if matches!(&left.data, ValueData::Nil) || matches!(&right.data, ValueData::Nil) {
            return Ok(None);
        }

        let left_concrete = explicit_concrete_type_for_value(left);
        let right_concrete = explicit_concrete_type_for_value(right);
        if left_concrete != right_concrete {
            return Ok(compare_eq(false, op));
        }

        if (left_interface || right_interface) && !dynamic_value_is_comparable(left) {
            return Err(VmError::InvalidComparisonOperands {
                function: function.into(),
                op: compare_op_symbol(op).into(),
                left: describe_value(left),
                right: describe_value(right),
            });
        }

        Ok(None)
    }
}

fn interface_wrapper_type(program: &Program, typ: crate::TypeId) -> bool {
    typ == TYPE_ANY
        || typ == TYPE_ERROR
        || program_type_inventory(program)
            .and_then(|inventory| inventory.type_info_for_type_id(typ))
            .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

fn dynamic_value_is_comparable(value: &Value) -> bool {
    match &value.data {
        ValueData::Nil
        | ValueData::Int(_)
        | ValueData::Float(_)
        | ValueData::String(_)
        | ValueData::Bool(_)
        | ValueData::Error(_)
        | ValueData::Pointer(_)
        | ValueData::Channel(_) => true,
        ValueData::Array(array) => array
            .values_snapshot()
            .iter()
            .all(dynamic_value_is_comparable),
        ValueData::Struct(fields) => fields
            .iter()
            .all(|(_, field_value)| dynamic_value_is_comparable(field_value)),
        ValueData::Slice(_) | ValueData::Map(_) | ValueData::Function(_) => false,
    }
}

pub(crate) fn function_name(program: &Program, function_id: usize) -> Result<String, VmError> {
    program
        .functions
        .get(function_id)
        .map(|function| function.name.clone())
        .ok_or(VmError::UnknownFunction {
            function: function_id,
        })
}

impl Vm {
    pub(super) fn execute_convert_to_int(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::Int(_) => value,
            ValueData::Float(f) => Value::int(f.0.trunc() as i64),
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "int-convertible value".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_to_float64(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::Float(_) => value,
            ValueData::Int(n) => Value::float(*n as f64),
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "float64-convertible value".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_to_string(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::String(_) => value,
            ValueData::Int(n) => {
                let ch = char::from_u32(*n as u32).unwrap_or('\u{FFFD}');
                Value::string(ch.to_string())
            }
            ValueData::Slice(slice) => {
                let values = slice.values_snapshot();
                let mut bytes = Vec::with_capacity(values.len());
                for v in &values {
                    if let ValueData::Int(b) = &v.data {
                        bytes.push(*b as u8);
                    }
                }
                Value::string(String::from_utf8_lossy(&bytes).into_owned())
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "string-convertible value".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_to_byte(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::Int(n) => Value::int((*n as u8) as i64),
            ValueData::Float(f) => Value::int((f.0.trunc() as i64 as u8) as i64),
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "byte-convertible value".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_to_byte_slice(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::String(s) => {
                let bytes: Vec<Value> = s.bytes().map(|b| Value::int(b as i64)).collect();
                Value::slice_typed(
                    bytes,
                    ConcreteType::Slice {
                        element: Box::new(ConcreteType::TypeId(crate::TYPE_INT)),
                    },
                )
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "string for []byte conversion".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_to_rune_slice(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::String(s) => {
                let runes: Vec<Value> = s.chars().map(|ch| Value::int(ch as i64)).collect();
                Value::slice_typed(
                    runes,
                    ConcreteType::Slice {
                        element: Box::new(ConcreteType::TypeId(crate::TYPE_INT)),
                    },
                )
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "string for []rune conversion".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }

    pub(super) fn execute_convert_rune_slice_to_string(
        &mut self,
        program: &Program,
        dst: usize,
        src: usize,
    ) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        let result = match &value.data {
            ValueData::Slice(slice) => {
                let mut text = String::new();
                for value in slice.values_snapshot() {
                    let ValueData::Int(rune) = value.data else {
                        return Err(VmError::TypeMismatch {
                            function: self.current_function_name(program)?,
                            expected: "[]rune-convertible value".into(),
                        });
                    };
                    let ch = char::from_u32(rune as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
                    text.push(ch);
                }
                Value::string(text)
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    function: self.current_function_name(program)?,
                    expected: "[]rune for string conversion".into(),
                });
            }
        };
        self.set_register(program, dst, result)
    }
}
