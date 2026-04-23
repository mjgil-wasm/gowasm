use std::collections::BTreeSet;

use super::{
    program_type_inventory, DeferredCallKind, GoroutineId, GoroutineStatus, Program, ReturnTarget,
    RuntimeTypeKind, TypeId, UnwindState, Value, ValueData, Vm, TYPE_ANY, TYPE_ARRAY,
    TYPE_BASE64_ENCODING, TYPE_BASE64_ENCODING_PTR, TYPE_BOOL, TYPE_CHANNEL, TYPE_CONTEXT,
    TYPE_EMPTY_STRUCT, TYPE_ERROR, TYPE_FLOAT64, TYPE_FS_DIR_ENTRY, TYPE_FS_FILE,
    TYPE_FS_FILE_INFO, TYPE_FS_FILE_MODE, TYPE_FS_SUB_FS, TYPE_FUNCTION, TYPE_HTTP_CLIENT,
    TYPE_HTTP_CLIENT_PTR, TYPE_HTTP_HEADER, TYPE_HTTP_REQUEST, TYPE_HTTP_REQUEST_BODY,
    TYPE_HTTP_REQUEST_PTR, TYPE_HTTP_RESPONSE, TYPE_HTTP_RESPONSE_BODY, TYPE_HTTP_RESPONSE_PTR,
    TYPE_INT, TYPE_INT64, TYPE_MAP, TYPE_NIL, TYPE_OS_DIR_FS, TYPE_POINTER, TYPE_REFLECT_KIND,
    TYPE_REFLECT_RTYPE, TYPE_REFLECT_RVALUE, TYPE_REFLECT_STRUCT_FIELD, TYPE_REFLECT_STRUCT_TAG,
    TYPE_REFLECT_TYPE, TYPE_REFLECT_VALUE, TYPE_REGEXP, TYPE_SLICE, TYPE_STRING,
    TYPE_STRINGS_REPLACER, TYPE_SYNC_MUTEX, TYPE_SYNC_MUTEX_PTR, TYPE_SYNC_ONCE,
    TYPE_SYNC_ONCE_PTR, TYPE_SYNC_RW_MUTEX, TYPE_SYNC_RW_MUTEX_PTR, TYPE_SYNC_WAIT_GROUP,
    TYPE_SYNC_WAIT_GROUP_PTR, TYPE_TIME, TYPE_TIME_DURATION, TYPE_TIME_PTR, TYPE_TIME_TIMER,
    TYPE_TIME_TIMER_PTR, TYPE_URL, TYPE_URL_PTR, TYPE_URL_USERINFO, TYPE_URL_USERINFO_PTR,
    TYPE_URL_VALUES,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeInvariantMode {
    #[default]
    Off,
    AfterEachInstruction,
}

impl Vm {
    pub fn set_runtime_invariant_mode(&mut self, mode: RuntimeInvariantMode) {
        self.runtime_invariant_mode = mode;
    }

    pub fn runtime_invariant_mode(&self) -> RuntimeInvariantMode {
        self.runtime_invariant_mode
    }

    pub(crate) fn assert_runtime_invariants(&self, program: &Program) {
        if let Err(message) = self.runtime_invariant_error(program) {
            panic!("{message}");
        }
    }

    pub(crate) fn assert_runtime_invariants_if_enabled(&self, program: &Program) {
        if self.runtime_invariant_mode == RuntimeInvariantMode::AfterEachInstruction {
            self.assert_runtime_invariants(program);
        }
    }

    pub(crate) fn any_frame_register_value(
        &self,
        frame_id: u64,
        register: usize,
    ) -> Option<&Value> {
        self.goroutines
            .iter()
            .flat_map(|goroutine| goroutine.frames.iter())
            .find(|frame| frame.id == frame_id)
            .and_then(|frame| frame.registers.get(register))
    }

    fn runtime_invariant_error(&self, program: &Program) -> Result<(), String> {
        self.frame_layout_invariant_error(program)?;
        self.root_value_invariant_error(program)?;
        self.gc_metadata_invariant_error()?;
        self.runtime_state_invariant_error()?;
        self.channel_wait_queue_invariant_error()?;
        Ok(())
    }

    fn frame_layout_invariant_error(&self, program: &Program) -> Result<(), String> {
        if self.goroutines.is_empty() {
            if self.current_goroutine != 0 {
                return Err(format!(
                    "current goroutine index {} must be zero when no goroutines exist",
                    self.current_goroutine
                ));
            }
            return Ok(());
        }

        if self.current_goroutine >= self.goroutines.len() {
            return Err(format!(
                "current goroutine index {} is out of bounds for {} goroutines",
                self.current_goroutine,
                self.goroutines.len()
            ));
        }

        let mut goroutine_ids = BTreeSet::new();
        let mut frame_ids = BTreeSet::new();
        for goroutine in &self.goroutines {
            if !goroutine_ids.insert(goroutine.id.0) {
                return Err(format!("duplicate goroutine id {}", goroutine.id.0));
            }
            match goroutine.status {
                GoroutineStatus::Done => {
                    if !goroutine.frames.is_empty() {
                        return Err(format!(
                            "done goroutine {} must not retain stack frames",
                            goroutine.id.0
                        ));
                    }
                }
                GoroutineStatus::Runnable | GoroutineStatus::Blocked => {
                    if goroutine.frames.is_empty() {
                        return Err(format!(
                            "{:?} goroutine {} must retain at least one frame",
                            goroutine.status, goroutine.id.0
                        ));
                    }
                }
            }
            if goroutine.active_select.is_some() && goroutine.status != GoroutineStatus::Blocked {
                return Err(format!(
                    "goroutine {} has an active select while in {:?} state",
                    goroutine.id.0, goroutine.status
                ));
            }

            for (frame_index, frame) in goroutine.frames.iter().enumerate() {
                if !frame_ids.insert(frame.id) {
                    return Err(format!("duplicate frame id {}", frame.id));
                }
                let Some(function) = program.functions.get(frame.function) else {
                    return Err(format!(
                        "frame {} references unknown function {}",
                        frame.id, frame.function
                    ));
                };
                if frame.pc > function.code.len() {
                    return Err(format!(
                        "frame {} pc {} exceeds function `{}` code length {}",
                        frame.id,
                        frame.pc,
                        function.name,
                        function.code.len()
                    ));
                }
                if frame.registers.len() != function.register_count {
                    return Err(format!(
                        "frame {} register count {} disagrees with function `{}` register count {}",
                        frame.id,
                        frame.registers.len(),
                        function.name,
                        function.register_count
                    ));
                }
                self.return_target_invariant_error(
                    &goroutine.id,
                    frame.id,
                    frame_index,
                    &goroutine.frames,
                    &frame.return_target,
                )?;
                self.deferred_call_invariant_error(program, frame.id, &frame.deferred)?;
            }
        }
        Ok(())
    }

    fn return_target_invariant_error(
        &self,
        goroutine_id: &GoroutineId,
        frame_id: u64,
        frame_index: usize,
        frames: &[super::Frame],
        return_target: &ReturnTarget,
    ) -> Result<(), String> {
        match return_target {
            ReturnTarget::None => Ok(()),
            ReturnTarget::Deferred | ReturnTarget::Callback => {
                if frame_index == 0 {
                    return Err(format!(
                        "goroutine {} frame {} uses {:?} without a caller frame",
                        goroutine_id.0, frame_id, return_target
                    ));
                }
                Ok(())
            }
            ReturnTarget::One(dst) => {
                let Some(caller) = frames.get(frame_index.saturating_sub(1)) else {
                    return Err(format!(
                        "goroutine {} frame {} stores one result without a caller frame",
                        goroutine_id.0, frame_id
                    ));
                };
                if *dst >= caller.registers.len() {
                    return Err(format!(
                        "goroutine {} frame {} stores result register {} beyond caller frame {} register count {}",
                        goroutine_id.0,
                        frame_id,
                        dst,
                        caller.id,
                        caller.registers.len()
                    ));
                }
                Ok(())
            }
            ReturnTarget::Many(dsts) => {
                let Some(caller) = frames.get(frame_index.saturating_sub(1)) else {
                    return Err(format!(
                        "goroutine {} frame {} stores multiple results without a caller frame",
                        goroutine_id.0, frame_id
                    ));
                };
                let mut seen = BTreeSet::new();
                for dst in dsts {
                    if *dst >= caller.registers.len() {
                        return Err(format!(
                            "goroutine {} frame {} stores result register {} beyond caller frame {} register count {}",
                            goroutine_id.0,
                            frame_id,
                            dst,
                            caller.id,
                            caller.registers.len()
                        ));
                    }
                    if !seen.insert(*dst) {
                        return Err(format!(
                            "goroutine {} frame {} repeats caller result register {}",
                            goroutine_id.0, frame_id, dst
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    fn deferred_call_invariant_error(
        &self,
        program: &Program,
        frame_id: u64,
        deferred: &[super::DeferredCall],
    ) -> Result<(), String> {
        for (deferred_index, call) in deferred.iter().enumerate() {
            match &call.kind {
                DeferredCallKind::Closure { function } => {
                    self.value_invariant_error(
                        program,
                        function,
                        &format!("frame[{frame_id}].deferred[{deferred_index}].closure"),
                    )?;
                }
                DeferredCallKind::Function { function } => {
                    if program.functions.get(*function).is_none() {
                        return Err(format!(
                            "frame[{frame_id}].deferred[{deferred_index}] references unknown function {}",
                            function
                        ));
                    }
                }
                DeferredCallKind::Stdlib { .. } => {}
            }
            for (arg_index, value) in call.args.iter().enumerate() {
                self.value_invariant_error(
                    program,
                    value,
                    &format!("frame[{frame_id}].deferred[{deferred_index}].args[{arg_index}]"),
                )?;
            }
        }
        Ok(())
    }

    fn root_value_invariant_error(&self, program: &Program) -> Result<(), String> {
        for (global, value) in self.globals.iter().enumerate() {
            self.value_invariant_error(program, value, &format!("globals[{global}]"))?;
        }

        for goroutine in &self.goroutines {
            for frame in &goroutine.frames {
                for (register, value) in frame.registers.iter().enumerate() {
                    self.value_invariant_error(
                        program,
                        value,
                        &format!(
                            "goroutine[{}].frame[{}].register[{register}]",
                            goroutine.id.0, frame.id
                        ),
                    )?;
                }
                if let Some(unwind) = &frame.unwind {
                    match unwind {
                        UnwindState::Return(values) => {
                            for (index, value) in values.iter().enumerate() {
                                self.value_invariant_error(
                                    program,
                                    value,
                                    &format!(
                                        "goroutine[{}].frame[{}].unwind.return[{index}]",
                                        goroutine.id.0, frame.id
                                    ),
                                )?;
                            }
                        }
                        UnwindState::Panic(value) => {
                            self.value_invariant_error(
                                program,
                                value,
                                &format!(
                                    "goroutine[{}].frame[{}].unwind.panic",
                                    goroutine.id.0, frame.id
                                ),
                            )?;
                        }
                        UnwindState::Recovered => {}
                    }
                }
            }
        }

        if let Some(results) = &self.callback_result {
            for (index, value) in results.iter().enumerate() {
                self.value_invariant_error(program, value, &format!("callback_result[{index}]"))?;
            }
        }
        if let Some(values) = &self.callback_captured_args {
            for (index, value) in values.iter().enumerate() {
                self.value_invariant_error(
                    program,
                    value,
                    &format!("callback_captured_args[{index}]"),
                )?;
            }
        }
        if let Some(value) = &self.pending_http_request_context {
            self.value_invariant_error(program, value, "pending_http_request_context")?;
        }
        for (body_id, state) in &self.http_request_bodies {
            self.value_invariant_error(
                program,
                &state.reader,
                &format!("http_request_bodies[{body_id}].reader"),
            )?;
        }
        for (body_id, state) in &self.http_response_bodies {
            if let Some(error) = &state.terminal_error {
                self.value_invariant_error(
                    program,
                    error,
                    &format!("http_response_bodies[{body_id}].terminal_error"),
                )?;
            }
            if let Some(context) = &state.request_context {
                self.value_invariant_error(
                    program,
                    context,
                    &format!("http_response_bodies[{body_id}].request_context"),
                )?;
            }
        }
        for (channel_id, watchers) in &self.context_done_watchers {
            for (watcher_index, watcher) in watchers.iter().enumerate() {
                self.value_invariant_error(
                    program,
                    &watcher.parent,
                    &format!("context_done_watchers[{channel_id}][{watcher_index}].parent"),
                )?;
            }
        }
        Ok(())
    }

    fn gc_metadata_invariant_error(&self) -> Result<(), String> {
        let mut free = BTreeSet::new();
        for cell in &self.free_heap_cells {
            if *cell >= self.heap_cells.len() {
                return Err(format!(
                    "free heap cell index {} exceeds heap length {}",
                    cell,
                    self.heap_cells.len()
                ));
            }
            if !free.insert(*cell) {
                return Err(format!("free heap cell index {} is duplicated", cell));
            }
            if self.heap_cells[*cell].is_some() {
                return Err(format!(
                    "free heap cell index {} still contains a live value",
                    cell
                ));
            }
        }

        for (cell, slot) in self.heap_cells.iter().enumerate() {
            if slot.is_none() && !free.contains(&cell) {
                return Err(format!(
                    "heap cell {} is empty but missing from the free list",
                    cell
                ));
            }
            if slot.is_some() && free.contains(&cell) {
                return Err(format!(
                    "heap cell {} is live but also listed as free",
                    cell
                ));
            }
        }

        Ok(())
    }

    fn runtime_state_invariant_error(&self) -> Result<(), String> {
        for goroutine in self.sleeping_goroutines.keys() {
            let Some(state) = self
                .goroutines
                .iter()
                .find(|candidate| candidate.id == *goroutine)
            else {
                return Err(format!(
                    "sleep queue references unknown goroutine {}",
                    goroutine.0
                ));
            };
            if state.status != GoroutineStatus::Blocked {
                return Err(format!(
                    "sleep queue goroutine {} must be blocked",
                    goroutine.0
                ));
            }
        }

        for timer in &self.time_channel_timers {
            if self.channels.get(timer.channel_id as usize).is_none() {
                return Err(format!(
                    "time channel timer references unknown channel {}",
                    timer.channel_id
                ));
            }
        }

        for timer in &self.context_deadline_timers {
            if !self.context_values.contains_key(&timer.context_id) {
                return Err(format!(
                    "context deadline timer references unknown context {}",
                    timer.context_id
                ));
            }
        }

        for (channel_id, watchers) in &self.context_done_watchers {
            if self.channels.get(*channel_id as usize).is_none() {
                return Err(format!(
                    "context done watcher list references unknown channel {}",
                    channel_id
                ));
            }
            for watcher in watchers {
                if !self.context_values.contains_key(&watcher.context_id) {
                    return Err(format!(
                        "context done watcher references unknown context {}",
                        watcher.context_id
                    ));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn value_type_invariant_error(
        &self,
        program: &Program,
        value: &Value,
        path: &str,
    ) -> Result<(), String> {
        self.type_id_invariant_error(program, value.typ, &value.data, path)?;
        Ok(())
    }

    pub(crate) fn concrete_type_invariant_error(
        &self,
        program: &Program,
        concrete_type: &super::ConcreteType,
        path: &str,
    ) -> Result<(), String> {
        match concrete_type {
            super::ConcreteType::TypeId(type_id) => {
                self.type_id_exists_if_inventory_knows_it(program, *type_id, path)
            }
            super::ConcreteType::Array { element, .. }
            | super::ConcreteType::Slice { element }
            | super::ConcreteType::Pointer { element }
            | super::ConcreteType::Channel { element, .. } => {
                self.concrete_type_invariant_error(program, element, path)
            }
            super::ConcreteType::Map { key, value } => {
                self.concrete_type_invariant_error(program, key, path)?;
                self.concrete_type_invariant_error(program, value, path)
            }
            super::ConcreteType::Function { params, results } => {
                for typ in params {
                    self.concrete_type_invariant_error(program, typ, path)?;
                }
                for typ in results {
                    self.concrete_type_invariant_error(program, typ, path)?;
                }
                Ok(())
            }
        }
    }

    fn type_id_invariant_error(
        &self,
        program: &Program,
        type_id: TypeId,
        data: &ValueData,
        path: &str,
    ) -> Result<(), String> {
        if let Some(expected_kind) = builtin_type_kind(type_id) {
            let actual_kind = runtime_value_kind(data);
            if actual_kind == "nil" && builtin_type_kind_accepts_nil(expected_kind) {
                return Ok(());
            }
            if expected_kind != "interface"
                && expected_kind != "error"
                && expected_kind != "nilable-interface"
                && actual_kind != expected_kind
            {
                return Err(format!(
                    "{path} uses builtin type id {} with runtime kind `{actual_kind}` instead of `{expected_kind}`",
                    type_id.0
                ));
            }
            return Ok(());
        }

        let Some(inventory) = program_type_inventory(program) else {
            return Ok(());
        };
        let Some(info) = inventory.type_info_for_type_id(type_id) else {
            return Ok(());
        };
        if matches!(info.kind, RuntimeTypeKind::Interface) {
            return Ok(());
        }
        let actual_kind = runtime_value_kind(data);
        if actual_kind == "nil" && runtime_type_kind_accepts_nil(&info.kind) {
            return Ok(());
        }
        let expected_kind = runtime_type_kind_name(&info.kind);
        if actual_kind != expected_kind {
            return Err(format!(
                "{path} uses named type id {} with runtime kind `{actual_kind}` instead of `{expected_kind}`",
                type_id.0
            ));
        }
        Ok(())
    }

    fn type_id_exists_if_inventory_knows_it(
        &self,
        program: &Program,
        type_id: TypeId,
        path: &str,
    ) -> Result<(), String> {
        if builtin_type_kind(type_id).is_some() {
            return Ok(());
        }
        let Some(inventory) = program_type_inventory(program) else {
            return Ok(());
        };
        if inventory.type_info_for_type_id(type_id).is_none() {
            return Err(format!(
                "{path} references unknown type id {} in the program inventory",
                type_id.0
            ));
        }
        Ok(())
    }
}

fn builtin_type_kind(type_id: TypeId) -> Option<&'static str> {
    match type_id {
        TYPE_NIL => Some("nil"),
        TYPE_INT | TYPE_TIME_DURATION | TYPE_FS_FILE_MODE | TYPE_REFLECT_KIND => Some("int"),
        TYPE_INT64 => Some("int"),
        TYPE_STRING => Some("string"),
        TYPE_BOOL => Some("bool"),
        TYPE_FLOAT64 => Some("float"),
        TYPE_ARRAY => Some("array"),
        TYPE_SLICE => Some("slice"),
        TYPE_MAP | TYPE_HTTP_HEADER | TYPE_URL_VALUES => Some("map"),
        TYPE_POINTER
        | TYPE_SYNC_WAIT_GROUP_PTR
        | TYPE_SYNC_ONCE_PTR
        | TYPE_SYNC_MUTEX_PTR
        | TYPE_TIME_PTR
        | TYPE_TIME_TIMER_PTR
        | TYPE_HTTP_REQUEST_PTR
        | TYPE_HTTP_RESPONSE_PTR
        | TYPE_URL_PTR
        | TYPE_URL_USERINFO_PTR
        | TYPE_HTTP_CLIENT_PTR
        | TYPE_SYNC_RW_MUTEX_PTR
        | TYPE_BASE64_ENCODING_PTR => Some("pointer"),
        TYPE_FUNCTION => Some("function"),
        TYPE_CHANNEL => Some("channel"),
        TYPE_ANY => Some("nilable-interface"),
        TYPE_ERROR => Some("error"),
        TYPE_TIME
        | TYPE_TIME_TIMER
        | TYPE_EMPTY_STRUCT
        | TYPE_CONTEXT
        | TYPE_OS_DIR_FS
        | TYPE_FS_FILE
        | TYPE_FS_FILE_INFO
        | TYPE_FS_DIR_ENTRY
        | TYPE_HTTP_REQUEST
        | TYPE_HTTP_RESPONSE
        | TYPE_URL
        | TYPE_HTTP_RESPONSE_BODY
        | TYPE_HTTP_CLIENT
        | TYPE_FS_SUB_FS
        | TYPE_HTTP_REQUEST_BODY
        | TYPE_URL_USERINFO
        | TYPE_SYNC_WAIT_GROUP
        | TYPE_SYNC_ONCE
        | TYPE_SYNC_MUTEX
        | TYPE_SYNC_RW_MUTEX
        | TYPE_STRINGS_REPLACER
        | TYPE_REGEXP
        | TYPE_BASE64_ENCODING
        | TYPE_REFLECT_TYPE
        | TYPE_REFLECT_STRUCT_FIELD
        | TYPE_REFLECT_RTYPE
        | TYPE_REFLECT_VALUE
        | TYPE_REFLECT_RVALUE => Some("struct"),
        TYPE_REFLECT_STRUCT_TAG => Some("string"),
        _ => None,
    }
}

fn runtime_value_kind(data: &ValueData) -> &'static str {
    match data {
        ValueData::Nil => "nil",
        ValueData::Int(_) => "int",
        ValueData::Float(_) => "float",
        ValueData::String(_) => "string",
        ValueData::Bool(_) => "bool",
        ValueData::Error(_) => "error",
        ValueData::Array(_) => "array",
        ValueData::Slice(_) => "slice",
        ValueData::Map(_) => "map",
        ValueData::Channel(_) => "channel",
        ValueData::Pointer(_) => "pointer",
        ValueData::Function(_) => "function",
        ValueData::Struct(_) => "struct",
    }
}

fn runtime_type_kind_name(kind: &RuntimeTypeKind) -> &'static str {
    match kind {
        RuntimeTypeKind::Nil => "nil",
        RuntimeTypeKind::Int => "int",
        RuntimeTypeKind::Float64 => "float",
        RuntimeTypeKind::String => "string",
        RuntimeTypeKind::Bool => "bool",
        RuntimeTypeKind::Array => "array",
        RuntimeTypeKind::Slice => "slice",
        RuntimeTypeKind::Map => "map",
        RuntimeTypeKind::Struct => "struct",
        RuntimeTypeKind::Interface => "interface",
        RuntimeTypeKind::Pointer => "pointer",
        RuntimeTypeKind::Function => "function",
        RuntimeTypeKind::Channel => "channel",
    }
}

fn builtin_type_kind_accepts_nil(kind: &str) -> bool {
    matches!(
        kind,
        "pointer"
            | "function"
            | "slice"
            | "map"
            | "channel"
            | "interface"
            | "error"
            | "nilable-interface"
    )
}

fn runtime_type_kind_accepts_nil(kind: &RuntimeTypeKind) -> bool {
    matches!(
        kind,
        RuntimeTypeKind::Slice
            | RuntimeTypeKind::Map
            | RuntimeTypeKind::Interface
            | RuntimeTypeKind::Pointer
            | RuntimeTypeKind::Function
            | RuntimeTypeKind::Channel
    )
}
