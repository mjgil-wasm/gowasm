use crate::{FetchResponseStart, GoroutineId, Value};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WaitGroupState {
    pub(crate) count: i64,
    pub(crate) waiters: Vec<GoroutineId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OnceState {
    pub(crate) done: bool,
    pub(crate) running: bool,
    pub(crate) waiters: Vec<GoroutineId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MutexState {
    pub(crate) locked: bool,
    pub(crate) waiters: Vec<GoroutineId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RwMutexState {
    pub(crate) reader_count: usize,
    pub(crate) reader_waiters: Vec<GoroutineId>,
    pub(crate) writer_active: bool,
    pub(crate) writer_waiters: Vec<GoroutineId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StringsReplacerState {
    pub(crate) pairs: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceFsFileState {
    pub(crate) path: String,
    pub(crate) closed: bool,
    pub(crate) is_dir: bool,
    pub(crate) source_is_os: bool,
    pub(crate) read_offset: usize,
    pub(crate) read_dir_offset: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HttpResponseBodyState {
    pub(crate) buffered: Vec<u8>,
    pub(crate) read_offset: usize,
    pub(crate) closed: bool,
    pub(crate) session_id: Option<u64>,
    pub(crate) eof: bool,
    pub(crate) terminal_error: Option<Value>,
    pub(crate) request_context: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpRequestBodyState {
    pub(crate) reader: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HttpRequestUploadPhase {
    Started,
    Streaming,
    Closing,
    AwaitingResponse,
    Aborting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpRequestUploadState {
    pub(crate) session_id: u64,
    pub(crate) phase: HttpRequestUploadPhase,
    pub(crate) error: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingFetchResponseStart {
    pub(crate) session_id: u64,
    pub(crate) response: FetchResponseStart,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ContextState {
    pub(crate) parent_id: Option<u64>,
    pub(crate) parent_value: Option<Value>,
    pub(crate) children: Vec<u64>,
    pub(crate) done_channel_id: Option<u64>,
    pub(crate) deadline_unix_nanos: Option<i64>,
    pub(crate) err: Option<Value>,
    pub(crate) values: Vec<(Value, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimeChannelTimer {
    pub(crate) channel_id: u64,
    pub(crate) remaining_nanos: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextDeadlineTimer {
    pub(crate) context_id: u64,
    pub(crate) remaining_nanos: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextDoneWatcher {
    pub(crate) context_id: u64,
    pub(crate) parent: Value,
}
