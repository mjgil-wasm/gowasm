use super::*;
use crate::value::{set_index_value, set_struct_field, SetFieldError, SetIndexError};

impl Vm {
    pub(super) fn method_dispatch_type(&self, program: &Program, value: &Value) -> TypeId {
        if !interface_wrapper_type(program, value.typ) {
            return value.typ;
        }
        match concrete_type_for_value(self, program, value) {
            Some(ConcreteType::TypeId(type_id)) => type_id,
            _ => value.typ,
        }
    }

    pub(super) fn value_matches_type(
        &self,
        program: &Program,
        value: &Value,
        target: &TypeCheck,
    ) -> bool {
        match target {
            TypeCheck::Int => exact_runtime_type_id(self, program, value) == Some(TYPE_INT),
            TypeCheck::Float64 => exact_runtime_type_id(self, program, value) == Some(TYPE_FLOAT64),
            TypeCheck::String => exact_runtime_type_id(self, program, value) == Some(TYPE_STRING),
            TypeCheck::Bool => exact_runtime_type_id(self, program, value) == Some(TYPE_BOOL),
            TypeCheck::Exact { type_id, .. } => {
                exact_runtime_type_id(self, program, value) == Some(*type_id)
            }
            TypeCheck::Interface { methods, .. } => {
                if interface_wrapper_type(program, value.typ)
                    && matches!(value.data, ValueData::Nil)
                {
                    return false;
                }
                let receiver_type = self.method_dispatch_type(program, value);
                methods.iter().all(|method| {
                    program.methods.iter().any(|binding| {
                        binding.receiver_type == receiver_type
                            && method_binding_matches_interface_check(binding, method)
                    }) || resolve_stdlib_runtime_method(receiver_type, &method.name).is_some_and(
                        |function| stdlib_method_matches_interface_check(function, method),
                    )
                })
            }
            TypeCheck::Struct { type_id, .. } => {
                exact_runtime_type_id(self, program, value) == Some(*type_id)
            }
        }
    }

    pub(super) fn read_global(&self, global: usize) -> Result<Value, VmError> {
        self.globals
            .get(global)
            .cloned()
            .ok_or(VmError::InvalidGlobal { global })
    }

