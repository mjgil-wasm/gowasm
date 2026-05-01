use crate::map_value::{build_map_index, map_key_is_comparable};

use super::*;

const CONTEXT_ID_FIELD: &str = "__context_id";
const TIME_UNIX_NANOS_FIELD: &str = "__time_unix_nanos";
const TIMER_CHANNEL_FIELD: &str = "__time_timer_channel";
const WAIT_GROUP_ID_FIELD: &str = "__sync_wait_group_id";
const ONCE_ID_FIELD: &str = "__sync_once_id";
const MUTEX_ID_FIELD: &str = "__sync_mutex_id";
const RW_MUTEX_ID_FIELD: &str = "__sync_rw_mutex_id";
const DIR_FS_ROOT_FIELD: &str = "__dirfs_root";
const FILE_ID_FIELD: &str = "__fs_file_id";
const REQUEST_BODY_ID_FIELD: &str = "__http_request_body_id";
const RESPONSE_BODY_ID_FIELD: &str = "__http_response_body_id";
const REPLACER_ID_FIELD: &str = "__strings_replacer_id";
const REGEXP_ID_FIELD: &str = "__regexp_id";
const BASE64_ENCODING_KIND_FIELD: &str = "__encodingKind";

const BASE64_STD_KIND: i64 = 0;
const BASE64_URL_KIND: i64 = 1;
const BASE64_RAW_STD_KIND: i64 = 2;
const BASE64_RAW_URL_KIND: i64 = 3;

impl Vm {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn assert_value_invariants(&self, program: &Program, value: &Value) {
        if let Err(message) = self.value_invariant_error(program, value, "value") {
            panic!("{message}");
        }
    }

    pub(crate) fn debug_assert_value_invariants(&self, _program: &Program, _value: &Value) {
        #[cfg(any(test, debug_assertions))]
        self.assert_value_invariants(_program, _value);
    }

