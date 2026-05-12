mod access;
mod access_frame_ops;
mod access_maps;
mod calls;
mod channels;
mod channels_support;
mod errors;
mod execute;
mod fs_runtime;
mod gc_collect;
mod gc_metadata;
mod gc_roots;
mod host_runtime;
mod instruction;
mod map_value;
mod ops;
mod replay;
mod runtime_invariants;
mod runtime_state;
mod scheduler;
mod span;
mod stdlib;
mod type_inventory;
mod value;
mod value_invariants;

use serde::{Deserialize, Serialize};

pub use errors::{
    CapabilityRequest, FetchBodyAbortRequest, FetchBodyChunkRequest, FetchBodyCompleteRequest,
    FetchBodyCompleteResult, FetchHeader, FetchRequest, FetchResponse, FetchResponseChunkRequest,
    FetchResponseChunkResult, FetchResponseCloseRequest, FetchResponseStart, FetchResult,
    FetchStartRequest, TracedVmError, VmError,
};
pub use instruction::{CompareOp, Instruction, SelectCaseOp, SelectCaseOpKind};
pub(crate) use ops::function_name;
pub use replay::{replay_program_trace, VmReplayError};
pub use runtime_invariants::RuntimeInvariantMode;
pub(crate) use runtime_state::{
    ContextDeadlineTimer, ContextDoneWatcher, ContextState, HttpRequestBodyState,
    HttpRequestUploadPhase, HttpRequestUploadState, HttpResponseBodyState, MutexState, OnceState,
    PendingFetchResponseStart, RwMutexState, StringsReplacerState, TimeChannelTimer,
    WaitGroupState, WorkspaceFsFileState,
};
pub use scheduler::{GoroutineId, GoroutineStatus, SchedulerState};
pub use span::{
    program_debug_info, register_program_debug_info, FrameDebugInfo, FunctionDebugInfo,
    InstructionSourceSpan, ProgramDebugInfo, SourceFileDebugInfo, SourceLocation,
};
pub use stdlib::{
    resolve_stdlib_constant, resolve_stdlib_function, resolve_stdlib_method,
    resolve_stdlib_runtime_method, resolve_stdlib_value, stdlib_function_mutates_first_arg,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value, stdlib_function_variadic_param_type, stdlib_packages,
    StdlibConstantValue, StdlibFunctionId, StdlibPackage, StdlibValueInit,
};
pub(crate) use type_inventory::{concrete_type_for_value, explicit_concrete_type_for_value};
pub use type_inventory::{
    program_type_inventory, register_program_type_inventory, value_runtime_type, ConcreteType,
    ProgramTypeInventory, RuntimeChannelDirection, RuntimeTypeField, RuntimeTypeInfo,
    RuntimeTypeKind,
};
pub(crate) use value::{describe_value, format_value};
pub use value::{
    ArrayValue, ChannelValue, Float64, FunctionValue, MapValue, PointerTarget, PointerValue,
    SliceValue, Value, ValueData,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(pub u32);

pub const TYPE_NIL: TypeId = TypeId(0);
pub const TYPE_INT: TypeId = TypeId(1);
pub const TYPE_STRING: TypeId = TypeId(2);
pub const TYPE_BOOL: TypeId = TypeId(3);
pub const TYPE_ARRAY: TypeId = TypeId(4);
pub const TYPE_SLICE: TypeId = TypeId(5);
pub const TYPE_MAP: TypeId = TypeId(6);
pub const TYPE_ERROR: TypeId = TypeId(7);
pub const TYPE_POINTER: TypeId = TypeId(8);
pub const TYPE_FUNCTION: TypeId = TypeId(9);
pub const TYPE_CHANNEL: TypeId = TypeId(10);
pub const TYPE_FLOAT64: TypeId = TypeId(11);
pub const TYPE_REGEXP: TypeId = TypeId(12);
pub const TYPE_SYNC_WAIT_GROUP: TypeId = TypeId(13);
pub const TYPE_SYNC_WAIT_GROUP_PTR: TypeId = TypeId(14);
pub const TYPE_SYNC_ONCE: TypeId = TypeId(15);
pub const TYPE_SYNC_ONCE_PTR: TypeId = TypeId(16);
pub const TYPE_SYNC_MUTEX: TypeId = TypeId(17);
pub const TYPE_SYNC_MUTEX_PTR: TypeId = TypeId(18);
pub const TYPE_STRINGS_REPLACER: TypeId = TypeId(19);
pub const TYPE_TIME: TypeId = TypeId(20);
pub const TYPE_TIME_PTR: TypeId = TypeId(21);
pub const TYPE_TIME_DURATION: TypeId = TypeId(22);
pub const TYPE_TIME_TIMER: TypeId = TypeId(23);
pub const TYPE_TIME_TIMER_PTR: TypeId = TypeId(24);
pub const TYPE_EMPTY_STRUCT: TypeId = TypeId(25);
pub const TYPE_CONTEXT: TypeId = TypeId(26);
pub const TYPE_OS_DIR_FS: TypeId = TypeId(27);
pub const TYPE_FS_FILE: TypeId = TypeId(28);
pub const TYPE_FS_FILE_INFO: TypeId = TypeId(29);
pub const TYPE_FS_DIR_ENTRY: TypeId = TypeId(30);
pub const TYPE_FS_FILE_MODE: TypeId = TypeId(31);
pub const TYPE_HTTP_REQUEST: TypeId = TypeId(32);
pub const TYPE_HTTP_REQUEST_PTR: TypeId = TypeId(33);
pub const TYPE_HTTP_HEADER: TypeId = TypeId(34);
pub const TYPE_HTTP_RESPONSE: TypeId = TypeId(35);
pub const TYPE_HTTP_RESPONSE_PTR: TypeId = TypeId(36);
pub const TYPE_URL: TypeId = TypeId(37);
pub const TYPE_URL_PTR: TypeId = TypeId(38);
pub const TYPE_HTTP_RESPONSE_BODY: TypeId = TypeId(39);
pub const TYPE_HTTP_CLIENT: TypeId = TypeId(40);
pub const TYPE_HTTP_CLIENT_PTR: TypeId = TypeId(41);
pub const TYPE_URL_VALUES: TypeId = TypeId(42);
pub const TYPE_FS_SUB_FS: TypeId = TypeId(43);
pub const TYPE_HTTP_REQUEST_BODY: TypeId = TypeId(44);
pub const TYPE_URL_USERINFO: TypeId = TypeId(45);
pub const TYPE_URL_USERINFO_PTR: TypeId = TypeId(46);
pub const TYPE_SYNC_RW_MUTEX: TypeId = TypeId(47);
pub const TYPE_SYNC_RW_MUTEX_PTR: TypeId = TypeId(48);
pub const TYPE_BASE64_ENCODING: TypeId = TypeId(49);
pub const TYPE_BASE64_ENCODING_PTR: TypeId = TypeId(50);
pub const TYPE_ANY: TypeId = TypeId(51);
pub const TYPE_REFLECT_TYPE: TypeId = TypeId(52);
pub const TYPE_REFLECT_KIND: TypeId = TypeId(53);
pub const TYPE_REFLECT_STRUCT_FIELD: TypeId = TypeId(54);
pub const TYPE_REFLECT_RTYPE: TypeId = TypeId(55);
pub const TYPE_REFLECT_VALUE: TypeId = TypeId(56);
pub const TYPE_REFLECT_RVALUE: TypeId = TypeId(57);
pub const TYPE_REFLECT_STRUCT_TAG: TypeId = TypeId(58);
pub const TYPE_INT64: TypeId = TypeId(59);
pub const TYPE_TESTING_T: TypeId = TypeId(60);
pub const TYPE_TESTING_T_PTR: TypeId = TypeId(61);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeCheck {
    Int,
    Float64,
    String,
    Bool,
    Exact {
        type_id: TypeId,
        name: String,
    },
    Interface {
        name: String,
        methods: Vec<InterfaceMethodCheck>,
    },
    Struct {
        type_id: TypeId,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceMethodCheck {
    pub name: String,
    pub param_types: Vec<String>,
    pub result_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub param_count: usize,
    pub register_count: usize,
    pub code: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotedFieldAccess {
    Value,
    Pointer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotedFieldStep {
    pub field: String,
    pub access: PromotedFieldAccess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodBinding {
    pub receiver_type: TypeId,
    pub target_receiver_type: TypeId,
    pub name: String,
    pub function: usize,
    pub param_types: Vec<String>,
    pub result_types: Vec<String>,
    pub promoted_fields: Vec<PromotedFieldStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub methods: Vec<MethodBinding>,
    pub global_count: usize,
    pub entry_function: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeferredCallKind {
    Closure { function: Value },
    Function { function: usize },
    Stdlib { function: StdlibFunctionId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeferredCall {
    kind: DeferredCallKind,
    args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UnwindState {
    Return(Vec<Value>),
    Panic(Value),
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    id: u64,
    function: usize,
    pc: usize,
    registers: Vec<Value>,
    deferred: Vec<DeferredCall>,
    unwind: Option<UnwindState>,
    return_target: ReturnTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReturnTarget {
    None,
    One(usize),
    Many(Vec<usize>),
    Deferred,
    Callback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextErrorKind {
    Canceled,
    DeadlineExceeded,
}

fn context_error_value(kind: ContextErrorKind) -> Value {
    match kind {
        ContextErrorKind::Canceled => Value::error("context canceled"),
        ContextErrorKind::DeadlineExceeded => Value::error("context deadline exceeded"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Completed,
    CapabilityRequest(CapabilityRequest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcStats {
    pub allocation_threshold: Option<usize>,
    pub allocations_since_gc: usize,
    pub heap_cells: usize,
    pub live_heap_cells: usize,
    pub free_heap_cells: usize,
    pub last_freed_cells: usize,
    pub total_collections: u64,
    pub total_freed_cells: usize,
}

fn timer_time_value(unix_nanos: i64) -> Value {
    Value {
        typ: TYPE_TIME,
        data: ValueData::Struct(vec![("__time_unix_nanos".into(), Value::int(unix_nanos))]),
    }
}

const DEFAULT_GC_ALLOCATION_THRESHOLD: usize = 256;

#[derive(Debug, Default)]
pub struct Vm {
    goroutines: Vec<scheduler::Goroutine>,
    current_goroutine: usize,
    next_goroutine_id: u64,
    next_frame_id: u64,
    next_channel_id: u64,
    next_fetch_session_id: u64,
    next_select_id: u64,
    next_select_start: usize,
    heap_cells: Vec<Option<Value>>,
    free_heap_cells: Vec<usize>,
    gc_allocation_threshold: Option<usize>,
    allocations_since_gc: usize,
    last_gc_freed_cells: usize,
    total_gc_collections: u64,
    total_gc_freed_cells: usize,
    replay_trace: Option<gowasm_host_types::VmReplayTrace>,
    channels: Vec<channels::ChannelState>,
    globals: Vec<Value>,
    stdout: String,
    log_prefix: String,
    log_flags: i64,
    initial_rng_seed: u64,
    rng_state: u64,
    pub env: std::collections::HashMap<String, String>,
    pub workspace_files: std::collections::HashMap<String, String>,
    pub workspace_dirs: std::collections::BTreeSet<String>,
    fixed_time_now_override_unix_nanos: Option<i64>,
    clock_now_result_unix_nanos: Option<i64>,
    fetch_result: Option<FetchResult>,
    fetch_response_start: Option<PendingFetchResponseStart>,
    http_request_upload: Option<HttpRequestUploadState>,
    pending_http_request_context: Option<Value>,
    capability_requests_enabled: bool,
    paused_host_wait: bool,
    cancelled: bool,
    pending_panic_stack: Option<Vec<FrameDebugInfo>>,
    instruction_yield_interval: u64,
    instructions_since_yield: u64,
    instruction_budget_limit: Option<u64>,
    remaining_instruction_budget: Option<u64>,
    executed_instruction_count: u64,
    callback_result: Option<Vec<Value>>,
    callback_panic: Option<Value>,
    callback_capture_arg_indices: Option<Vec<usize>>,
    callback_captured_args: Option<Vec<Value>>,
    next_stdlib_object_id: u64,
    compiled_regexps: std::collections::HashMap<u64, regex::Regex>,
    sleeping_goroutines: std::collections::HashMap<GoroutineId, i64>,
    time_channel_timers: Vec<TimeChannelTimer>,
    context_deadline_timers: Vec<ContextDeadlineTimer>,
    wait_groups: std::collections::HashMap<u64, WaitGroupState>,
    once_values: std::collections::HashMap<u64, OnceState>,
    mutex_values: std::collections::HashMap<u64, MutexState>,
    rw_mutex_values: std::collections::HashMap<u64, RwMutexState>,
    workspace_fs_files: std::collections::HashMap<u64, WorkspaceFsFileState>,
    http_request_bodies: std::collections::HashMap<u64, HttpRequestBodyState>,
    http_response_bodies: std::collections::HashMap<u64, HttpResponseBodyState>,
    string_replacers: std::collections::HashMap<u64, StringsReplacerState>,
    context_values: std::collections::HashMap<u64, ContextState>,
    context_done_watchers: std::collections::HashMap<u64, Vec<ContextDoneWatcher>>,
    runtime_invariant_mode: RuntimeInvariantMode,
}

impl Vm {
    pub fn new() -> Self {
        let mut vm = Self::default();
        vm.set_gc_allocation_threshold(DEFAULT_GC_ALLOCATION_THRESHOLD);
        vm
    }

    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    pub fn set_time_now_override_unix_nanos(&mut self, unix_nanos: i64) {
        self.fixed_time_now_override_unix_nanos = Some(unix_nanos);
    }

    pub fn enable_capability_requests(&mut self) {
        self.capability_requests_enabled = true;
    }

    pub fn run_program(&mut self, program: &Program) -> Result<(), VmError> {
        match self.start_program(program)? {
            RunOutcome::Completed => Ok(()),
            RunOutcome::CapabilityRequest(kind) => Err(VmError::UnhandledPanic {
                function: self
                    .current_function_name(program)
                    .unwrap_or_else(|_| "<engine>".into()),
                value: format!("a host capability request `{kind:?}` escaped direct VM execution"),
            }),
        }
    }

    pub fn start_program(&mut self, program: &Program) -> Result<RunOutcome, VmError> {
        self.reset_scheduler();
        self.reset_replay_trace_for_run();
        self.heap_cells.clear();
        self.free_heap_cells.clear();
        self.allocations_since_gc = 0;
        self.last_gc_freed_cells = 0;
        self.total_gc_collections = 0;
        self.total_gc_freed_cells = 0;
        self.channels.clear();
        self.globals = vec![Value::nil(); program.global_count];
        self.stdout.clear();
        self.log_prefix.clear();
        self.log_flags = 0;
        self.rng_state = self.initial_rng_seed;
        self.clock_now_result_unix_nanos = None;
        self.fetch_result = None;
        self.fetch_response_start = None;
        self.http_request_upload = None;
        self.pending_http_request_context = None;
        self.callback_result = None;
        self.callback_panic = None;
        self.callback_capture_arg_indices = None;
        self.callback_captured_args = None;
        self.next_stdlib_object_id = 0;
        self.compiled_regexps.clear();
        self.sleeping_goroutines.clear();
        self.time_channel_timers.clear();
        self.context_deadline_timers.clear();
        self.wait_groups.clear();
        self.once_values.clear();
        self.mutex_values.clear();
        self.rw_mutex_values.clear();
        self.workspace_fs_files.clear();
        self.http_request_bodies.clear();
        self.http_response_bodies.clear();
        self.string_replacers.clear();
        self.context_values.clear();
        self.context_done_watchers.clear();
        self.pending_panic_stack = None;
        self.instructions_since_yield = 0;
        self.reset_instruction_budget_state();
        self.spawn_goroutine(program, program.entry_function, Vec::new())?;
        self.resume_program(program)
    }

    pub fn resume_program(&mut self, program: &Program) -> Result<RunOutcome, VmError> {
        if self.cancelled {
            return Ok(RunOutcome::Completed);
        }
        self.paused_host_wait = false;
        self.instructions_since_yield = 0;
        while self.has_live_goroutines() {
            if !self.advance_to_next_runnable() {
                if let Some(duration_nanos) = self.next_timer_duration_nanos() {
                    if self.capability_requests_enabled() {
                        self.paused_host_wait = true;
                        self.record_capability_request(&CapabilityRequest::Sleep {
                            duration_nanos,
                        });
                        self.assert_runtime_invariants_if_enabled(program);
                        return Ok(RunOutcome::CapabilityRequest(CapabilityRequest::Sleep {
                            duration_nanos,
                        }));
                    }
                    return Err(self.trace_runtime_error(
                        program,
                        VmError::UnhandledPanic {
                            function: self
                                .current_function_name(program)
                                .unwrap_or_else(|_| "<engine>".into()),
                            value: "pending timer wait requires a host-backed timer capability"
                                .into(),
                        },
                    ));
                }
                if self.has_blocked_goroutines() {
                    let error = VmError::Deadlock;
                    self.record_terminal_error(&error);
                    return Err(error);
                }
                let error = VmError::UnsupportedConcurrencyOpcode {
                    opcode: "scheduler blocked without runnable goroutine".into(),
                };
                self.record_terminal_error(&error);
                return Err(error);
            }
            if let Some(error) = self.take_pending_error_for_current_goroutine() {
                let error = self.trace_runtime_error(program, error);
                self.record_terminal_error(&error);
                return Err(error);
            }
            self.ensure_instruction_budget(program).map_err(|error| {
                let error = self.trace_runtime_error(program, error);
                self.record_terminal_error(&error);
                error
            })?;
            let instruction = self.fetch_next_instruction(program).map_err(|error| {
                let error = self.trace_runtime_error(program, error);
                self.record_terminal_error(&error);
                error
            })?;
            match self.execute_instruction(program, instruction) {
                Ok(()) => self.finish_executed_instruction(program),
                Err(error) => {
                    if let VmError::CapabilityRequest { kind } = error {
                        self.rewind_current_instruction(program)?;
                        self.paused_host_wait = true;
                        self.record_capability_request(&kind);
                        self.assert_runtime_invariants_if_enabled(program);
                        return Ok(RunOutcome::CapabilityRequest(kind));
                    }
                    self.finish_executed_instruction(program);
                    let error = self.trace_runtime_error(program, error);
                    self.record_terminal_error(&error);
                    return Err(error);
                }
            }
            if self.should_request_cooperative_yield() && self.has_live_goroutines() {
                self.paused_host_wait = true;
                self.record_capability_request(&CapabilityRequest::Yield);
                self.assert_runtime_invariants_if_enabled(program);
                return Ok(RunOutcome::CapabilityRequest(CapabilityRequest::Yield));
            }
        }

        self.record_terminal_completed();
        Ok(RunOutcome::Completed)
    }

    fn trace_runtime_error(&mut self, program: &Program, error: VmError) -> VmError {
        if matches!(error, VmError::Traced(_)) {
            return error;
        }
        let stack_trace = self
            .pending_panic_stack
            .take()
            .unwrap_or_else(|| self.current_stack_debug_info(program));
        if stack_trace.is_empty() {
            return error;
        }
        VmError::Traced(Box::new(TracedVmError {
            root_cause: error,
            stack_trace,
        }))
    }

    fn fetch_next_instruction(&mut self, program: &Program) -> Result<Instruction, VmError> {
        let (function_id, pc) = {
            let frame = self.current_frame();
            (frame.function, frame.pc)
        };
        self.record_instruction_event(self.current_goroutine_id().0, function_id, pc);
        let function = program
            .functions
            .get(function_id)
            .ok_or(VmError::UnknownFunction {
                function: function_id,
            })?;
        let instruction =
            function
                .code
                .get(pc)
                .cloned()
                .ok_or_else(|| VmError::InvalidInstructionPointer {
                    function: function.name.clone(),
                })?;
        self.current_frame_mut().pc += 1;
        Ok(instruction)
    }

    fn set_register(
        &mut self,
        program: &Program,
        register: usize,
        value: Value,
    ) -> Result<(), VmError> {
        self.debug_assert_value_invariants(program, &value);
        let frame = self.current_frame_mut();
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

    fn rewind_current_instruction(&mut self, program: &Program) -> Result<(), VmError> {
        let function = self.current_function_name(program)?;
        let frame = self.current_frame_mut();
        if frame.pc == 0 {
            return Err(VmError::InvalidInstructionPointer { function });
        }
        frame.pc -= 1;
        Ok(())
    }

    fn set_register_on_caller(
        &mut self,
        program: &Program,
        register: usize,
        value: Value,
    ) -> Result<(), VmError> {
        let frame = self.current_frame_mut();
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

    fn read_register(&self, program: &Program, register: usize) -> Result<Value, VmError> {
        let frame = self.current_frame();
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

    fn set_current_pc(&mut self, pc: usize) {
        self.current_frame_mut().pc = pc;
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bitwise;
#[cfg(test)]
mod tests_budget;
#[cfg(test)]
mod tests_builtins;
#[cfg(test)]
mod tests_channel_contention;
#[cfg(test)]
mod tests_channel_stress;
#[cfg(test)]
mod tests_channel_stress_large;
#[cfg(test)]
mod tests_channels;
#[cfg(test)]
mod tests_channels_closed;
#[cfg(test)]
mod tests_channels_more;
#[cfg(test)]
mod tests_error_taxonomy;
#[cfg(test)]
mod tests_error_text;
#[cfg(test)]
mod tests_frame_debug;
#[cfg(test)]
mod tests_functions;
#[cfg(test)]
mod tests_gc_auto;
#[cfg(test)]
mod tests_gc_collect;
#[cfg(test)]
mod tests_gc_metadata;
#[cfg(test)]
mod tests_gc_observability;
#[cfg(test)]
mod tests_gc_roots;
#[cfg(test)]
mod tests_gc_stress;
#[cfg(test)]
mod tests_globals;
#[cfg(test)]
mod tests_goroutines;
#[cfg(test)]
mod tests_interfaces;
#[cfg(test)]
mod tests_map_scalability;
#[cfg(test)]
mod tests_maps;
#[cfg(test)]
mod tests_multi_results;
#[cfg(test)]
mod tests_pointers;
#[cfg(test)]
mod tests_replay;
#[cfg(test)]
mod tests_runtime_invariants;
#[cfg(test)]
mod tests_scheduler;
#[cfg(test)]
mod tests_select_runtime;
#[cfg(test)]
mod tests_select_runtime_dense_ready;
#[cfg(test)]
mod tests_select_runtime_live_closed;
#[cfg(test)]
mod tests_select_runtime_multi_case;
#[cfg(test)]
mod tests_select_runtime_multi_fairness;
#[cfg(test)]
mod tests_select_runtime_nil_live;
#[cfg(test)]
mod tests_select_runtime_nil_recv;
#[cfg(test)]
mod tests_slice_array_semantics;
#[cfg(test)]
mod tests_stdlib;
#[cfg(test)]
mod tests_stdlib_errors;
#[cfg(test)]
mod tests_stdlib_path;
#[cfg(test)]
mod tests_stdlib_sort;
#[cfg(test)]
mod tests_stdlib_strings;
#[cfg(test)]
mod tests_stdlib_strings_fold;
#[cfg(test)]
mod tests_stdlib_strings_more;
#[cfg(test)]
mod tests_stdlib_strings_split_after_n;
#[cfg(test)]
mod tests_stdlib_strings_splitn;
#[cfg(test)]
mod tests_stdlib_unicode;
#[cfg(test)]
mod tests_stdlib_unicode_case;
#[cfg(test)]
mod tests_stdlib_unicode_more;
#[cfg(test)]
mod tests_structs;
#[cfg(test)]
mod tests_type_inventory;
#[cfg(test)]
mod tests_unwind;
#[cfg(test)]
mod tests_value_invariants;