    pub(super) fn pointer_to_local(&self, register: usize, typ: TypeId) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::Local {
                    frame_id: self.current_frame().id,
                    register,
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn box_heap_value(&mut self, value: Value, typ: TypeId) -> Value {
        let cell = if let Some(cell) = self.free_heap_cells.pop() {
            self.heap_cells[cell] = Some(value);
            cell
        } else {
            let cell = self.heap_cells.len();
            self.heap_cells.push(Some(value));
            cell
        };
        self.record_heap_allocation();
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::HeapCell { cell },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn pointer_to_global(&self, global: usize, typ: TypeId) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::Global { global },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn project_field_pointer(
        &self,
        program: &Program,
        pointer: &Value,
        field: &str,
        typ: TypeId,
    ) -> Result<Value, VmError> {
        let ValueData::Pointer(pointer) = &pointer.data else {
            return Err(VmError::InvalidPointerProjectionTarget {
                function: self.current_function_name(program)?,
            });
        };

        if matches!(&pointer.target, PointerTarget::Nil) {
            return Err(VmError::NilPointerDereference {
                function: self.current_function_name(program)?,
            });
        }

        Ok(Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::ProjectedField {
                    base: Box::new(pointer.target.clone()),
                    field: field.into(),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        })
    }

    pub(super) fn project_index_pointer(
        &self,
        program: &Program,
        pointer: &Value,
        index: Value,
        typ: TypeId,
    ) -> Result<Value, VmError> {
        let ValueData::Pointer(pointer) = &pointer.data else {
            return Err(VmError::InvalidPointerProjectionTarget {
                function: self.current_function_name(program)?,
            });
        };

        if matches!(&pointer.target, PointerTarget::Nil) {
            return Err(VmError::NilPointerDereference {
                function: self.current_function_name(program)?,
            });
        }

        Ok(Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::ProjectedIndex {
                    base: Box::new(pointer.target.clone()),
                    index: Box::new(index),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        })
    }

    pub(super) fn pointer_to_local_field(
        &self,
        register: usize,
        field: &str,
        typ: TypeId,
    ) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::LocalField {
                    frame_id: self.current_frame().id,
                    register,
                    field: field.into(),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn pointer_to_global_field(&self, global: usize, field: &str, typ: TypeId) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::GlobalField {
                    global,
                    field: field.into(),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn pointer_to_local_index(
        &self,
        register: usize,
        index: Value,
        typ: TypeId,
    ) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::LocalIndex {
                    frame_id: self.current_frame().id,
                    register,
                    index: Box::new(index),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn pointer_to_global_index(
        &self,
        global: usize,
        index: Value,
        typ: TypeId,
    ) -> Value {
        Value {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::GlobalIndex {
                    global,
                    index: Box::new(index),
                },
                concrete_type: pointer_concrete_type(typ),
            }),
        }
    }

    pub(super) fn set_global(&mut self, global: usize, value: Value) -> Result<(), VmError> {
        let slot = self
            .globals
            .get_mut(global)
            .ok_or(VmError::InvalidGlobal { global })?;
        *slot = value;
        Ok(())
    }

    pub(super) fn set_field_on_register(
        &mut self,
        program: &Program,
        register: usize,
        field: &str,
        value: Value,
    ) -> Result<(), VmError> {
        let frame = self.current_frame_mut();
        let function_name = function_name(program, frame.function)?;
        let slot = frame
            .registers
            .get_mut(register)
            .ok_or(VmError::InvalidRegister {
                function: function_name.clone(),
                register,
            })?;
        let target = describe_value(slot);
        match set_struct_field(slot, field, value) {
            Ok(()) => Ok(()),
            Err(SetFieldError::InvalidTarget) => Err(VmError::InvalidFieldTarget {
                function: function_name,
                target,
            }),
            Err(SetFieldError::UnknownField) => Err(VmError::UnknownField {
                function: function_name,
                field: field.into(),
            }),
        }
    }

    pub(super) fn set_index_on_register(
        &mut self,
        program: &Program,
        register: usize,
        index: &Value,
        value: Value,
    ) -> Result<(), VmError> {
        let frame = self.current_frame();
        let map_target = matches!(
            frame.registers.get(register),
            Some(Value {
                data: ValueData::Map(_),
                ..
            })
        );
        let function_name = function_name(program, frame.function)?;
        if map_target {
            self.validate_map_key_value(program, index)?;
        }
        let frame = self.current_frame_mut();
        let slot = frame
            .registers
            .get_mut(register)
            .ok_or(VmError::InvalidRegister {
                function: function_name.clone(),
                register,
            })?;
        let target = describe_value(slot);
        match set_index_value(slot, index, value) {
            Ok(()) => Ok(()),
            Err(SetIndexError::InvalidTarget) => Err(VmError::InvalidIndexTarget {
                function: function_name,
                target: target.clone(),
            }),
            Err(SetIndexError::InvalidIndexValue) => Err(VmError::InvalidIndexValue {
                function: function_name,
            }),
            Err(SetIndexError::IndexOutOfBounds { index, len }) => Err(VmError::IndexOutOfBounds {
                function: function_name,
                index,
                len,
            }),
            Err(SetIndexError::NilMap) => Err(VmError::AssignToNilMap {
                function: function_name,
                target,
            }),
        }
    }

    pub(super) fn read_register_list(
        &self,
        program: &Program,
        registers: &[usize],
    ) -> Result<Vec<Value>, VmError> {
        registers
            .iter()
            .map(|register| self.read_register(program, *register))
            .collect()
    }

    pub(crate) fn deref_pointer(
        &self,
        program: &Program,
        pointer: &Value,
    ) -> Result<Value, VmError> {
        let ValueData::Pointer(pointer) = &pointer.data else {
            return Err(VmError::InvalidDerefTarget {
                function: self.current_function_name(program)?,
            });
        };

        self.deref_pointer_target(program, &pointer.target)
    }

    fn deref_pointer_target(
        &self,
        program: &Program,
        target: &PointerTarget,
    ) -> Result<Value, VmError> {
        match target {
            PointerTarget::Nil => Err(VmError::NilPointerDereference {
                function: self.current_function_name(program)?,
            }),
            PointerTarget::HeapCell { cell } => self
                .heap_cells
                .get(*cell)
                .and_then(|slot| slot.as_ref())
                .cloned()
                .ok_or(VmError::InvalidIndirectTarget {
                    function: self.current_function_name(program)?,
                }),
            PointerTarget::Global { global } => self.read_global(*global),
            PointerTarget::Local { frame_id, register } => {
                self.read_register_from_frame(program, *frame_id, *register)
            }
            PointerTarget::ProjectedField { base, field } => {
                let target = self.deref_pointer_target(program, base)?;
                self.field_value(program, &target, field)
            }
            PointerTarget::ProjectedIndex { base, index } => {
                let target = self.deref_pointer_target(program, base)?;
                self.index_value(program, &target, index)
            }
            PointerTarget::LocalField {
                frame_id,
                register,
                field,
            } => {
                let target = self.read_register_from_frame(program, *frame_id, *register)?;
                self.field_value(program, &target, field)
            }
            PointerTarget::GlobalField { global, field } => {
                let target = self.read_global(*global)?;
                self.field_value(program, &target, field)
            }
            PointerTarget::LocalIndex {
                frame_id,
                register,
                index,
            } => {
                let target = self.read_register_from_frame(program, *frame_id, *register)?;
                self.index_value(program, &target, index)
            }
            PointerTarget::GlobalIndex { global, index } => {
                let target = self.read_global(*global)?;
                self.index_value(program, &target, index)
            }
        }
    }

    pub(crate) fn store_indirect(
        &mut self,
        program: &Program,
        pointer: &Value,
        value: Value,
    ) -> Result<(), VmError> {
        let ValueData::Pointer(pointer) = &pointer.data else {
            return Err(VmError::InvalidIndirectTarget {
                function: self.current_function_name(program)?,
            });
        };

        self.store_pointer_target(program, &pointer.target, value)
    }

    fn store_pointer_target(
        &mut self,
        program: &Program,
        target: &PointerTarget,
        value: Value,
    ) -> Result<(), VmError> {
        match target {
            PointerTarget::Nil => Err(VmError::NilPointerDereference {
                function: self.current_function_name(program)?,
            }),
            PointerTarget::HeapCell { cell } => {
                let function = self.current_function_name(program)?;
                let slot =
                    self.heap_cells
                        .get_mut(*cell)
                        .ok_or(VmError::InvalidIndirectTarget {
                            function: function.clone(),
                        })?;
                let slot = slot
                    .as_mut()
                    .ok_or(VmError::InvalidIndirectTarget { function })?;
                *slot = value;
                Ok(())
            }
            PointerTarget::Global { global } => self.set_global(*global, value),
            PointerTarget::Local { frame_id, register } => {
                self.set_register_on_frame(program, *frame_id, *register, value)
            }
            PointerTarget::ProjectedField { base, field } => {
                let mut target = self.deref_pointer_target(program, base)?;
                self.apply_field_store(program, &mut target, field, value)?;
                self.store_pointer_target(program, base, target)
            }
            PointerTarget::ProjectedIndex { base, index } => {
                let mut target = self.deref_pointer_target(program, base)?;
                self.apply_index_store(program, &mut target, index, value)?;
                self.store_pointer_target(program, base, target)
            }
            PointerTarget::LocalField {
                frame_id,
                register,
                field,
            } => {
                let mut target = self.read_register_from_frame(program, *frame_id, *register)?;
                self.apply_field_store(program, &mut target, field, value)?;
                self.set_register_on_frame(program, *frame_id, *register, target)
            }
            PointerTarget::GlobalField { global, field } => {
                let mut target = self.read_global(*global)?;
                self.apply_field_store(program, &mut target, field, value)?;
                self.set_global(*global, target)
            }
            PointerTarget::LocalIndex {
                frame_id,
                register,
                index,
            } => {
                let mut target = self.read_register_from_frame(program, *frame_id, *register)?;
                self.apply_index_store(program, &mut target, index, value)?;
                self.set_register_on_frame(program, *frame_id, *register, target)
            }
            PointerTarget::GlobalIndex { global, index } => {
                let mut target = self.read_global(*global)?;
                self.apply_index_store(program, &mut target, index, value)?;
                self.set_global(*global, target)
            }
        }
    }

    pub(super) fn copy_into_register(
        &mut self,
        program: &Program,
        register: usize,
        source: &Value,
    ) -> Result<usize, VmError> {
        let frame = self.current_frame_mut();
        let function_name = function_name(program, frame.function)?;
        let slot = frame
            .registers
            .get_mut(register)
            .ok_or(VmError::InvalidRegister {
                function: function_name.clone(),
                register,
            })?;

        let ValueData::Slice(target_slice) = &mut slot.data else {
            return Err(VmError::InvalidCopyTarget {
                function: function_name,
            });
        };
        let ValueData::Slice(source_slice) = &source.data else {
            return Err(VmError::InvalidCopySource {
                function: self.current_function_name(program)?,
            });
        };

        let source_values = source_slice.values_snapshot();
        let count = target_slice.len().min(source_slice.len());
        let mut target_values = target_slice.values.borrow_mut();
        let write_start = target_slice.start;
        let write_end = write_start + count;
        if write_end > target_values.len() {
            return Ok(0);
        }
        for (offset, source) in source_values.into_iter().take(count).enumerate() {
            target_values[write_start + offset] = source;
        }
        Ok(count)
    }

    pub(super) fn struct_value(
        &self,
        program: &Program,
        typ: TypeId,
        fields: &[(String, usize)],
    ) -> Result<Value, VmError> {
        let mut values = Vec::with_capacity(fields.len());
        for (name, register) in fields {
            values.push((name.clone(), self.read_register(program, *register)?));
        }
        Ok(Value::struct_value(typ, values))
    }

    pub(super) fn slice_value(
        &self,
        program: &Program,
        target: &Value,
        low: Option<&Value>,
        high: Option<&Value>,
    ) -> Result<Value, VmError> {
        let function = self.current_function_name(program)?;
        let to_index = |v: &Value| -> Result<usize, VmError> {
            match v.data {
                ValueData::Int(n) if n >= 0 => Ok(n as usize),
                ValueData::Int(n) => Err(VmError::IndexOutOfBounds {
                    function: function.clone(),
                    index: n,
                    len: 0,
                }),
                _ => Err(VmError::InvalidIndexValue {
                    function: function.clone(),
                }),
            }
        };
        match &target.data {
            ValueData::String(text) => {
                let low = low.map(&to_index).transpose()?.unwrap_or(0);
                let high = high.map(&to_index).transpose()?.unwrap_or(text.len());
                if low > high || high > text.len() {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index: high as i64,
                        len: text.len(),
                    });
                }
                let bytes = text.as_bytes();
                let sliced = &bytes[low..high];
                Ok(Value::string(String::from_utf8_lossy(sliced).into_owned()))
            }
            ValueData::Slice(slice) => {
                let low = low.map(&to_index).transpose()?.unwrap_or(0);
                let high = high.map(&to_index).transpose()?.unwrap_or(slice.len());
                if low > high || high > slice.len() {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index: high as i64,
                        len: slice.len(),
                    });
                }
                Ok(Value {
                    typ: crate::TYPE_SLICE,
                    data: ValueData::Slice(slice.subslice(low, high)),
                })
            }
            ValueData::Array(array) => {
                let low = low.map(&to_index).transpose()?.unwrap_or(0);
                let array_len = array.len();
                let high = high.map(&to_index).transpose()?.unwrap_or(array_len);
                if low > high || high > array_len {
                    return Err(VmError::IndexOutOfBounds {
                        function,
                        index: high as i64,
                        len: array_len,
                    });
                }
                let concrete_type = match &array.concrete_type {
                    Some(ConcreteType::Array { element, .. }) => Some(ConcreteType::Slice {
                        element: element.clone(),
                    }),
                    _ => None,
                };
                Ok(Value {
                    typ: crate::TYPE_SLICE,
                    data: ValueData::Slice(crate::SliceValue {
                        values: array.values.clone(),
                        start: low,
                        len: high - low,
                        cap: array_len - low,
                        is_nil: false,
                        concrete_type,
                    }),
                })
            }
            _ => Err(VmError::InvalidIndexTarget {
                function,
                target: describe_value(target),
            }),
        }
    }

    pub(super) fn field_value(
        &self,
        program: &Program,
        target: &Value,
        field: &str,
    ) -> Result<Value, VmError> {
        if let ValueData::Pointer(pointer) = &target.data {
            if pointer.is_nil() {
                return Err(VmError::NilPointerDereference {
                    function: self.current_function_name(program)?,
                });
            }
            let dereferenced = self.deref_pointer(program, target)?;
            return self.field_value(program, &dereferenced, field);
        }

        let ValueData::Struct(fields) = &target.data else {
            return Err(VmError::InvalidFieldTarget {
                function: self.current_function_name(program)?,
                target: describe_value(target),
            });
        };

        fields
            .iter()
            .find(|(name, _)| name == field)
            .map(|(_, value)| value.clone())
            .ok_or(VmError::UnknownField {
                function: self.current_function_name(program)?,
                field: field.into(),
            })
    }

    pub(super) fn assert_type(
        &self,
        program: &Program,
        value: &Value,
        target: &TypeCheck,
    ) -> Result<Value, VmError> {
        if self.value_matches_type(program, value, target) {
            match target {
                TypeCheck::Exact { type_id, .. } => Ok(narrow_exact_value(value, *type_id)),
                TypeCheck::Struct { type_id, .. } => Ok(narrow_exact_value(value, *type_id)),
                _ => Ok(value.clone()),
            }
        } else {
            Err(VmError::TypeAssertionFailed {
                function: self.current_function_name(program)?,
                target: match target {
                    TypeCheck::Int => "int".into(),
                    TypeCheck::Float64 => "float64".into(),
                    TypeCheck::String => "string".into(),
                    TypeCheck::Bool => "bool".into(),
                    TypeCheck::Exact { name, .. } => name.clone(),
                    TypeCheck::Interface { name, .. } => name.clone(),
                    TypeCheck::Struct { name, .. } => name.clone(),
                },
            })
        }
    }
}