    pub(crate) fn value_invariant_error(
        &self,
        program: &Program,
        value: &Value,
        path: &str,
    ) -> Result<(), String> {
        self.value_type_invariant_error(program, value, path)?;
        match &value.data {
            ValueData::Nil
            | ValueData::Int(_)
            | ValueData::Float(_)
            | ValueData::String(_)
            | ValueData::Bool(_) => {}
            ValueData::Error(error) => {
                if let Some(wrapped) = &error.wrapped {
                    self.value_invariant_error(program, wrapped, &format!("{path}.wrapped"))?;
                }
            }
            ValueData::Array(array) => {
                if let Some(concrete_type) = &array.concrete_type {
                    self.ensure_matching_concrete_type(path, "array", concrete_type, "array")?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                    if let ConcreteType::Array { len, .. } = concrete_type {
                        if *len != array.len() {
                            return Err(format!(
                                "{path} array length {} disagrees with concrete len {len}",
                                array.len()
                            ));
                        }
                    }
                }
                for (index, item) in array.values_snapshot().iter().enumerate() {
                    self.value_invariant_error(program, item, &format!("{path}[{index}]"))?;
                }
            }
            ValueData::Slice(slice) => {
                if slice.cap < slice.len() {
                    return Err(format!(
                        "{path} slice cap {} is smaller than len {}",
                        slice.cap,
                        slice.len()
                    ));
                }
                if slice.is_nil && (!slice.is_empty() || slice.cap != 0) {
                    return Err(format!(
                        "{path} nil slice must have empty values and zero cap"
                    ));
                }
                if let Some(concrete_type) = &slice.concrete_type {
                    self.ensure_matching_concrete_type(path, "slice", concrete_type, "slice")?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                }
                for (index, item) in slice.values_snapshot().iter().enumerate() {
                    self.value_invariant_error(program, item, &format!("{path}[{index}]"))?;
                }
            }
            ValueData::Map(map) => {
                self.value_invariant_error(program, &map.zero_value, &format!("{path}.zero"))?;
                match (&map.entries, &map.index) {
                    (None, None) => {}
                    (Some(entries), Some(index)) => {
                        let entries = entries.borrow();
                        let index = index.borrow();
                        if !entries.iter().all(|(key, _)| map_key_is_comparable(key)) {
                            return Err(format!(
                                "{path} map contains a non-comparable key in runtime storage"
                            ));
                        }
                        let rebuilt = build_map_index(&entries);
                        if rebuilt != *index {
                            return Err(format!("{path} map index does not match its entries"));
                        }
                        for (entry_index, (key, item)) in entries.iter().enumerate() {
                            self.value_invariant_error(
                                program,
                                key,
                                &format!("{path}.key[{entry_index}]"),
                            )?;
                            self.value_invariant_error(
                                program,
                                item,
                                &format!("{path}.value[{entry_index}]"),
                            )?;
                        }
                    }
                    _ => {
                        return Err(format!(
                            "{path} map entries/index must either both exist or both be absent"
                        ));
                    }
                }
                if let Some(concrete_type) = &map.concrete_type {
                    self.ensure_matching_concrete_type(path, "map", concrete_type, "map")?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                }
            }
            ValueData::Channel(channel) => {
                if let Some(id) = channel.id {
                    if self.channels.get(id as usize).is_none() {
                        return Err(format!("{path} channel id {id} does not exist"));
                    }
                }
                if let Some(concrete_type) = &channel.concrete_type {
                    self.ensure_matching_concrete_type(path, "channel", concrete_type, "channel")?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                }
            }
            ValueData::Pointer(pointer) => {
                self.pointer_target_invariant_error(program, &pointer.target, path)?;
                if let Some(concrete_type) = &pointer.concrete_type {
                    self.ensure_matching_concrete_type(path, "pointer", concrete_type, "pointer")?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                }
            }
            ValueData::Function(function) => {
                if program.functions.get(function.function).is_none() {
                    return Err(format!(
                        "{path} function id {} does not exist in the current program",
                        function.function
                    ));
                }
                if let Some(concrete_type) = &function.concrete_type {
                    self.ensure_matching_concrete_type(
                        path,
                        "function",
                        concrete_type,
                        "function",
                    )?;
                    self.concrete_type_invariant_error(
                        program,
                        concrete_type,
                        &format!("{path}.concrete_type"),
                    )?;
                }
                for (index, capture) in function.captures.iter().enumerate() {
                    self.value_invariant_error(
                        program,
                        capture,
                        &format!("{path}.capture[{index}]"),
                    )?;
                }
            }
            ValueData::Struct(fields) => {
                self.ensure_unique_field_names(fields, path)?;
                for (name, field_value) in fields {
                    self.value_invariant_error(program, field_value, &format!("{path}.{name}"))?;
                }
                self.host_struct_invariant_error(fields, path, value.typ)?;
            }
        }

        Ok(())
    }

    fn pointer_target_invariant_error(
        &self,
        program: &Program,
        target: &PointerTarget,
        path: &str,
    ) -> Result<(), String> {
        match target {
            PointerTarget::Nil => Ok(()),
            PointerTarget::HeapCell { cell } => {
                if self
                    .heap_cells
                    .get(*cell)
                    .and_then(|slot| slot.as_ref())
                    .is_none()
                {
                    return Err(format!("{path} heap cell {cell} is not live"));
                }
                Ok(())
            }
            PointerTarget::LocalField { field, .. } | PointerTarget::GlobalField { field, .. } => {
                if field.is_empty() {
                    return Err(format!("{path} projected field name must not be empty"));
                }
                Ok(())
            }
            PointerTarget::LocalIndex { index, .. } | PointerTarget::GlobalIndex { index, .. } => {
                self.value_invariant_error(program, index, &format!("{path}.index"))
            }
            PointerTarget::Local { frame_id, register } => self
                .any_frame_register_value(*frame_id, *register)
                .ok_or_else(|| {
                    format!("{path} points at missing frame/register {frame_id}:{register}")
                })
                .map(|_| ()),
            PointerTarget::Global { global } => {
                if self.globals.get(*global).is_none() {
                    return Err(format!("{path} points at missing global {global}"));
                }
                Ok(())
            }
            PointerTarget::ProjectedField { base, field } => {
                if field.is_empty() {
                    return Err(format!("{path} projected field name must not be empty"));
                }
                self.pointer_target_invariant_error(program, base, &format!("{path}.base"))
            }
            PointerTarget::ProjectedIndex { base, index } => {
                self.pointer_target_invariant_error(program, base, &format!("{path}.base"))?;
                self.value_invariant_error(program, index, &format!("{path}.index"))
            }
        }
    }

