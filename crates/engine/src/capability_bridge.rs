use gowasm_host_types::{
    CapabilityRequest as HostCapabilityRequest, CapabilityResult, EngineResponse, ErrorCategory,
    FetchBodyAbortRequest as HostFetchBodyAbortRequest,
    FetchBodyChunkRequest as HostFetchBodyChunkRequest,
    FetchBodyCompleteRequest as HostFetchBodyCompleteRequest,
    FetchBodyCompleteResult as HostFetchBodyCompleteResult, FetchHeader as HostFetchHeader,
    FetchRequest as HostFetchRequest, FetchResponse as HostFetchResponse,
    FetchResponseChunkRequest as HostFetchResponseChunkRequest,
    FetchResponseChunkResult as HostFetchResponseChunkResult,
    FetchResponseCloseRequest as HostFetchResponseCloseRequest,
    FetchResponseStart as HostFetchResponseStart, FetchResult as HostFetchResult,
    FetchStartRequest as HostFetchStartRequest,
};
use gowasm_vm::{
    CapabilityRequest as VmCapabilityRequest, FetchBodyAbortRequest as VmFetchBodyAbortRequest,
    FetchBodyChunkRequest as VmFetchBodyChunkRequest,
    FetchBodyCompleteRequest as VmFetchBodyCompleteRequest,
    FetchBodyCompleteResult as VmFetchBodyCompleteResult, FetchHeader as VmFetchHeader,
    FetchRequest as VmFetchRequest, FetchResponse as VmFetchResponse,
    FetchResponseChunkRequest as VmFetchResponseChunkRequest,
    FetchResponseChunkResult as VmFetchResponseChunkResult,
    FetchResponseCloseRequest as VmFetchResponseCloseRequest,
    FetchResponseStart as VmFetchResponseStart, FetchResult as VmFetchResult,
    FetchStartRequest as VmFetchStartRequest, Program, Vm,
};

use crate::vm_run_error;

#[allow(clippy::result_large_err)]
pub(super) fn apply_capability_result(
    entry_path: &str,
    program: &Program,
    vm: &mut Vm,
    expected: VmCapabilityRequest,
    capability: CapabilityResult,
) -> Result<(), EngineResponse> {
    match (expected, capability) {
        (VmCapabilityRequest::ClockNow, CapabilityResult::ClockNow { unix_millis }) => {
            let unix_nanos =
                unix_millis
                    .checked_mul(1_000_000)
                    .ok_or_else(|| EngineResponse::Fatal {
                        message: format!(
                            "capability result `clock_now` value `{unix_millis}` overflowed Unix nanoseconds"
                        ),
                        category: ErrorCategory::ProtocolError,
                    })?;
            vm.set_clock_now_result_unix_nanos(unix_nanos);
            Ok(())
        }
        (
            VmCapabilityRequest::Sleep { duration_nanos },
            CapabilityResult::Sleep { unix_millis },
        ) => {
            let fired_at_unix_nanos =
                unix_millis
                    .checked_mul(1_000_000)
                    .ok_or_else(|| EngineResponse::Fatal {
                        message: format!(
                            "capability result `sleep` value `{unix_millis}` overflowed Unix nanoseconds"
                        ),
                        category: ErrorCategory::ProtocolError,
                    })?;
            vm.advance_timers(
                program,
                duration_nanos_to_millis(duration_nanos).saturating_mul(1_000_000),
                Some(fired_at_unix_nanos),
            )
            .map_err(|error| vm_run_error(entry_path, &error))?;
            Ok(())
        }
        (VmCapabilityRequest::Fetch { .. }, CapabilityResult::Fetch { result }) => {
            match map_host_fetch_result(result) {
                VmFetchResult::Response { response } => vm.set_fetch_response(response),
                VmFetchResult::Error { message } => vm.set_fetch_error(message),
            }
            Ok(())
        }
        (VmCapabilityRequest::FetchStart { .. }, CapabilityResult::FetchStart) => {
            vm.acknowledge_fetch_start();
            Ok(())
        }
        (VmCapabilityRequest::FetchBodyChunk { .. }, CapabilityResult::FetchBodyChunk) => {
            vm.acknowledge_fetch_body_chunk();
            Ok(())
        }
        (
            VmCapabilityRequest::FetchBodyComplete { request },
            CapabilityResult::FetchBodyComplete { result },
        ) => {
            match map_host_fetch_body_complete_result(result) {
                VmFetchBodyCompleteResult::Response { response } => vm.set_fetch_response(response),
                VmFetchBodyCompleteResult::ResponseStart { response } => {
                    vm.set_fetch_response_start(request.session_id, response);
                }
                VmFetchBodyCompleteResult::Error { message } => vm.set_fetch_error(message),
            }
            Ok(())
        }
        (VmCapabilityRequest::FetchBodyAbort { .. }, CapabilityResult::FetchBodyAbort) => {
            vm.acknowledge_fetch_body_abort();
            Ok(())
        }
        (
            VmCapabilityRequest::FetchResponseChunk { request },
            CapabilityResult::FetchResponseChunk { result },
        ) => {
            if vm.apply_fetch_response_chunk(
                request.session_id,
                map_host_fetch_response_chunk_result(result),
            ) {
                Ok(())
            } else {
                Err(EngineResponse::Fatal {
                    message: format!(
                        "run resumed with response chunk for unknown fetch session `{}`",
                        request.session_id
                    ),
                    category: ErrorCategory::ProtocolError,
                })
            }
        }
        (
            VmCapabilityRequest::FetchResponseClose { request },
            CapabilityResult::FetchResponseClose,
        ) => {
            if vm.finish_fetch_response_close(request.session_id) {
                Ok(())
            } else {
                Err(EngineResponse::Fatal {
                    message: format!(
                        "run resumed with response close for unknown fetch session `{}`",
                        request.session_id
                    ),
                    category: ErrorCategory::ProtocolError,
                })
            }
        }
        (VmCapabilityRequest::Yield, CapabilityResult::Yield) => {
            vm.acknowledge_cooperative_yield();
            Ok(())
        }
        (expected, capability) => Err(EngineResponse::Fatal {
            message: format!(
                "run resumed with capability result `{}` while waiting for `{}`",
                capability_result_name(&capability),
                capability_request_name(&expected),
            ),
            category: ErrorCategory::ProtocolError,
        }),
    }
}

