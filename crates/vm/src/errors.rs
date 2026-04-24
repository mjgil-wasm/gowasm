use std::fmt;

use crate::span::FrameDebugInfo;
use gowasm_host_types::ErrorCategory;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchHeader {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
    pub body: Vec<u8>,
    pub context_deadline_unix_millis: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchStartRequest {
    pub session_id: u64,
    pub method: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
    pub context_deadline_unix_millis: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchBodyChunkRequest {
    pub session_id: u64,
    pub chunk: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchBodyCompleteRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchBodyAbortRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponseChunkRequest {
    pub session_id: u64,
    pub max_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponseCloseRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponse {
    pub status_code: i64,
    pub status: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponseStart {
    pub status_code: i64,
    pub status: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchResult {
    Response { response: FetchResponse },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchBodyCompleteResult {
    Response { response: FetchResponse },
    ResponseStart { response: FetchResponseStart },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchResponseChunkResult {
    Chunk { chunk: Vec<u8>, eof: bool },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityRequest {
    ClockNow,
    Sleep { duration_nanos: i64 },
    Fetch { request: FetchRequest },
    FetchStart { request: FetchStartRequest },
    FetchBodyChunk { request: FetchBodyChunkRequest },
    FetchBodyComplete { request: FetchBodyCompleteRequest },
    FetchBodyAbort { request: FetchBodyAbortRequest },
    FetchResponseChunk { request: FetchResponseChunkRequest },
    FetchResponseClose { request: FetchResponseCloseRequest },
    Yield,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VmError {
    #[error(transparent)]
    Traced(#[from] Box<TracedVmError>),
    #[error("unknown function id {function}")]
    UnknownFunction { function: usize },
    #[error("unknown goroutine id {goroutine}")]
    UnknownGoroutine { goroutine: u64 },
    #[error("unknown channel {channel} in function `{function}`")]
    UnknownChannel { function: String, channel: u64 },
    #[error("instruction pointer out of bounds in function `{function}`")]
    InvalidInstructionPointer { function: String },
    #[error("register {register} out of bounds in function `{function}`")]
    InvalidRegister { function: String, register: usize },
    #[error("global {global} out of bounds")]
    InvalidGlobal { global: usize },
    #[error("function `{function}` expected {expected} argument(s), found {actual}")]
    WrongArgumentCount {
        function: String,
        expected: usize,
        actual: usize,
    },
    #[error("`len` expects string, array, slice, map, or channel in function `{function}`")]
    InvalidLenArgument { function: String },
    #[error("`cap` expects an array, slice, or channel in function `{function}`")]
    InvalidCapArgument { function: String },
    #[error("`append` target must be a slice in function `{function}`")]
    InvalidAppendTarget { function: String },
    #[error("`delete` target must be a map in function `{function}`")]
    InvalidDeleteTarget { function: String },
    #[error("`{builtin}` expects {expected} in function `{function}`")]
    InvalidStringFunctionArgument {
        function: String,
        builtin: String,
        expected: String,
    },
    #[error("`strings.Join` expects a []string and string argument in function `{function}`")]
    InvalidJoinArgument { function: String },
    #[error("`strings.Repeat` count {count} must not be negative in function `{function}`")]
    NegativeRepeatCount { function: String, count: i64 },
    #[error("`{builtin}` expects {expected} in function `{function}`")]
    InvalidStrconvFunctionArgument {
        function: String,
        builtin: String,
        expected: String,
    },
    #[error("`strconv.FormatInt` base {base} must be between 2 and 36 in function `{function}`")]
    InvalidFormatIntBase { function: String, base: i64 },
    #[error("`make` length must evaluate to int in function `{function}`")]
    InvalidMakeLength { function: String },
    #[error("`make` length {len} must not be negative in function `{function}`")]
    NegativeMakeLength { function: String, len: i64 },
    #[error("`make` capacity must evaluate to int in function `{function}`")]
    InvalidMakeCapacity { function: String },
    #[error("`make` capacity {cap} must not be negative in function `{function}`")]
    NegativeMakeCapacity { function: String, cap: i64 },
    #[error("`make` capacity {cap} must be >= length {len} in function `{function}`")]
    MakeCapacityLessThanLength {
        function: String,
        len: i64,
        cap: i64,
    },
    #[error("`copy` target must be a slice in function `{function}`")]
    InvalidCopyTarget { function: String },
    #[error("`copy` source must be a slice in function `{function}`")]
    InvalidCopySource { function: String },
    #[error("`range` target must be an array, slice, map, or string in function `{function}`")]
    InvalidRangeTarget { function: String },
    #[error("unsupported operands for `+` in function `{function}`: left {left}, right {right}")]
    InvalidAddOperands {
        function: String,
        left: String,
        right: String,
    },
    #[error("unsupported operand for `!` in function `{function}`: {operand}")]
    InvalidNotOperand { function: String, operand: String },
    #[error("unsupported operand for unary `-` in function `{function}`: {operand}")]
    InvalidNegateOperand { function: String, operand: String },
    #[error(
        "unsupported operands for `{op}` in function `{function}`: left {left}, right {right}"
    )]
    InvalidArithmeticOperands {
        function: String,
        op: String,
        left: String,
        right: String,
    },
    #[error("division by zero in function `{function}`: left {left}, right {right}")]
    DivisionByZero {
        function: String,
        left: String,
        right: String,
    },
    #[error(
        "unsupported operands for `{op}` in function `{function}`: left {left}, right {right}"
    )]
    InvalidComparisonOperands {
        function: String,
        op: String,
        left: String,
        right: String,
    },
    #[error("branch condition must evaluate to bool in function `{function}`")]
    InvalidConditionValue { function: String },
    #[error(
        "index target must be an array, slice, map, or string in function `{function}`; got {target}"
    )]
    InvalidIndexTarget { function: String, target: String },
    #[error("comma-ok lookup target must be a map in function `{function}`; got {target}")]
    InvalidMapLookupTarget { function: String, target: String },
    #[error("type assertion to `{target}` failed in function `{function}`")]
    TypeAssertionFailed { function: String, target: String },
    #[error("index must evaluate to int in function `{function}`")]
    InvalidIndexValue { function: String },
    #[error("index {index} is out of bounds for length {len} in function `{function}`")]
    IndexOutOfBounds {
        function: String,
        index: i64,
        len: usize,
    },
    #[error("cannot assign into a nil map in function `{function}`; target {target}")]
    AssignToNilMap { function: String, target: String },
    #[error("field selector target must be a struct in function `{function}`; got {target}")]
    InvalidFieldTarget { function: String, target: String },
    #[error("channel operation target must be a channel in function `{function}`; got {target}")]
    InvalidChannelValue { function: String, target: String },
    #[error("close of nil channel in function `{function}`; target {channel}")]
    CloseNilChannel { function: String, channel: String },
    #[error("close of closed channel in function `{function}`; target {channel}")]
    CloseClosedChannel { function: String, channel: String },
    #[error("send on closed channel in function `{function}`; value {value}, target {channel}")]
    SendOnClosedChannel {
        function: String,
        channel: String,
        value: String,
    },
    #[error("pointer target must be dereferenced in function `{function}`")]
    InvalidDerefTarget { function: String },
    #[error("pointer target must be assignable in function `{function}`")]
    InvalidIndirectTarget { function: String },
    #[error("pointer projection target must be a pointer in function `{function}`")]
    InvalidPointerProjectionTarget { function: String },
    #[error("nil pointer dereference in function `{function}`")]
    NilPointerDereference { function: String },
    #[error("dangling pointer to frame {frame_id} in function `{function}`")]
    DanglingPointer { function: String, frame_id: u64 },
    #[error("builtin error target must be an error value in function `{function}`")]
    InvalidErrorValue { function: String },
    #[error("struct field `{field}` is not defined in function `{function}`")]
    UnknownField { function: String, field: String },
    #[error("concurrency opcode `{opcode}` is not implemented yet")]
    UnsupportedConcurrencyOpcode { opcode: String },
    #[error("unknown stdlib function id {function}")]
    UnknownStdlibFunction { function: u16 },
    #[error("call target must be a function value in function `{function}`; got {target}")]
    InvalidFunctionValue { function: String, target: String },
    #[error("function `{function}` returned {actual} value(s) but caller expected {expected}")]
    ReturnValueCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "unknown method `{method}` for receiver type {receiver_type} in function `{function}`"
    )]
    UnknownMethod {
        function: String,
        receiver_type: u32,
        method: String,
    },
    #[error(
        "mutating method `{method}` is not supported for this receiver in function `{function}`"
    )]
    UnsupportedMutatingMethod { function: String, method: String },
    #[error("panic in function `{function}`: {value}")]
    UnhandledPanic { function: String, value: String },
    #[error(
        "instruction budget exhausted in function `{function}` after {executed} instruction(s) (budget {budget})"
    )]
    InstructionBudgetExceeded {
        function: String,
        budget: u64,
        executed: u64,
    },
    #[error("all goroutines are blocked")]
    Deadlock,
    #[error("type conversion expects {expected} in function `{function}`")]
    TypeMismatch { function: String, expected: String },
    #[error("os.Exit({code})")]
    ProgramExit { code: i64 },
    #[error("capability request `{kind:?}`")]
    CapabilityRequest { kind: CapabilityRequest },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracedVmError {
    pub root_cause: VmError,
    pub stack_trace: Vec<FrameDebugInfo>,
}

