use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, ErrorCategory,
    FetchBodyAbortRequest, FetchBodyChunkRequest, FetchBodyCompleteRequest,
    FetchBodyCompleteResult, FetchHeader, FetchRequest, FetchResponse, FetchResponseChunkRequest,
    FetchResponseChunkResult, FetchResponseCloseRequest, FetchResult, FetchStartRequest,
    WorkspaceFile,
};
use gowasm_vm::{
    CapabilityRequest as VmCapabilityRequest, FetchBodyAbortRequest as VmFetchBodyAbortRequest,
    FetchBodyChunkRequest as VmFetchBodyChunkRequest,
    FetchBodyCompleteRequest as VmFetchBodyCompleteRequest, FetchHeader as VmFetchHeader,
    FetchRequest as VmFetchRequest, FetchResponse as VmFetchResponse,
    FetchResponseChunkRequest as VmFetchResponseChunkRequest,
    FetchResponseCloseRequest as VmFetchResponseCloseRequest, FetchResult as VmFetchResult,
    FetchStartRequest as VmFetchStartRequest, Program, Vm,
};

use super::{apply_capability_result, map_vm_capability_request, Engine};
use crate::capability_bridge::{capability_request_name, capability_result_name};

fn empty_program() -> Program {
    Program {
        functions: Vec::new(),
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    }
}

#[test]
fn fetch_capability_request_maps_to_host_payloads() {
    let request = VmCapabilityRequest::Fetch {
        request: VmFetchRequest {
            method: "POST".into(),
            url: "https://example.com/upload".into(),
            headers: vec![VmFetchHeader {
                name: "Content-Type".into(),
                values: vec!["application/json".into()],
            }],
            body: b"{\"ok\":true}".to_vec(),
            context_deadline_unix_millis: None,
        },
    };

    assert_eq!(
        map_vm_capability_request(request),
        CapabilityRequest::Fetch {
            request: FetchRequest {
                method: "POST".into(),
                url: "https://example.com/upload".into(),
                headers: vec![FetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["application/json".into()],
                }],
                body: b"{\"ok\":true}".to_vec(),
                context_deadline_unix_millis: None,
            },
        }
    );
}

#[test]
fn staged_fetch_capability_requests_map_to_host_payloads() {
    let start_request = VmCapabilityRequest::FetchStart {
        request: VmFetchStartRequest {
            session_id: 7,
            method: "POST".into(),
            url: "https://example.com/upload".into(),
            headers: vec![VmFetchHeader {
                name: "Content-Type".into(),
                values: vec!["application/octet-stream".into()],
            }],
            context_deadline_unix_millis: None,
        },
    };
    assert_eq!(
        map_vm_capability_request(start_request),
        CapabilityRequest::FetchStart {
            request: FetchStartRequest {
                session_id: 7,
                method: "POST".into(),
                url: "https://example.com/upload".into(),
                headers: vec![FetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["application/octet-stream".into()],
                }],
                context_deadline_unix_millis: None,
            },
        }
    );

    let chunk_request = VmCapabilityRequest::FetchBodyChunk {
        request: VmFetchBodyChunkRequest {
            session_id: 7,
            chunk: b"payload".to_vec(),
        },
    };
    assert_eq!(
        map_vm_capability_request(chunk_request),
        CapabilityRequest::FetchBodyChunk {
            request: FetchBodyChunkRequest {
                session_id: 7,
                chunk: b"payload".to_vec(),
            },
        }
    );

    let complete_request = VmCapabilityRequest::FetchBodyComplete {
        request: VmFetchBodyCompleteRequest { session_id: 7 },
    };
    assert_eq!(
        map_vm_capability_request(complete_request),
        CapabilityRequest::FetchBodyComplete {
            request: FetchBodyCompleteRequest { session_id: 7 },
        }
    );

    let abort_request = VmCapabilityRequest::FetchBodyAbort {
        request: VmFetchBodyAbortRequest { session_id: 7 },
    };
    assert_eq!(
        map_vm_capability_request(abort_request),
        CapabilityRequest::FetchBodyAbort {
            request: FetchBodyAbortRequest { session_id: 7 },
        }
    );

    let response_chunk_request = VmCapabilityRequest::FetchResponseChunk {
        request: VmFetchResponseChunkRequest {
            session_id: 7,
            max_bytes: 16,
        },
    };
    assert_eq!(
        map_vm_capability_request(response_chunk_request),
        CapabilityRequest::FetchResponseChunk {
            request: FetchResponseChunkRequest {
                session_id: 7,
                max_bytes: 16,
            },
        }
    );

    let response_close_request = VmCapabilityRequest::FetchResponseClose {
        request: VmFetchResponseCloseRequest { session_id: 7 },
    };
    assert_eq!(
        map_vm_capability_request(response_close_request),
        CapabilityRequest::FetchResponseClose {
            request: FetchResponseCloseRequest { session_id: 7 },
        }
    );
}