pub(super) fn map_vm_capability_request(kind: VmCapabilityRequest) -> HostCapabilityRequest {
    match kind {
        VmCapabilityRequest::ClockNow => HostCapabilityRequest::ClockNow,
        VmCapabilityRequest::Sleep { duration_nanos } => HostCapabilityRequest::Sleep {
            duration_millis: duration_nanos_to_millis(duration_nanos),
        },
        VmCapabilityRequest::Fetch { request } => HostCapabilityRequest::Fetch {
            request: map_vm_fetch_request(request),
        },
        VmCapabilityRequest::FetchStart { request } => HostCapabilityRequest::FetchStart {
            request: map_vm_fetch_start_request(request),
        },
        VmCapabilityRequest::FetchBodyChunk { request } => HostCapabilityRequest::FetchBodyChunk {
            request: map_vm_fetch_body_chunk_request(request),
        },
        VmCapabilityRequest::FetchBodyComplete { request } => {
            HostCapabilityRequest::FetchBodyComplete {
                request: map_vm_fetch_body_complete_request(request),
            }
        }
        VmCapabilityRequest::FetchBodyAbort { request } => HostCapabilityRequest::FetchBodyAbort {
            request: map_vm_fetch_body_abort_request(request),
        },
        VmCapabilityRequest::FetchResponseChunk { request } => {
            HostCapabilityRequest::FetchResponseChunk {
                request: map_vm_fetch_response_chunk_request(request),
            }
        }
        VmCapabilityRequest::FetchResponseClose { request } => {
            HostCapabilityRequest::FetchResponseClose {
                request: map_vm_fetch_response_close_request(request),
            }
        }
        VmCapabilityRequest::Yield => HostCapabilityRequest::Yield,
    }
}

fn map_vm_fetch_request(request: VmFetchRequest) -> HostFetchRequest {
    HostFetchRequest {
        method: request.method,
        url: request.url,
        headers: request
            .headers
            .into_iter()
            .map(map_vm_fetch_header)
            .collect(),
        body: request.body,
        context_deadline_unix_millis: request.context_deadline_unix_millis,
    }
}

fn map_vm_fetch_start_request(request: VmFetchStartRequest) -> HostFetchStartRequest {
    HostFetchStartRequest {
        session_id: request.session_id,
        method: request.method,
        url: request.url,
        headers: request
            .headers
            .into_iter()
            .map(map_vm_fetch_header)
            .collect(),
        context_deadline_unix_millis: request.context_deadline_unix_millis,
    }
}

fn map_vm_fetch_body_chunk_request(request: VmFetchBodyChunkRequest) -> HostFetchBodyChunkRequest {
    HostFetchBodyChunkRequest {
        session_id: request.session_id,
        chunk: request.chunk,
    }
}

fn map_vm_fetch_body_complete_request(
    request: VmFetchBodyCompleteRequest,
) -> HostFetchBodyCompleteRequest {
    HostFetchBodyCompleteRequest {
        session_id: request.session_id,
    }
}

fn map_vm_fetch_body_abort_request(request: VmFetchBodyAbortRequest) -> HostFetchBodyAbortRequest {
    HostFetchBodyAbortRequest {
        session_id: request.session_id,
    }
}