impl TracedVmError {
    pub fn root_cause(&self) -> &VmError {
        self.root_cause.root_cause()
    }
}

impl fmt::Display for TracedVmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root_cause())?;
        if self.stack_trace.is_empty() {
            return Ok(());
        }
        write!(f, "\nstack trace:")?;
        for frame in &self.stack_trace {
            write!(f, "\n  at {}", frame.function)?;
            if let Some(location) = &frame.source_location {
                write!(
                    f,
                    " ({}:{}:{})",
                    location.path, location.line, location.column
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for TracedVmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.root_cause)
    }
}

impl VmError {
    pub fn root_cause(&self) -> &VmError {
        match self {
            VmError::Traced(error) => error.root_cause(),
            _ => self,
        }
    }

    pub fn category(&self) -> ErrorCategory {
        match self.root_cause() {
            VmError::UnhandledPanic { .. } => ErrorCategory::RuntimePanic,
            VmError::InstructionBudgetExceeded { .. } => ErrorCategory::RuntimeBudgetExhaustion,
            VmError::Deadlock => ErrorCategory::RuntimeDeadlock,
            VmError::ProgramExit { .. } => ErrorCategory::RuntimeExit,
            VmError::CapabilityRequest { .. } => ErrorCategory::ProtocolError,
            VmError::Traced(_) => unreachable!("root_cause() removes traced wrappers"),
            _ => ErrorCategory::RuntimeTrap,
        }
    }

    pub fn stack_trace(&self) -> &[FrameDebugInfo] {
        match self {
            VmError::Traced(error) => &error.stack_trace,
            _ => &[],
        }
    }
}
