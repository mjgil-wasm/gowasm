use super::{
    timer_time_value, ContextDeadlineTimer, ContextDoneWatcher, ContextErrorKind,
    FetchResponseChunkResult, FetchResult, GcStats, HttpRequestUploadState,
    PendingFetchResponseStart, Program, TimeChannelTimer, Value, Vm, VmError,
};

impl Vm {
    pub fn set_gc_allocation_threshold(&mut self, threshold: usize) {
        self.gc_allocation_threshold = if threshold == 0 {
            None
        } else {
            Some(threshold)
        };
    }

    pub fn clear_gc_allocation_threshold(&mut self) {
        self.gc_allocation_threshold = None;
    }

    pub fn gc_stats(&self) -> GcStats {
        GcStats {
            allocation_threshold: self.gc_allocation_threshold,
            allocations_since_gc: self.allocations_since_gc,
            heap_cells: self.heap_cells.len(),
            live_heap_cells: self.heap_cells.iter().filter(|slot| slot.is_some()).count(),
            free_heap_cells: self.free_heap_cells.len(),
            last_freed_cells: self.last_gc_freed_cells,
            total_collections: self.total_gc_collections,
            total_freed_cells: self.total_gc_freed_cells,
        }
    }

    pub fn set_instruction_budget(&mut self, budget: u64) {
        self.instruction_budget_limit = Some(budget);
        self.remaining_instruction_budget = Some(budget);
        self.executed_instruction_count = 0;
    }

    pub fn clear_instruction_budget(&mut self) {
        self.instruction_budget_limit = None;
        self.remaining_instruction_budget = None;
    }

    pub fn executed_instruction_count(&self) -> u64 {
        self.executed_instruction_count
    }

    pub fn instruction_budget_remaining(&self) -> Option<u64> {
        self.remaining_instruction_budget
    }

    pub fn set_instruction_yield_interval(&mut self, interval: u64) {
        self.instruction_yield_interval = interval;
    }

