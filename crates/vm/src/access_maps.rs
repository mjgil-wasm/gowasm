use super::*;
use crate::map_value::map_key_is_comparable;

impl Vm {
    fn map_key_type_name(&self, program: &Program, key: &Value) -> String {
        value_runtime_type(program, self, key)
            .map(|info| info.display_name)
            .unwrap_or_else(|| match &key.data {
                ValueData::Nil => "nil".into(),
                ValueData::Int(_) => "int".into(),
                ValueData::Float(_) => "float64".into(),
                ValueData::String(_) => "string".into(),
                ValueData::Bool(_) => "bool".into(),
                ValueData::Error(_) => "error".into(),
                ValueData::Array(array) => format!("[{}]<unknown>", array.len()),
                ValueData::Slice(_) => "[]<unknown>".into(),
                ValueData::Map(_) => "map[<unknown>]<unknown>".into(),
                ValueData::Channel(_) => "chan <unknown>".into(),
                ValueData::Pointer(_) => "*<unknown>".into(),
                ValueData::Function(_) => "func".into(),
                ValueData::Struct(_) => "struct".into(),
            })
    }

    pub(crate) fn validate_map_key_value(
        &self,
        program: &Program,
        key: &Value,
    ) -> Result<(), VmError> {
        if map_key_is_comparable(key) {
            return Ok(());
        }
        Err(VmError::UnhandledPanic {
            function: self.current_function_name(program)?,
            value: format!(
                "hash of unhashable type {}",
                self.map_key_type_name(program, key)
            ),
        })
    }

    pub(super) fn map_value(
        &self,
        program: &Program,
        entries: &[(usize, usize)],
        zero: usize,
        concrete_type: Option<&ConcreteType>,
    ) -> Result<Value, VmError> {
        let mut values = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let key = self.read_register(program, *key)?;
            self.validate_map_key_value(program, &key)?;
            values.push((key, self.read_register(program, *value)?));
        }
        let zero_value = self.read_register(program, zero)?;
        Ok(match concrete_type.cloned() {
            Some(concrete_type) => Value::map_typed(values, zero_value, concrete_type),
            None => Value::map(values, zero_value),
        })
    }

    pub(super) fn index_value(
        &self,
        program: &Program,
        target: &Value,
        index: &Value,
    ) -> Result<Value, VmError> {
        let function = self.current_function_name(program)?;
        match &target.data {
            ValueData::Array(array) => {
                let ValueData::Int(index) = index.data else {
                    return Err(VmError::InvalidIndexValue { function });
                };

                if index < 0 {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index,
                        len: 0,
                    });
                }

                let index = index as usize;
                array.get(index).ok_or(VmError::IndexOutOfBounds {
                    function: self.current_function_name(program)?,
                    index: index as i64,
                    len: array.len(),
                })
            }
            ValueData::Slice(slice) => {
                let ValueData::Int(index) = index.data else {
                    return Err(VmError::InvalidIndexValue { function });
                };

                if index < 0 {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index,
                        len: 0,
                    });
                }

                let index = index as usize;
                slice.get(index).ok_or(VmError::IndexOutOfBounds {
                    function: self.current_function_name(program)?,
                    index: index as i64,
                    len: slice.len(),
                })
            }
            ValueData::String(text) => {
                let ValueData::Int(index) = index.data else {
                    return Err(VmError::InvalidIndexValue { function });
                };

                if index < 0 {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index,
                        len: 0,
                    });
                }

                let index = index as usize;
                text.as_bytes()
                    .get(index)
                    .map(|value| Value::int(*value as i64))
                    .ok_or(VmError::IndexOutOfBounds {
                        function: self.current_function_name(program)?,
                        index: index as i64,
                        len: text.len(),
                    })
            }
            ValueData::Map(map) => {
                self.validate_map_key_value(program, index)?;
                Ok(map.get(index).unwrap_or_else(|| (*map.zero_value).clone()))
            }
            _ => Err(VmError::InvalidIndexTarget {
                function,
                target: describe_value(target),
            }),
        }
    }

    pub(super) fn map_contains(
        &self,
        program: &Program,
        target: &Value,
        index: &Value,
    ) -> Result<bool, VmError> {
        match &target.data {
            ValueData::Map(map) => {
                self.validate_map_key_value(program, index)?;
                Ok(map.contains_key(index))
            }
            _ => Err(VmError::InvalidMapLookupTarget {
                function: self.current_function_name(program)?,
                target: describe_value(target),
            }),
        }
    }
}