fn map_vm_fetch_response_chunk_request(
    request: VmFetchResponseChunkRequest,
) -> HostFetchResponseChunkRequest {
    HostFetchResponseChunkRequest {
        session_id: request.session_id,
        max_bytes: request.max_bytes,
    }
}

fn map_vm_fetch_response_close_request(
    request: VmFetchResponseCloseRequest,
) -> HostFetchResponseCloseRequest {
    HostFetchResponseCloseRequest {
        session_id: request.session_id,
    }
}

fn map_vm_fetch_header(header: VmFetchHeader) -> HostFetchHeader {
    HostFetchHeader {
        name: header.name,
        values: header.values,
    }
}

fn map_host_fetch_response(response: HostFetchResponse) -> VmFetchResponse {
    VmFetchResponse {
        status_code: response.status_code,
        status: response.status,
        url: response.url,
        headers: response
            .headers
            .into_iter()
            .map(map_host_fetch_header)
            .collect(),
        body: response.body,
    }
}

fn map_host_fetch_result(result: HostFetchResult) -> VmFetchResult {
    match result {
        HostFetchResult::Response { response } => VmFetchResult::Response {
            response: map_host_fetch_response(response),
        },
        HostFetchResult::Error { message } => VmFetchResult::Error { message },
    }
}

fn map_host_fetch_response_start(response: HostFetchResponseStart) -> VmFetchResponseStart {
    VmFetchResponseStart {
        status_code: response.status_code,
        status: response.status,
        url: response.url,
        headers: response
            .headers
            .into_iter()
            .map(map_host_fetch_header)
            .collect(),
    }
}

fn map_host_fetch_body_complete_result(
    result: HostFetchBodyCompleteResult,
) -> VmFetchBodyCompleteResult {
    match result {
        HostFetchBodyCompleteResult::Response { response } => VmFetchBodyCompleteResult::Response {
            response: map_host_fetch_response(response),
        },
        HostFetchBodyCompleteResult::ResponseStart { response } => {
            VmFetchBodyCompleteResult::ResponseStart {
                response: map_host_fetch_response_start(response),
            }
        }
        HostFetchBodyCompleteResult::Error { message } => {
            VmFetchBodyCompleteResult::Error { message }
        }
    }
}

fn map_host_fetch_response_chunk_result(
    result: HostFetchResponseChunkResult,
) -> VmFetchResponseChunkResult {
    match result {
        HostFetchResponseChunkResult::Chunk { chunk, eof } => {
            VmFetchResponseChunkResult::Chunk { chunk, eof }
        }
        HostFetchResponseChunkResult::Error { message } => {
            VmFetchResponseChunkResult::Error { message }
        }
    }
}

fn map_host_fetch_header(header: HostFetchHeader) -> VmFetchHeader {
    VmFetchHeader {
        name: header.name,
        values: header.values,
    }
}

fn duration_nanos_to_millis(duration_nanos: i64) -> i64 {
    if duration_nanos <= 0 {
        0
    } else {
        ((duration_nanos - 1) / 1_000_000) + 1
    }
}

pub(super) fn capability_request_name(kind: &VmCapabilityRequest) -> &'static str {
    match kind {
        VmCapabilityRequest::ClockNow => "clock_now",
        VmCapabilityRequest::Sleep { .. } => "sleep",
        VmCapabilityRequest::Fetch { .. } => "fetch",
        VmCapabilityRequest::FetchStart { .. } => "fetch_start",
        VmCapabilityRequest::FetchBodyChunk { .. } => "fetch_body_chunk",
        VmCapabilityRequest::FetchBodyComplete { .. } => "fetch_body_complete",
        VmCapabilityRequest::FetchBodyAbort { .. } => "fetch_body_abort",
        VmCapabilityRequest::FetchResponseChunk { .. } => "fetch_response_chunk",
        VmCapabilityRequest::FetchResponseClose { .. } => "fetch_response_close",
        VmCapabilityRequest::Yield => "yield",
    }
}

pub(super) fn capability_result_name(kind: &CapabilityResult) -> &'static str {
    match kind {
        CapabilityResult::ClockNow { .. } => "clock_now",
        CapabilityResult::Sleep { .. } => "sleep",
        CapabilityResult::Fetch { .. } => "fetch",
        CapabilityResult::FetchStart => "fetch_start",
        CapabilityResult::FetchBodyChunk => "fetch_body_chunk",
        CapabilityResult::FetchBodyComplete { .. } => "fetch_body_complete",
        CapabilityResult::FetchBodyAbort => "fetch_body_abort",
        CapabilityResult::FetchResponseChunk { .. } => "fetch_response_chunk",
        CapabilityResult::FetchResponseClose => "fetch_response_close",
        CapabilityResult::Yield => "yield",
    }
}