    fn host_struct_invariant_error(
        &self,
        fields: &[(String, Value)],
        path: &str,
        typ: TypeId,
    ) -> Result<(), String> {
        match typ {
            TYPE_TIME => {
                if let Some(value) = find_field(fields, TIME_UNIX_NANOS_FIELD) {
                    if !matches!(value.data, ValueData::Int(_)) {
                        return Err(format!("{path}.{TIME_UNIX_NANOS_FIELD} must be int"));
                    }
                }
                Ok(())
            }
            TYPE_TIME_TIMER => {
                let c = self.expect_optional_channel_id_field(fields, path, "C")?;
                let hidden =
                    self.expect_optional_channel_id_field(fields, path, TIMER_CHANNEL_FIELD)?;
                if let (Some(c), Some(hidden)) = (c, hidden) {
                    if c != hidden {
                        return Err(format!(
                            "{path} timer channel ids disagree between `C` and `{TIMER_CHANNEL_FIELD}`"
                        ));
                    }
                }
                Ok(())
            }
            TYPE_CONTEXT => self.expect_registry_id(fields, path, CONTEXT_ID_FIELD, |id| {
                self.context_values.contains_key(&id)
            }),
            TYPE_SYNC_WAIT_GROUP => {
                self.expect_optional_registry_id(fields, path, WAIT_GROUP_ID_FIELD, |id| {
                    self.wait_groups.contains_key(&id)
                })
            }
            TYPE_SYNC_ONCE => self.expect_optional_registry_id(fields, path, ONCE_ID_FIELD, |id| {
                self.once_values.contains_key(&id)
            }),
            TYPE_SYNC_MUTEX => {
                self.expect_optional_registry_id(fields, path, MUTEX_ID_FIELD, |id| {
                    self.mutex_values.contains_key(&id)
                })
            }
            TYPE_SYNC_RW_MUTEX => {
                self.expect_optional_registry_id(fields, path, RW_MUTEX_ID_FIELD, |id| {
                    self.rw_mutex_values.contains_key(&id)
                })
            }
            TYPE_OS_DIR_FS => {
                self.expect_field_kind(fields, path, DIR_FS_ROOT_FIELD, "string", |value| {
                    matches!(value.data, ValueData::String(_))
                })
            }
            TYPE_FS_FILE => self.expect_registry_id(fields, path, FILE_ID_FIELD, |id| {
                self.workspace_fs_files.contains_key(&id)
            }),
            TYPE_HTTP_REQUEST_BODY => {
                self.expect_registry_id(fields, path, REQUEST_BODY_ID_FIELD, |id| {
                    self.http_request_bodies.contains_key(&id)
                })
            }
            TYPE_HTTP_RESPONSE_BODY => {
                self.expect_registry_id(fields, path, RESPONSE_BODY_ID_FIELD, |id| {
                    self.http_response_bodies.contains_key(&id)
                })
            }
            TYPE_STRINGS_REPLACER => {
                self.expect_registry_id(fields, path, REPLACER_ID_FIELD, |id| {
                    self.string_replacers.contains_key(&id)
                })
            }
            TYPE_REGEXP => self.expect_registry_id(fields, path, REGEXP_ID_FIELD, |id| {
                self.compiled_regexps.contains_key(&id)
            }),
            TYPE_BASE64_ENCODING => self.expect_field_kind(
                fields,
                path,
                BASE64_ENCODING_KIND_FIELD,
                "known base64 encoding kind",
                |value| {
                    matches!(
                        value.data,
                        ValueData::Int(
                            BASE64_STD_KIND
                                | BASE64_URL_KIND
                                | BASE64_RAW_STD_KIND
                                | BASE64_RAW_URL_KIND
                        )
                    )
                },
            ),
            _ => Ok(()),
        }
    }

