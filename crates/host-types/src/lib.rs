use serde::{Deserialize, Serialize};

mod error_taxonomy;
mod module_protocol;
mod replay_protocol;

pub use error_taxonomy::ErrorCategory;
pub use module_protocol::{
    ModuleCacheFillRequest, ModuleCacheKey, ModuleCacheLookupRequest, ModuleCacheLookupResult,
    ModuleFetchRequest, ModuleFetchResult, ModuleGraphRoot, ModuleRequest, ModuleResult,
    ModuleSourceBundle,
};
pub use replay_protocol::{
    VmReplayCapabilityRequest, VmReplayCapabilityResponse, VmReplayConfig, VmReplayEvent,
    VmReplayTerminal, VmReplayTrace,
};

pub const ENGINE_PROTOCOL_VERSION: u32 = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceExcerpt {
    pub line: u32,
    pub text: String,
    pub highlight_start_column: u32,
    pub highlight_end_column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSourceSpan {
    pub path: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSourceLocation {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStackFrame {
    pub function: String,
    pub instruction_index: u32,
    pub source_span: Option<RuntimeSourceSpan>,
    pub source_location: Option<RuntimeSourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDiagnostic {
    pub root_message: String,
    #[serde(default)]
    pub category: ErrorCategory,
    pub stack_trace: Vec<RuntimeStackFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    #[serde(default)]
    pub category: ErrorCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<SourceSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_excerpt: Option<SourceExcerpt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeDiagnostic>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self::error_with_category(ErrorCategory::Tooling, message)
    }

    pub fn error_with_category(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Error,
            category,
            file_path: None,
            position: None,
            source_span: None,
            source_excerpt: None,
            suggested_action: None,
            runtime: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineInfo {
    pub protocol_version: u32,
    pub engine_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestRunnerKind {
    Package,
    Snippet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestResultDetails {
    pub subject_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_test: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchHeader {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
    pub body: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_deadline_unix_millis: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchStartRequest {
    pub session_id: u64,
    pub method: String,
    pub url: String,
    pub headers: Vec<FetchHeader>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_deadline_unix_millis: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchBodyChunkRequest {
    pub session_id: u64,
    pub chunk: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchBodyCompleteRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchBodyAbortRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchResponse {
    pub status_code: i64,
    pub status: String,
    #[serde(default)]
    pub url: String,
    pub headers: Vec<FetchHeader>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchResponseStart {
    pub status_code: i64,
    pub status: String,
    #[serde(default)]
    pub url: String,
    pub headers: Vec<FetchHeader>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FetchResult {
    Response { response: FetchResponse },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FetchBodyCompleteResult {
    Response { response: FetchResponse },
    ResponseStart { response: FetchResponseStart },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchResponseChunkRequest {
    pub session_id: u64,
    pub max_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchResponseCloseRequest {
    pub session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FetchResponseChunkResult {
    Chunk { chunk: Vec<u8>, eof: bool },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapabilityRequest {
    ClockNow,
    Sleep { duration_millis: i64 },
    Fetch { request: FetchRequest },
    FetchStart { request: FetchStartRequest },
    FetchBodyChunk { request: FetchBodyChunkRequest },
    FetchBodyComplete { request: FetchBodyCompleteRequest },
    FetchBodyAbort { request: FetchBodyAbortRequest },
    FetchResponseChunk { request: FetchResponseChunkRequest },
    FetchResponseClose { request: FetchResponseCloseRequest },
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapabilityResult {
    ClockNow { unix_millis: i64 },
    Sleep { unix_millis: i64 },
    Fetch { result: FetchResult },
    FetchStart,
    FetchBodyChunk,
    FetchBodyComplete { result: FetchBodyCompleteResult },
    FetchBodyAbort,
    FetchResponseChunk { result: FetchResponseChunkResult },
    FetchResponseClose,
    Yield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EngineRequest {
    Boot,
    LoadModuleGraph {
        modules: Vec<ModuleGraphRoot>,
    },
    Compile {
        files: Vec<WorkspaceFile>,
        entry_path: String,
    },
    Format {
        files: Vec<WorkspaceFile>,
    },
    Lint {
        files: Vec<WorkspaceFile>,
    },
    TestPackage {
        files: Vec<WorkspaceFile>,
        target_path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<String>,
    },
    TestSnippet {
        files: Vec<WorkspaceFile>,
        entry_path: String,
    },
    Run {
        files: Vec<WorkspaceFile>,
        entry_path: String,
        host_time_unix_nanos: Option<i64>,
        host_time_unix_millis: Option<i64>,
    },
    Resume {
        run_id: u64,
        capability: CapabilityResult,
    },
    ResumeModule {
        request_id: u64,
        module: ModuleResult,
    },
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EngineResponse {
    Ready {
        info: EngineInfo,
    },
    ModuleGraphResult {
        modules: Vec<ModuleSourceBundle>,
    },
    Diagnostics {
        diagnostics: Vec<Diagnostic>,
    },
    FormatResult {
        files: Vec<WorkspaceFile>,
        diagnostics: Vec<Diagnostic>,
    },
    LintResult {
        diagnostics: Vec<Diagnostic>,
    },
    TestResult {
        runner: TestRunnerKind,
        passed: bool,
        stdout: String,
        diagnostics: Vec<Diagnostic>,
        details: TestResultDetails,
    },
    RunResult {
        stdout: String,
        diagnostics: Vec<Diagnostic>,
    },
    CapabilityRequest {
        run_id: u64,
        capability: CapabilityRequest,
    },
    ModuleRequest {
        request_id: u64,
        module: ModuleRequest,
    },
    Cancelled {
        #[serde(default)]
        category: ErrorCategory,
    },
    Fatal {
        message: String,
        #[serde(default)]
        category: ErrorCategory,
    },
}

#[cfg(test)]
mod tests_core_protocol;
#[cfg(test)]
mod tests_module_protocol;
#[cfg(test)]
mod tests_tooling_protocol;
