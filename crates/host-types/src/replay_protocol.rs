use serde::{Deserialize, Serialize};

use crate::{
    FetchBodyCompleteResult, FetchRequest, FetchResponseChunkResult, FetchResponseStart,
    FetchResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmReplayConfig {
    pub capability_requests_enabled: bool,
    pub instruction_yield_interval: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_budget: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gc_allocation_threshold: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_time_now_override_unix_nanos: Option<i64>,
    pub initial_rng_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VmReplayCapabilityRequest {
    ClockNow,
    Sleep { duration_nanos: i64 },
    Fetch { request: FetchRequest },
    FetchStart { session_id: u64 },
    FetchBodyChunk { session_id: u64 },
    FetchBodyComplete { session_id: u64 },
    FetchBodyAbort { session_id: u64 },
    FetchResponseChunk { session_id: u64, max_bytes: u32 },
    FetchResponseClose { session_id: u64 },
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VmReplayCapabilityResponse {
    ClockNow {
        unix_nanos: i64,
    },
    Sleep {
        elapsed_nanos: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fired_at_unix_nanos: Option<i64>,
    },
    Fetch {
        result: FetchResult,
    },
    FetchStart,
    FetchBodyChunk,
    FetchBodyComplete {
        result: FetchBodyCompleteResult,
    },
    FetchBodyAbort,
    FetchResponseStart {
        session_id: u64,
        response: FetchResponseStart,
    },
    FetchResponseChunk {
        session_id: u64,
        result: FetchResponseChunkResult,
    },
    FetchResponseClose {
        session_id: u64,
    },
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VmReplayTerminal {
    Completed,
    RuntimeError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VmReplayEvent {
    SchedulerPick {
        goroutine_id: u64,
    },
    Instruction {
        goroutine_id: u64,
        function: usize,
        instruction_index: usize,
    },
    SelectStart {
        case_count: usize,
        start_index: usize,
    },
    CapabilityRequest {
        capability: VmReplayCapabilityRequest,
    },
    CapabilityResponse {
        response: VmReplayCapabilityResponse,
    },
    RandomSeed {
        seed: u64,
    },
    RandomAdvance {
        state: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmReplayTrace {
    pub config: VmReplayConfig,
    pub events: Vec<VmReplayEvent>,
    pub stdout: String,
    pub terminal: VmReplayTerminal,
}

impl VmReplayTrace {
    pub fn with_config(config: VmReplayConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            stdout: String::new(),
            terminal: VmReplayTerminal::Completed,
        }
    }
}
