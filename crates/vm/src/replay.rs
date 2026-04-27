use gowasm_host_types::{
    FetchBodyCompleteResult as ReplayFetchBodyCompleteResult, FetchRequest as ReplayFetchRequest,
    FetchResponseChunkResult as ReplayFetchResponseChunkResult,
    FetchResponseStart as ReplayFetchResponseStart, FetchResult as ReplayFetchResult,
    VmReplayCapabilityRequest, VmReplayCapabilityResponse, VmReplayConfig, VmReplayEvent,
    VmReplayTerminal, VmReplayTrace,
};

use crate::{
    CapabilityRequest, FetchRequest, FetchResponseChunkResult, FetchResponseStart, Program,
    RunOutcome, Vm, VmError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmReplayError {
    InvalidTrace { message: String },
    RuntimeError(VmError),
    TraceMismatch { message: String },
}

pub fn replay_program_trace(program: &Program, trace: &VmReplayTrace) -> Result<(), VmReplayError> {
    let mut vm = Vm::new();
    vm.set_replay_config(&trace.config);
    vm.enable_replay_capture();

    let expected_requests = trace
        .events
        .iter()
        .filter_map(|event| match event {
            VmReplayEvent::CapabilityRequest { capability } => Some(capability.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let expected_responses = trace
        .events
        .iter()
        .filter_map(|event| match event {
            VmReplayEvent::CapabilityResponse { response } => Some(response.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut request_index = 0usize;
    let mut response_index = 0usize;
    let mut outcome = vm
        .start_program(program)
        .map_err(VmReplayError::RuntimeError)?;
    loop {
        match outcome {
            RunOutcome::Completed => break,
            RunOutcome::CapabilityRequest(kind) => {
                let actual_request = replay_capability_request(&kind);
                let expected_request = expected_requests.get(request_index).ok_or_else(|| {
                    VmReplayError::InvalidTrace {
                        message: format!(
                            "replay trace is missing capability request #{request_index}: {actual_request:?}"
                        ),
                    }
                })?;
                if expected_request != &actual_request {
                    return Err(VmReplayError::TraceMismatch {
                        message: format!(
                            "capability request #{request_index} mismatch: expected {expected_request:?}, got {actual_request:?}"
                        ),
                    });
                }
                request_index += 1;

                let response = expected_responses.get(response_index).ok_or_else(|| {
                    VmReplayError::InvalidTrace {
                        message: format!(
                            "replay trace is missing a response for capability request {actual_request:?}"
                        ),
                    }
                })?;
                apply_replay_response(program, &mut vm, &actual_request, response)?;
                response_index += 1;
                outcome = vm
                    .resume_program(program)
                    .map_err(VmReplayError::RuntimeError)?;
            }
        }
    }

    if request_index != expected_requests.len() {
        return Err(VmReplayError::InvalidTrace {
            message: format!(
                "replay trace recorded {} capability request(s) but only {} were consumed",
                expected_requests.len(),
                request_index
            ),
        });
    }
    if response_index != expected_responses.len() {
        return Err(VmReplayError::InvalidTrace {
            message: format!(
                "replay trace recorded {} capability response(s) but only {} were consumed",
                expected_responses.len(),
                response_index
            ),
        });
    }

    let replayed = vm
        .take_replay_trace()
        .ok_or_else(|| VmReplayError::InvalidTrace {
            message: "replay capture was not enabled on the replay VM".into(),
        })?;
    if replayed != *trace {
        return Err(VmReplayError::TraceMismatch {
            message: format!("replayed trace diverged from the recorded trace: {replayed:?}"),
        });
    }

    Ok(())
}

impl Vm {
    pub fn enable_replay_capture(&mut self) {
        self.replay_trace = Some(VmReplayTrace::with_config(self.current_replay_config()));
    }

    pub fn replay_trace(&self) -> Option<&VmReplayTrace> {
        self.replay_trace.as_ref()
    }

    pub fn take_replay_trace(&mut self) -> Option<VmReplayTrace> {
        self.replay_trace.take()
    }

    pub fn set_initial_rng_seed(&mut self, seed: u64) {
        self.initial_rng_seed = seed;
        self.rng_state = seed;
    }

    pub(crate) fn set_replay_config(&mut self, config: &VmReplayConfig) {
        match config.gc_allocation_threshold {
            Some(threshold) => self.set_gc_allocation_threshold(threshold),
            None => self.clear_gc_allocation_threshold(),
        }
        self.capability_requests_enabled = config.capability_requests_enabled;
        self.instruction_yield_interval = config.instruction_yield_interval;
        self.initial_rng_seed = config.initial_rng_seed;
        self.rng_state = config.initial_rng_seed;
        self.fixed_time_now_override_unix_nanos = config.fixed_time_now_override_unix_nanos;
        match config.instruction_budget {
            Some(budget) => self.set_instruction_budget(budget),
            None => self.clear_instruction_budget(),
        }
    }

    pub(crate) fn reset_replay_trace_for_run(&mut self) {
        if self.replay_trace.is_none() {
            return;
        }
        self.replay_trace = Some(VmReplayTrace::with_config(self.current_replay_config()));
    }

    pub(crate) fn current_replay_config(&self) -> VmReplayConfig {
        VmReplayConfig {
            capability_requests_enabled: self.capability_requests_enabled,
            instruction_yield_interval: self.instruction_yield_interval,
            instruction_budget: self.instruction_budget_limit,
            gc_allocation_threshold: self.gc_allocation_threshold,
            fixed_time_now_override_unix_nanos: self.fixed_time_now_override_unix_nanos,
            initial_rng_seed: self.initial_rng_seed,
        }
    }

    pub(crate) fn record_scheduler_pick(&mut self, goroutine_id: u64) {
        self.record_replay_event(VmReplayEvent::SchedulerPick { goroutine_id });
    }

    pub(crate) fn record_instruction_event(
        &mut self,
        goroutine_id: u64,
        function: usize,
        instruction_index: usize,
    ) {
        self.record_replay_event(VmReplayEvent::Instruction {
            goroutine_id,
            function,
            instruction_index,
        });
    }

    pub(crate) fn record_select_start(&mut self, case_count: usize, start_index: usize) {
        self.record_replay_event(VmReplayEvent::SelectStart {
            case_count,
            start_index,
        });
    }

    pub(crate) fn record_capability_request(&mut self, capability: &CapabilityRequest) {
        self.record_replay_event(VmReplayEvent::CapabilityRequest {
            capability: replay_capability_request(capability),
        });
    }

    pub(crate) fn record_capability_response(&mut self, response: VmReplayCapabilityResponse) {
        self.record_replay_event(VmReplayEvent::CapabilityResponse { response });
    }

    pub(crate) fn record_rng_seed(&mut self, seed: u64) {
        self.record_replay_event(VmReplayEvent::RandomSeed { seed });
    }

    pub(crate) fn record_rng_advance(&mut self, state: u64) {
        self.record_replay_event(VmReplayEvent::RandomAdvance { state });
    }

    pub(crate) fn record_terminal_completed(&mut self) {
        if let Some(trace) = self.replay_trace.as_mut() {
            trace.stdout = self.stdout.clone();
            trace.terminal = VmReplayTerminal::Completed;
        }
    }

    pub(crate) fn record_terminal_error(&mut self, error: &VmError) {
        if let Some(trace) = self.replay_trace.as_mut() {
            trace.stdout = self.stdout.clone();
            trace.terminal = VmReplayTerminal::RuntimeError {
                message: error.to_string(),
            };
        }
    }

    fn record_replay_event(&mut self, event: VmReplayEvent) {
        if let Some(trace) = self.replay_trace.as_mut() {
            trace.events.push(event);
        }
    }
}

fn replay_capability_request(kind: &CapabilityRequest) -> VmReplayCapabilityRequest {
    match kind {
        CapabilityRequest::ClockNow => VmReplayCapabilityRequest::ClockNow,
        CapabilityRequest::Sleep { duration_nanos } => VmReplayCapabilityRequest::Sleep {
            duration_nanos: *duration_nanos,
        },
        CapabilityRequest::Fetch { request } => VmReplayCapabilityRequest::Fetch {
            request: map_fetch_request(request),
        },
        CapabilityRequest::FetchStart { request } => VmReplayCapabilityRequest::FetchStart {
            session_id: request.session_id,
        },
        CapabilityRequest::FetchBodyChunk { request } => {
            VmReplayCapabilityRequest::FetchBodyChunk {
                session_id: request.session_id,
            }
        }
        CapabilityRequest::FetchBodyComplete { request } => {
            VmReplayCapabilityRequest::FetchBodyComplete {
                session_id: request.session_id,
            }
        }
        CapabilityRequest::FetchBodyAbort { request } => {
            VmReplayCapabilityRequest::FetchBodyAbort {
                session_id: request.session_id,
            }
        }
        CapabilityRequest::FetchResponseChunk { request } => {
            VmReplayCapabilityRequest::FetchResponseChunk {
                session_id: request.session_id,
                max_bytes: request.max_bytes,
            }
        }
        CapabilityRequest::FetchResponseClose { request } => {
            VmReplayCapabilityRequest::FetchResponseClose {
                session_id: request.session_id,
            }
        }
        CapabilityRequest::Yield => VmReplayCapabilityRequest::Yield,
    }
}

fn map_fetch_request(request: &FetchRequest) -> ReplayFetchRequest {
    ReplayFetchRequest {
        method: request.method.clone(),
        url: request.url.clone(),
        headers: request
            .headers
            .iter()
            .map(|header| gowasm_host_types::FetchHeader {
                name: header.name.clone(),
                values: header.values.clone(),
            })
            .collect(),
        body: request.body.clone(),
        context_deadline_unix_millis: request.context_deadline_unix_millis,
    }
}

fn apply_replay_response(
    program: &Program,
    vm: &mut Vm,
    request: &VmReplayCapabilityRequest,
    response: &VmReplayCapabilityResponse,
) -> Result<(), VmReplayError> {
    match (request, response) {
        (
            VmReplayCapabilityRequest::ClockNow,
            VmReplayCapabilityResponse::ClockNow { unix_nanos },
        ) => {
            vm.set_clock_now_result_unix_nanos(*unix_nanos);
            Ok(())
        }
        (
            VmReplayCapabilityRequest::Sleep { .. },
            VmReplayCapabilityResponse::Sleep {
                elapsed_nanos,
                fired_at_unix_nanos,
            },
        ) => vm
            .advance_timers(program, *elapsed_nanos, *fired_at_unix_nanos)
            .map_err(VmReplayError::RuntimeError),
        (VmReplayCapabilityRequest::Fetch { .. }, VmReplayCapabilityResponse::Fetch { result }) => {
            match result {
                ReplayFetchResult::Response { response } => {
                    vm.set_fetch_response(gowasm_vm_fetch_response(response));
                }
                ReplayFetchResult::Error { message } => {
                    vm.set_fetch_error(message.clone());
                }
            }
            Ok(())
        }
        (VmReplayCapabilityRequest::FetchStart { .. }, VmReplayCapabilityResponse::FetchStart) => {
            vm.acknowledge_fetch_start();
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchBodyChunk { .. },
            VmReplayCapabilityResponse::FetchBodyChunk,
        ) => {
            vm.acknowledge_fetch_body_chunk();
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchBodyAbort { .. },
            VmReplayCapabilityResponse::FetchBodyAbort,
        ) => {
            vm.acknowledge_fetch_body_abort();
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchBodyComplete { session_id },
            VmReplayCapabilityResponse::FetchResponseStart {
                session_id: response_session_id,
                response,
            },
        ) if session_id == response_session_id => {
            vm.set_fetch_response_start(*session_id, gowasm_vm_fetch_response_start(response));
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchBodyComplete { .. },
            VmReplayCapabilityResponse::Fetch { result },
        ) => {
            match result {
                ReplayFetchResult::Response { response } => {
                    vm.set_fetch_response(gowasm_vm_fetch_response(response));
                }
                ReplayFetchResult::Error { message } => {
                    vm.set_fetch_error(message.clone());
                }
            }
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchBodyComplete { .. },
            VmReplayCapabilityResponse::FetchBodyComplete { result },
        ) => {
            match result {
                ReplayFetchBodyCompleteResult::Response { response } => {
                    vm.set_fetch_response(gowasm_vm_fetch_response(response));
                }
                ReplayFetchBodyCompleteResult::ResponseStart { response } => {
                    vm.set_fetch_response_start(0, gowasm_vm_fetch_response_start(response));
                }
                ReplayFetchBodyCompleteResult::Error { message } => {
                    vm.set_fetch_error(message.clone());
                }
            }
            Ok(())
        }
        (
            VmReplayCapabilityRequest::FetchResponseChunk { session_id, .. },
            VmReplayCapabilityResponse::FetchResponseChunk {
                session_id: response_session_id,
                result,
            },
        ) if session_id == response_session_id => vm
            .apply_fetch_response_chunk(*session_id, gowasm_vm_fetch_response_chunk_result(result))
            .then_some(())
            .ok_or_else(|| {
                VmReplayError::RuntimeError(VmError::UnhandledPanic {
                    function: "<replay>".into(),
                    value: format!("unknown fetch response chunk session `{session_id}`"),
                })
            }),
        (
            VmReplayCapabilityRequest::FetchResponseClose { session_id },
            VmReplayCapabilityResponse::FetchResponseClose {
                session_id: response_session_id,
            },
        ) if session_id == response_session_id => vm
            .finish_fetch_response_close(*session_id)
            .then_some(())
            .ok_or_else(|| {
                VmReplayError::RuntimeError(VmError::UnhandledPanic {
                    function: "<replay>".into(),
                    value: format!("unknown fetch response close session `{session_id}`"),
                })
            }),
        (VmReplayCapabilityRequest::Yield, VmReplayCapabilityResponse::Yield) => {
            vm.acknowledge_cooperative_yield();
            Ok(())
        }
        (request, response) => Err(VmReplayError::TraceMismatch {
            message: format!("replay response {response:?} does not satisfy request {request:?}"),
        }),
    }
}

fn gowasm_vm_fetch_response(response: &gowasm_host_types::FetchResponse) -> crate::FetchResponse {
    crate::FetchResponse {
        status_code: response.status_code,
        status: response.status.clone(),
        url: response.url.clone(),
        headers: response
            .headers
            .iter()
            .map(|header| crate::FetchHeader {
                name: header.name.clone(),
                values: header.values.clone(),
            })
            .collect(),
        body: response.body.clone(),
    }
}

fn gowasm_vm_fetch_response_start(response: &ReplayFetchResponseStart) -> FetchResponseStart {
    FetchResponseStart {
        status_code: response.status_code,
        status: response.status.clone(),
        url: response.url.clone(),
        headers: response
            .headers
            .iter()
            .map(|header| crate::FetchHeader {
                name: header.name.clone(),
                values: header.values.clone(),
            })
            .collect(),
    }
}

fn gowasm_vm_fetch_response_chunk_result(
    result: &ReplayFetchResponseChunkResult,
) -> FetchResponseChunkResult {
    match result {
        ReplayFetchResponseChunkResult::Chunk { chunk, eof } => FetchResponseChunkResult::Chunk {
            chunk: chunk.clone(),
            eof: *eof,
        },
        ReplayFetchResponseChunkResult::Error { message } => FetchResponseChunkResult::Error {
            message: message.clone(),
        },
    }
}