#[test]
fn fetch_capability_response_result_is_stored_on_the_vm() {
    let program = empty_program();
    let mut vm = Vm::new();

    apply_capability_result(
        "main.go",
        &program,
        &mut vm,
        VmCapabilityRequest::Fetch {
            request: VmFetchRequest {
                method: "GET".into(),
                url: "https://example.com/data".into(),
                headers: Vec::new(),
                body: Vec::new(),
                context_deadline_unix_millis: None,
            },
        },
        CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 200,
                    status: "200 OK".into(),
                    url: "https://example.com/data".into(),
                    headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["application/json".into()],
                    }],
                    body: b"{\"name\":\"gowasm\"}".to_vec(),
                },
            },
        },
    )
    .expect("fetch capability result should apply");

    assert_eq!(
        vm.take_fetch_result(),
        Some(VmFetchResult::Response {
            response: VmFetchResponse {
                status_code: 200,
                status: "200 OK".into(),
                url: "https://example.com/data".into(),
                headers: vec![VmFetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["application/json".into()],
                }],
                body: b"{\"name\":\"gowasm\"}".to_vec(),
            },
        })
    );
}

#[test]
fn fetch_capability_error_result_is_stored_on_the_vm() {
    let program = empty_program();
    let mut vm = Vm::new();

    apply_capability_result(
        "main.go",
        &program,
        &mut vm,
        VmCapabilityRequest::Fetch {
            request: VmFetchRequest {
                method: "GET".into(),
                url: "https://example.com/data".into(),
                headers: Vec::new(),
                body: Vec::new(),
                context_deadline_unix_millis: None,
            },
        },
        CapabilityResult::Fetch {
            result: FetchResult::Error {
                message: "fetch failed for GET https://example.com/data: network down".into(),
            },
        },
    )
    .expect("fetch capability error result should apply");

    assert_eq!(
        vm.take_fetch_result(),
        Some(VmFetchResult::Error {
            message: "fetch failed for GET https://example.com/data: network down".into(),
        })
    );
}

#[test]
fn staged_fetch_completion_result_is_stored_on_the_vm() {
    let program = empty_program();
    let mut vm = Vm::new();

    apply_capability_result(
        "main.go",
        &program,
        &mut vm,
        VmCapabilityRequest::FetchBodyComplete {
            request: VmFetchBodyCompleteRequest { session_id: 7 },
        },
        CapabilityResult::FetchBodyComplete {
            result: FetchBodyCompleteResult::Response {
                response: FetchResponse {
                    status_code: 201,
                    status: "201 Created".into(),
                    url: "https://example.com/upload".into(),
                    headers: Vec::new(),
                    body: b"done".to_vec(),
                },
            },
        },
    )
    .expect("fetch body complete result should apply");

    assert_eq!(
        vm.take_fetch_result(),
        Some(VmFetchResult::Response {
            response: VmFetchResponse {
                status_code: 201,
                status: "201 Created".into(),
                url: "https://example.com/upload".into(),
                headers: Vec::new(),
                body: b"done".to_vec(),
            },
        })
    );
}