fn interface_wrapper_type(program: &Program, typ: TypeId) -> bool {
    typ == TYPE_ANY
        || typ == TYPE_ERROR
        || imported_interface_wrapper_type(typ)
        || program_type_inventory(program)
            .and_then(|inventory| inventory.type_info_for_type_id(typ))
            .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

fn imported_interface_wrapper_type(typ: TypeId) -> bool {
    matches!(typ.0, 100 | 102..=114)
}

fn exact_runtime_type_id(vm: &Vm, program: &Program, value: &Value) -> Option<TypeId> {
    match &value.data {
        ValueData::Nil => None,
        _ => match concrete_type_for_value(vm, program, value) {
            Some(ConcreteType::TypeId(type_id)) => Some(type_id),
            _ => Some(value.typ),
        },
    }
}

fn narrow_exact_value(value: &Value, type_id: TypeId) -> Value {
    let mut narrowed = value.clone();
    narrowed.typ = type_id;
    narrowed
}

fn method_binding_matches_interface_check(
    binding: &MethodBinding,
    method: &InterfaceMethodCheck,
) -> bool {
    binding.name == method.name
        && binding.param_types == method.param_types
        && binding.result_types == method.result_types
}

fn stdlib_method_matches_interface_check(
    function: StdlibFunctionId,
    method: &InterfaceMethodCheck,
) -> bool {
    let Some(params) = stdlib_function_param_types(function) else {
        return false;
    };
    if params.is_empty() {
        return false;
    }
    let param_types = params
        .iter()
        .skip(1)
        .map(|typ| (*typ).to_string())
        .collect::<Vec<_>>();
    let result_types = stdlib_function_result_types(function)
        .unwrap_or(&[])
        .iter()
        .map(|typ| (*typ).to_string())
        .collect::<Vec<_>>();
    param_types == method.param_types && result_types == method.result_types
}

fn pointer_concrete_type(typ: TypeId) -> Option<ConcreteType> {
    (typ != TYPE_POINTER).then_some(ConcreteType::TypeId(typ))
}