    fn ensure_unique_field_names(
        &self,
        fields: &[(String, Value)],
        path: &str,
    ) -> Result<(), String> {
        let mut names = std::collections::BTreeSet::new();
        for (name, _) in fields {
            if !names.insert(name) {
                return Err(format!("{path} contains duplicate struct field `{name}`"));
            }
        }
        Ok(())
    }

    fn ensure_matching_concrete_type(
        &self,
        path: &str,
        value_kind: &str,
        concrete_type: &ConcreteType,
        expected_kind: &str,
    ) -> Result<(), String> {
        if matches!(concrete_type, ConcreteType::TypeId(_)) {
            return Ok(());
        }
        let actual_kind = match concrete_type {
            ConcreteType::TypeId(_) => unreachable!(),
            ConcreteType::Array { .. } => "array",
            ConcreteType::Slice { .. } => "slice",
            ConcreteType::Map { .. } => "map",
            ConcreteType::Pointer { .. } => "pointer",
            ConcreteType::Function { .. } => "function",
            ConcreteType::Channel { .. } => "channel",
        };
        if actual_kind != expected_kind {
            return Err(format!(
                "{path} {value_kind} carries mismatched concrete type kind `{actual_kind}`"
            ));
        }
        Ok(())
    }

    fn expect_registry_id<F>(
        &self,
        fields: &[(String, Value)],
        path: &str,
        field: &str,
        present: F,
    ) -> Result<(), String>
    where
        F: FnOnce(u64) -> bool,
    {
        let id = self.expect_positive_int_field(fields, path, field)?;
        if !present(id) {
            return Err(format!(
                "{path} references missing registry id {id} via `{field}`"
            ));
        }
        Ok(())
    }

    fn expect_optional_registry_id<F>(
        &self,
        fields: &[(String, Value)],
        path: &str,
        field: &str,
        present: F,
    ) -> Result<(), String>
    where
        F: FnOnce(u64) -> bool,
    {
        let Some(value) = find_field(fields, field) else {
            return Ok(());
        };
        let ValueData::Int(id) = value.data else {
            return Err(format!("{path}.{field} must be an int"));
        };
        let id = u64::try_from(id)
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| format!("{path}.{field} must be a positive runtime id"))?;
        if !present(id) {
            return Err(format!(
                "{path} references missing registry id {id} via `{field}`"
            ));
        }
        Ok(())
    }

    fn expect_optional_channel_id_field(
        &self,
        fields: &[(String, Value)],
        path: &str,
        field: &str,
    ) -> Result<Option<u64>, String> {
        let Some(value) = find_field(fields, field) else {
            return Ok(None);
        };
        let ValueData::Channel(channel) = &value.data else {
            return Err(format!("{path}.{field} must be a channel"));
        };
        let Some(id) = channel.id else {
            return Ok(None);
        };
        if self.channels.get(id as usize).is_none() {
            return Err(format!("{path}.{field} references unknown channel id {id}"));
        }
        Ok(Some(id))
    }

    fn expect_positive_int_field(
        &self,
        fields: &[(String, Value)],
        path: &str,
        field: &str,
    ) -> Result<u64, String> {
        let value = find_field(fields, field)
            .ok_or_else(|| format!("{path} is missing required field `{field}`"))?;
        let ValueData::Int(id) = value.data else {
            return Err(format!("{path}.{field} must be an int"));
        };
        u64::try_from(id)
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| format!("{path}.{field} must be a positive runtime id"))
    }

    fn expect_field_kind<F>(
        &self,
        fields: &[(String, Value)],
        path: &str,
        field: &str,
        expected: &str,
        matches: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&Value) -> bool,
    {
        let value = find_field(fields, field)
            .ok_or_else(|| format!("{path} is missing required field `{field}`"))?;
        if !matches(value) {
            return Err(format!("{path}.{field} must be {expected}"));
        }
        Ok(())
    }
}

fn find_field<'a>(fields: &'a [(String, Value)], field: &str) -> Option<&'a Value> {
    fields
        .iter()
        .find(|(name, _)| name == field)
        .map(|(_, value)| value)
}
