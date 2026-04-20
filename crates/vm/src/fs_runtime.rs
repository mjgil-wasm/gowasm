use super::{Program, Value, ValueData, Vm, VmError, TYPE_FS_FILE};

const FILE_ID_FIELD: &str = "__fs_file_id";

impl Vm {
    pub(crate) fn workspace_fs_read(
        &mut self,
        program: &Program,
        file: &Value,
        buffer: &Value,
    ) -> Result<Vec<Value>, VmError> {
        let mut buffer = buffer.clone();
        self.workspace_fs_read_into(program, file, &mut buffer)
    }

    pub(crate) fn workspace_fs_read_into(
        &mut self,
        program: &Program,
        file: &Value,
        buffer: &mut Value,
    ) -> Result<Vec<Value>, VmError> {
        let file_id = self.extract_workspace_file_id(program, "fs.File.Read", file)?;
        let Some(state) = self.workspace_fs_files.get(&file_id) else {
            return Err(self.invalid_workspace_file_argument(program, "fs.File.Read"));
        };
        if state.closed {
            return Ok(vec![
                Value::int(0),
                Value::error_with_kind(
                    format!("read {}: file already closed", state.path),
                    "file already closed",
                ),
            ]);
        }
        if state.is_dir {
            return Ok(vec![
                Value::int(0),
                Value::error(format!("read {}: is a directory", state.path)),
            ]);
        }
        let path = state.path.clone();
        let read_offset = state.read_offset;

        let ValueData::Slice(slice) = &buffer.data else {
            return Err(VmError::InvalidStringFunctionArgument {
                function: self.current_function_name(program)?,
                builtin: "fs.File.Read".into(),
                expected: "a file value and []byte buffer".into(),
            });
        };
        if slice.is_empty() {
            return Ok(vec![Value::int(0), Value::nil()]);
        }

        let Some(contents) = self.workspace_files.get(&path) else {
            return Ok(vec![
                Value::int(0),
                Value::error_with_kind(
                    format!("read {path}: file does not exist"),
                    "file does not exist",
                ),
            ]);
        };
        let bytes = contents.as_bytes();
        if read_offset >= bytes.len() {
            return Ok(vec![Value::int(0), Value::error("EOF")]);
        }

        let read_count = slice.len().min(bytes.len().saturating_sub(read_offset));
        for (index, byte) in bytes[read_offset..].iter().take(read_count).enumerate() {
            assert!(
                slice.set(index, Value::int(i64::from(*byte))),
                "workspace file reads should stay within the provided buffer window"
            );
        }
        if let Some(state) = self.workspace_fs_files.get_mut(&file_id) {
            state.read_offset = read_offset + read_count;
        }
        Ok(vec![Value::int(read_count as i64), Value::nil()])
    }

    fn extract_workspace_file_id(
        &self,
        program: &Program,
        builtin: &str,
        value: &Value,
    ) -> Result<u64, VmError> {
        if value.typ != TYPE_FS_FILE {
            return Err(self.invalid_workspace_file_argument(program, builtin));
        }
        let ValueData::Struct(fields) = &value.data else {
            return Err(self.invalid_workspace_file_argument(program, builtin));
        };
        fields
            .iter()
            .find(|(name, _)| name == FILE_ID_FIELD)
            .and_then(|(_, value)| match &value.data {
                ValueData::Int(id) if *id >= 0 => Some(*id as u64),
                _ => None,
            })
            .ok_or_else(|| self.invalid_workspace_file_argument(program, builtin))
    }

    fn invalid_workspace_file_argument(&self, program: &Program, builtin: &str) -> VmError {
        VmError::InvalidStringFunctionArgument {
            function: self
                .current_function_name(program)
                .unwrap_or_else(|_| "<unknown>".into()),
            builtin: builtin.into(),
            expected: "a file value created by fs.FS.Open".into(),
        }
    }
}