#[test]
fn capability_names_cover_fetch() {
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::Fetch {
            request: VmFetchRequest {
                method: "GET".into(),
                url: "https://example.com".into(),
                headers: Vec::new(),
                body: Vec::new(),
                context_deadline_unix_millis: None,
            },
        }),
        "fetch"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 204,
                    status: "204 No Content".into(),
                    url: "https://example.com".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        }),
        "fetch"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchStart {
            request: VmFetchStartRequest {
                session_id: 7,
                method: "POST".into(),
                url: "https://example.com/upload".into(),
                headers: Vec::new(),
                context_deadline_unix_millis: None,
            },
        }),
        "fetch_start"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchBodyChunk {
            request: VmFetchBodyChunkRequest {
                session_id: 7,
                chunk: b"payload".to_vec(),
            },
        }),
        "fetch_body_chunk"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchBodyComplete {
            request: VmFetchBodyCompleteRequest { session_id: 7 },
        }),
        "fetch_body_complete"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchBodyAbort {
            request: VmFetchBodyAbortRequest { session_id: 7 },
        }),
        "fetch_body_abort"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchResponseChunk {
            request: VmFetchResponseChunkRequest {
                session_id: 7,
                max_bytes: 16,
            },
        }),
        "fetch_response_chunk"
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::FetchResponseClose {
            request: VmFetchResponseCloseRequest { session_id: 7 },
        }),
        "fetch_response_close"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchStart),
        "fetch_start"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchBodyChunk),
        "fetch_body_chunk"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchBodyComplete {
            result: FetchBodyCompleteResult::Response {
                response: FetchResponse {
                    status_code: 204,
                    status: "204 No Content".into(),
                    url: "https://example.com/upload".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        }),
        "fetch_body_complete"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchBodyAbort),
        "fetch_body_abort"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchResponseChunk {
            result: FetchResponseChunkResult::Chunk {
                chunk: b"ok".to_vec(),
                eof: false,
            },
        }),
        "fetch_response_chunk"
    );
    assert_eq!(
        capability_result_name(&CapabilityResult::FetchResponseClose),
        "fetch_response_close"
    );
}

#[test]
fn yield_capability_round_trips_through_engine_mappings() {
    assert_eq!(
        map_vm_capability_request(VmCapabilityRequest::Yield),
        CapabilityRequest::Yield
    );
    assert_eq!(
        capability_request_name(&VmCapabilityRequest::Yield),
        "yield"
    );
    assert_eq!(capability_result_name(&CapabilityResult::Yield), "yield");

    let program = empty_program();
    let mut vm = Vm::new();
    apply_capability_result(
        "main.go",
        &program,
        &mut vm,
        VmCapabilityRequest::Yield,
        CapabilityResult::Yield,
    )
    .expect("yield capability result should apply");
}

#[test]
fn cancel_request_clears_paused_runs_and_rejects_future_resume() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: r#"
package main
import "time"

func main() {
    time.Sleep(1000000)
}
"#
            .into(),
        }],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(0),
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 1 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    assert_eq!(
        engine.handle_request(EngineRequest::Cancel),
        EngineResponse::Cancelled {
            category: ErrorCategory::RuntimeCancellation,
        }
    );

    match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep { unix_millis: 1 },
    }) {
        EngineResponse::Fatal { message, category } => {
            assert_eq!(
                message,
                format!("run `{run_id}` is not waiting for a capability result")
            );
            assert_eq!(category, ErrorCategory::ProtocolError);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn cancel_request_clears_paused_yield_runs_and_rejects_future_resume() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: r#"
package main

func main() {
    sum := 0
    for i := 0; i < 50000; i++ {
        sum += i
    }
    _ = sum
}
"#
            .into(),
        }],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Yield);
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    assert_eq!(
        engine.handle_request(EngineRequest::Cancel),
        EngineResponse::Cancelled {
            category: ErrorCategory::RuntimeCancellation,
        }
    );

    match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Yield,
    }) {
        EngineResponse::Fatal { message, category } => {
            assert_eq!(
                message,
                format!("run `{run_id}` is not waiting for a capability result")
            );
            assert_eq!(category, ErrorCategory::ProtocolError);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
