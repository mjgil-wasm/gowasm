use super::*;
use crate::value::{set_index_value, set_struct_field, SetFieldError, SetIndexError};

impl Vm {
    pub(super) fn read_register_from_frame(
        &self,
        program: &Program,
        frame_id: u64,
        register: usize,
    ) -> Result<Value, VmError> {
        let Some(frame) = self
            .current_goroutine()
            .frames
            .iter()
            .find(|frame| frame.id == frame_id)
        else {
            return Err(VmError::DanglingPointer {
                function: self.current_function_name(program)?,
                frame_id,
            });
        };
        let function_name = function_name(program, frame.function)?;
        frame
            .registers
            .get(register)
            .cloned()
            .ok_or(VmError::InvalidRegister {
                function: function_name,
                register,
            })
    }

    pub(super) fn set_register_on_frame(
        &mut self,
        program: &Program,
        frame_id: u64,
        register: usize,
        value: Value,
    ) -> Result<(), VmError> {
        self.debug_assert_value_invariants(program, &value);
        let Some(frame_index) = self
            .current_goroutine()
            .frames
            .iter()
            .position(|frame| frame.id == frame_id)
        else {
            return Err(VmError::DanglingPointer {
                function: self.current_function_name(program)?,
                frame_id,
            });
        };
        let frame = self
            .current_goroutine_mut()
            .frames
            .get_mut(frame_index)
            .expect("frame index should stay valid");
        let function_name = function_name(program, frame.function)?;
        let slot = frame
            .registers
            .get_mut(register)
            .ok_or(VmError::InvalidRegister {
                function: function_name,
                register,
            })?;
        *slot = value;
        Ok(())
    }

    pub(super) fn apply_field_store(
        &self,
        program: &Program,
        target: &mut Value,
        field: &str,
        value: Value,
    ) -> Result<(), VmError> {
        let target_description = describe_value(target);
        match set_struct_field(target, field, value) {
            Ok(()) => Ok(()),
            Err(SetFieldError::InvalidTarget) => Err(VmError::InvalidFieldTarget {
                function: self.current_function_name(program)?,
                target: target_description,
            }),
            Err(SetFieldError::UnknownField) => Err(VmError::UnknownField {
                function: self.current_function_name(program)?,
                field: field.into(),
            }),
        }
    }

    pub(super) fn apply_index_store(
        &self,
        program: &Program,
        target: &mut Value,
        index: &Value,
        value: Value,
    ) -> Result<(), VmError> {
        if matches!(target.data, ValueData::Map(_)) {
            self.validate_map_key_value(program, index)?;
        }
        let target_description = describe_value(target);
        match set_index_value(target, index, value) {
            Ok(()) => Ok(()),
            Err(SetIndexError::InvalidTarget) => Err(VmError::InvalidIndexTarget {
                function: self.current_function_name(program)?,
                target: target_description.clone(),
            }),
            Err(SetIndexError::InvalidIndexValue) => Err(VmError::InvalidIndexValue {
                function: self.current_function_name(program)?,
            }),
            Err(SetIndexError::IndexOutOfBounds { index, len }) => Err(VmError::IndexOutOfBounds {
                function: self.current_function_name(program)?,
                index,
                len,
            }),
            Err(SetIndexError::NilMap) => Err(VmError::AssignToNilMap {
                function: self.current_function_name(program)?,
                target: target_description,
            }),
        }
    }
}