    pub fn set_clock_now_result_unix_nanos(&mut self, unix_nanos: i64) {
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::ClockNow {
            unix_nanos,
        });
        self.clock_now_result_unix_nanos = Some(unix_nanos);
    }

    pub fn set_fetch_response(&mut self, response: super::FetchResponse) {
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::Fetch {
            result: gowasm_host_types::FetchResult::Response {
                response: gowasm_host_types::FetchResponse {
                    status_code: response.status_code,
                    status: response.status.clone(),
                    url: response.url.clone(),
                    headers: response
                        .headers
                        .iter()
                        .map(|header| gowasm_host_types::FetchHeader {
                            name: header.name.clone(),
                            values: header.values.clone(),
                        })
                        .collect(),
                    body: response.body.clone(),
                },
            },
        });
        self.fetch_result = Some(FetchResult::Response { response });
    }

    pub fn set_fetch_response_start(
        &mut self,
        session_id: u64,
        response: super::FetchResponseStart,
    ) {
        self.record_capability_response(
            gowasm_host_types::VmReplayCapabilityResponse::FetchResponseStart {
                session_id,
                response: gowasm_host_types::FetchResponseStart {
                    status_code: response.status_code,
                    status: response.status.clone(),
                    url: response.url.clone(),
                    headers: response
                        .headers
                        .iter()
                        .map(|header| gowasm_host_types::FetchHeader {
                            name: header.name.clone(),
                            values: header.values.clone(),
                        })
                        .collect(),
                },
            },
        );
        self.fetch_response_start = Some(PendingFetchResponseStart {
            session_id,
            response,
        });
    }

    pub fn set_fetch_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::Fetch {
            result: gowasm_host_types::FetchResult::Error {
                message: message.clone(),
            },
        });
        self.fetch_result = Some(FetchResult::Error { message });
    }

    pub fn advance_timers(
        &mut self,
        program: &Program,
        elapsed_nanos: i64,
        fired_at_unix_nanos: Option<i64>,
    ) -> Result<(), VmError> {
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::Sleep {
            elapsed_nanos,
            fired_at_unix_nanos,
        });
        if elapsed_nanos <= 0
            && self.sleeping_goroutines.is_empty()
            && self.time_channel_timers.is_empty()
            && self.context_deadline_timers.is_empty()
        {
            return Ok(());
        }

        let mut ready = Vec::new();
        for (goroutine, remaining_nanos) in &mut self.sleeping_goroutines {
            if *remaining_nanos <= elapsed_nanos {
                ready.push(*goroutine);
            } else {
                *remaining_nanos -= elapsed_nanos;
            }
        }

        for goroutine in ready {
            self.wake_goroutine(goroutine);
        }

        let mut ready_channels = Vec::new();
        for timer in &mut self.time_channel_timers {
            if timer.remaining_nanos <= elapsed_nanos {
                ready_channels.push(timer.channel_id);
                timer.remaining_nanos = 0;
            } else {
                timer.remaining_nanos -= elapsed_nanos;
            }
        }
        self.time_channel_timers
            .retain(|timer| timer.remaining_nanos > 0);

        let fired_at_unix_nanos = fired_at_unix_nanos.unwrap_or(0);
        for channel_id in ready_channels {
            self.send_to_channel_value(program, channel_id, timer_time_value(fired_at_unix_nanos))?;
        }

        let mut ready_contexts = Vec::new();
        for timer in &mut self.context_deadline_timers {
            if timer.remaining_nanos <= elapsed_nanos {
                ready_contexts.push(timer.context_id);
                timer.remaining_nanos = 0;
            } else {
                timer.remaining_nanos -= elapsed_nanos;
            }
        }
        self.context_deadline_timers
            .retain(|timer| timer.remaining_nanos > 0);
        for context_id in ready_contexts {
            let _ = self.cancel_context_with_reason(
                program,
                context_id,
                ContextErrorKind::DeadlineExceeded,
            )?;
        }
        Ok(())
    }

    pub fn acknowledge_fetch_start(&mut self) {
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::FetchStart);
    }

    pub fn acknowledge_fetch_body_chunk(&mut self) {
        self.record_capability_response(
            gowasm_host_types::VmReplayCapabilityResponse::FetchBodyChunk,
        );
    }

    pub fn acknowledge_fetch_body_abort(&mut self) {
        self.record_capability_response(
            gowasm_host_types::VmReplayCapabilityResponse::FetchBodyAbort,
        );
    }

    pub fn acknowledge_cooperative_yield(&mut self) {
        self.record_capability_response(gowasm_host_types::VmReplayCapabilityResponse::Yield);
    }

    pub(crate) fn capability_requests_enabled(&self) -> bool {
        self.capability_requests_enabled
    }

    pub(crate) fn take_current_time_unix_nanos(&mut self) -> Option<i64> {
        self.clock_now_result_unix_nanos
            .take()
            .or(self.fixed_time_now_override_unix_nanos)
    }

    pub fn take_fetch_result(&mut self) -> Option<FetchResult> {
        self.fetch_result.take()
    }

    pub(crate) fn take_fetch_response_start(&mut self) -> Option<PendingFetchResponseStart> {
        self.fetch_response_start.take()
    }

    pub(crate) fn allocate_fetch_session_id(&mut self) -> u64 {
        let session_id = self.next_fetch_session_id;
        self.next_fetch_session_id += 1;
        session_id
    }

    pub(crate) fn http_request_upload_state(&self) -> Option<HttpRequestUploadState> {
        self.http_request_upload.clone()
    }

    pub(crate) fn set_http_request_upload_state(&mut self, state: Option<HttpRequestUploadState>) {
        self.http_request_upload = state;
    }

    pub(crate) fn take_pending_http_request_context(&mut self) -> Option<Value> {
        self.pending_http_request_context.take()
    }

    pub(crate) fn set_pending_http_request_context(&mut self, context: Option<Value>) {
        self.pending_http_request_context = context;
    }

    pub fn apply_fetch_response_chunk(
        &mut self,
        session_id: u64,
        result: FetchResponseChunkResult,
    ) -> bool {
        self.record_capability_response(
            gowasm_host_types::VmReplayCapabilityResponse::FetchResponseChunk {
                session_id,
                result: match &result {
                    FetchResponseChunkResult::Chunk { chunk, eof } => {
                        gowasm_host_types::FetchResponseChunkResult::Chunk {
                            chunk: chunk.clone(),
                            eof: *eof,
                        }
                    }
                    FetchResponseChunkResult::Error { message } => {
                        gowasm_host_types::FetchResponseChunkResult::Error {
                            message: message.clone(),
                        }
                    }
                },
            },
        );
        let Some(state) = self
            .http_response_bodies
            .values_mut()
            .find(|state| state.session_id == Some(session_id))
        else {
            return false;
        };

        match result {
            FetchResponseChunkResult::Chunk { chunk, eof } => {
                state.buffered = chunk;
                state.read_offset = 0;
                state.eof = eof;
                state.terminal_error = None;
                if eof {
                    state.session_id = None;
                }
            }
            FetchResponseChunkResult::Error { message } => {
                state.buffered.clear();
                state.read_offset = 0;
                state.eof = false;
                state.session_id = None;
                state.terminal_error =
                    Some(Value::error(normalize_fetch_chunk_error_text(&message)));
            }
        }

        true
    }

    pub fn finish_fetch_response_close(&mut self, session_id: u64) -> bool {
        self.record_capability_response(
            gowasm_host_types::VmReplayCapabilityResponse::FetchResponseClose { session_id },
        );
        let Some(state) = self
            .http_response_bodies
            .values_mut()
            .find(|state| state.session_id == Some(session_id))
        else {
            return false;
        };

        state.buffered.clear();
        state.read_offset = 0;
        state.closed = true;
        state.session_id = None;
        state.eof = true;
        state.terminal_error = None;
        true
    }

    pub(crate) fn sleep_current_goroutine(&mut self, duration_nanos: i64) {
        self.sleeping_goroutines
            .insert(self.current_goroutine_id(), duration_nanos);
        self.block_current_goroutine();
    }

    pub(crate) fn schedule_time_channel_send(&mut self, channel_id: u64, duration_nanos: i64) {
        self.time_channel_timers.push(TimeChannelTimer {
            channel_id,
            remaining_nanos: duration_nanos,
        });
    }

    pub(crate) fn schedule_context_deadline(&mut self, context_id: u64, duration_nanos: i64) {
        self.context_deadline_timers.push(ContextDeadlineTimer {
            context_id,
            remaining_nanos: duration_nanos,
        });
    }

    pub(crate) fn watch_context_done_channel(
        &mut self,
        channel_id: u64,
        context_id: u64,
        parent: Value,
    ) {
        self.context_done_watchers
            .entry(channel_id)
            .or_default()
            .push(ContextDoneWatcher { context_id, parent });
    }

    pub(crate) fn cancel_time_channel_send(&mut self, channel_id: u64) -> bool {
        let before = self.time_channel_timers.len();
        self.time_channel_timers
            .retain(|timer| timer.channel_id != channel_id);
        before != self.time_channel_timers.len()
    }

    pub(crate) fn cancel_context_deadline(&mut self, context_id: u64) -> bool {
        let before = self.context_deadline_timers.len();
        self.context_deadline_timers
            .retain(|timer| timer.context_id != context_id);
        before != self.context_deadline_timers.len()
    }

    pub(crate) fn reset_time_channel_send(
        &mut self,
        program: &Program,
        channel_id: u64,
        duration_nanos: i64,
        fired_at_unix_nanos: Option<i64>,
    ) -> Result<bool, VmError> {
        let was_active = self.cancel_time_channel_send(channel_id);
        if !was_active {
            let _ = self.drain_channel_buffered_value(program, channel_id)?;
        }

        if duration_nanos <= 0 {
            let fired_at_unix_nanos = fired_at_unix_nanos.unwrap_or(0);
            self.send_to_channel_value(program, channel_id, timer_time_value(fired_at_unix_nanos))?;
            return Ok(was_active);
        }

        self.schedule_time_channel_send(channel_id, duration_nanos);
        Ok(was_active)
    }

    pub(crate) fn cancel_context_with_reason(
        &mut self,
        program: &Program,
        context_id: u64,
        reason: ContextErrorKind,
    ) -> Result<bool, VmError> {
        self.cancel_context_with_error(program, context_id, super::context_error_value(reason))
    }

    pub(crate) fn cancel_context_with_error(
        &mut self,
        program: &Program,
        context_id: u64,
        error: Value,
    ) -> Result<bool, VmError> {
        let mut stack = vec![context_id];
        let mut canceled_any = false;

        while let Some(current_id) = stack.pop() {
            let maybe_cancel = match self.context_values.get_mut(&current_id) {
                Some(state) if state.err.is_none() => {
                    state.err = Some(error.clone());
                    Some((state.children.clone(), state.done_channel_id))
                }
                _ => None,
            };
            let Some((children, done_channel_id)) = maybe_cancel else {
                continue;
            };
            canceled_any = true;

            let _ = self.cancel_context_deadline(current_id);
            if let Some(channel_id) = done_channel_id {
                self.close_channel_by_id(program, channel_id)?;
            }
            stack.extend(children);
        }

        Ok(canceled_any)
    }

    pub(crate) fn should_request_cooperative_yield(&mut self) -> bool {
        if !self.capability_requests_enabled || self.instruction_yield_interval == 0 {
            return false;
        }
        self.instructions_since_yield = self.instructions_since_yield.saturating_add(1);
        if self.instructions_since_yield < self.instruction_yield_interval {
            return false;
        }
        self.instructions_since_yield = 0;
        true
    }

    pub(crate) fn reset_instruction_budget_state(&mut self) {
        self.executed_instruction_count = 0;
        self.remaining_instruction_budget = self.instruction_budget_limit;
    }

    pub(crate) fn ensure_instruction_budget(&self, program: &Program) -> Result<(), VmError> {
        if self.remaining_instruction_budget != Some(0) {
            return Ok(());
        }
        Err(VmError::InstructionBudgetExceeded {
            function: self
                .current_function_name(program)
                .unwrap_or_else(|_| "<engine>".into()),
            budget: self.instruction_budget_limit.unwrap_or(0),
            executed: self.executed_instruction_count,
        })
    }

    pub(crate) fn record_executed_instruction(&mut self) {
        self.executed_instruction_count = self.executed_instruction_count.saturating_add(1);
        if let Some(remaining) = self.remaining_instruction_budget.as_mut() {
            *remaining = remaining.saturating_sub(1);
        }
    }

    pub(crate) fn finish_executed_instruction(&mut self, program: &Program) {
        self.record_executed_instruction();
        self.maybe_collect_garbage();
        self.assert_runtime_invariants_if_enabled(program);
    }

    pub(crate) fn next_timer_duration_nanos(&self) -> Option<i64> {
        self.sleeping_goroutines
            .values()
            .copied()
            .chain(
                self.time_channel_timers
                    .iter()
                    .map(|timer| timer.remaining_nanos),
            )
            .chain(
                self.context_deadline_timers
                    .iter()
                    .map(|timer| timer.remaining_nanos),
            )
            .min()
    }
}

fn normalize_fetch_chunk_error_text(message: &str) -> String {
    if matches!(message, "context canceled" | "context deadline exceeded") {
        message.to_string()
    } else {
        format!("net/http: {message}")
    }
}
