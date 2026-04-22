use serde_json::json;

use super::{
    CapabilityRequest, CapabilityResult, Diagnostic, EngineRequest, EngineResponse, ErrorCategory,
    FetchBodyAbortRequest, FetchBodyChunkRequest, FetchBodyCompleteRequest,
    FetchBodyCompleteResult, FetchHeader, FetchRequest, FetchResponse, FetchResponseChunkRequest,
    FetchResponseChunkResult, FetchResponseCloseRequest, FetchResponseStart, FetchResult,
    FetchStartRequest, Position, RuntimeDiagnostic, RuntimeSourceLocation, RuntimeSourceSpan,
    RuntimeStackFrame, Severity, SourceExcerpt, SourceSpan, WorkspaceFile,
};

#[test]
fn request_round_trips_through_json() {
    let request = EngineRequest::Run {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: "package main".into(),
        }],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(1_700_000_000_123_000_000),
        host_time_unix_millis: Some(1_700_000_000_123),
    };

    let json = serde_json::to_string(&request).expect("request should serialize");
    let decoded: EngineRequest = serde_json::from_str(&json).expect("request should parse");
    assert_eq!(decoded, request);
}

#[test]
fn response_round_trips_through_json() {
    let response = EngineResponse::Diagnostics {
        diagnostics: vec![Diagnostic {
            message: "bad token".into(),
            severity: Severity::Error,
            category: ErrorCategory::CompileError,
            file_path: Some("main.go".into()),
            position: Some(Position { line: 1, column: 2 }),
            source_span: Some(SourceSpan {
                start: Position { line: 1, column: 2 },
                end: Position {
                    line: 1,
                    column: 10,
                },
            }),
            source_excerpt: Some(SourceExcerpt {
                line: 1,
                text: "package main".into(),
                highlight_start_column: 2,
                highlight_end_column: 10,
            }),
            suggested_action: Some("Fix the source error and compile again.".into()),
            runtime: None,
        }],
    };

    let json = serde_json::to_string(&response).expect("response should serialize");
    let decoded: EngineResponse = serde_json::from_str(&json).expect("response should parse");
    assert_eq!(decoded, response);
}

#[test]
fn plain_diagnostic_omits_absent_optional_fields_on_serialize() {
    let response = EngineResponse::Diagnostics {
        diagnostics: vec![Diagnostic::error("bad token")],
    };

    let json = serde_json::to_value(&response).expect("response should serialize");
    assert_eq!(
        json,
        json!({
            "kind": "diagnostics",
            "diagnostics": [
                {
                    "message": "bad token",
                    "severity": "error",
                    "category": "tooling"
                }
            ]
        })
    );
}

#[test]
fn plain_diagnostic_accepts_legacy_null_optional_fields_on_deserialize() {
    let json = r#"{
  "kind": "diagnostics",
  "diagnostics": [
    {
      "message": "bad token",
      "severity": "error",
      "file_path": null,
      "position": null,
      "source_span": null,
      "source_excerpt": null,
      "suggested_action": null,
      "runtime": null
    }
  ]
}"#;

    let decoded: EngineResponse = serde_json::from_str(json).expect("response should parse");
    assert_eq!(
        decoded,
        EngineResponse::Diagnostics {
            diagnostics: vec![Diagnostic {
                message: "bad token".into(),
                severity: Severity::Error,
                category: ErrorCategory::Uncategorized,
                file_path: None,
                position: None,
                source_span: None,
                source_excerpt: None,
                suggested_action: None,
                runtime: None,
            }]
        }
    );
}

#[test]
fn capability_request_and_resume_round_trip_through_json() {
    let request = EngineRequest::Resume {
        run_id: 7,
        capability: CapabilityResult::ClockNow {
            unix_millis: 1_700_000_000_123,
        },
    };
    let request_json = serde_json::to_string(&request).expect("request should serialize");
    let decoded_request: EngineRequest =
        serde_json::from_str(&request_json).expect("request should parse");
    assert_eq!(decoded_request, request);

    let response = EngineResponse::CapabilityRequest {
        run_id: 7,
        capability: CapabilityRequest::ClockNow,
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);

    let sleep_request = EngineRequest::Resume {
        run_id: 8,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_456,
        },
    };
    let sleep_request_json =
        serde_json::to_string(&sleep_request).expect("request should serialize");
    let decoded_sleep_request: EngineRequest =
        serde_json::from_str(&sleep_request_json).expect("request should parse");
    assert_eq!(decoded_sleep_request, sleep_request);

    let sleep_response = EngineResponse::CapabilityRequest {
        run_id: 8,
        capability: CapabilityRequest::Sleep {
            duration_millis: 25,
        },
    };
    let sleep_response_json =
        serde_json::to_string(&sleep_response).expect("response should serialize");
    let decoded_sleep_response: EngineResponse =
        serde_json::from_str(&sleep_response_json).expect("response should parse");
    assert_eq!(decoded_sleep_response, sleep_response);

    let fetch_request = EngineRequest::Resume {
        run_id: 9,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 200,
                    status: "200 OK".into(),
                    url: "https://example.com".into(),
                    headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["text/plain".into()],
                    }],
                    body: b"hello".to_vec(),
                },
            },
        },
    };
    let fetch_request_json =
        serde_json::to_string(&fetch_request).expect("request should serialize");
    let decoded_fetch_request: EngineRequest =
        serde_json::from_str(&fetch_request_json).expect("request should parse");
    assert_eq!(decoded_fetch_request, fetch_request);

    let fetch_response = EngineResponse::CapabilityRequest {
        run_id: 9,
        capability: CapabilityRequest::Fetch {
            request: FetchRequest {
                method: "GET".into(),
                url: "https://example.com".into(),
                headers: vec![FetchHeader {
                    name: "Accept".into(),
                    values: vec!["text/plain".into()],
                }],
                body: Vec::new(),
                context_deadline_unix_millis: None,
            },
        },
    };
    let fetch_response_json =
        serde_json::to_string(&fetch_response).expect("response should serialize");
    let decoded_fetch_response: EngineResponse =
        serde_json::from_str(&fetch_response_json).expect("response should parse");
    assert_eq!(decoded_fetch_response, fetch_response);

    let yield_request = EngineRequest::Resume {
        run_id: 10,
        capability: CapabilityResult::Yield,
    };
    let yield_request_json =
        serde_json::to_string(&yield_request).expect("request should serialize");
    let decoded_yield_request: EngineRequest =
        serde_json::from_str(&yield_request_json).expect("request should parse");
    assert_eq!(decoded_yield_request, yield_request);

    let yield_response = EngineResponse::CapabilityRequest {
        run_id: 10,
        capability: CapabilityRequest::Yield,
    };
    let yield_response_json =
        serde_json::to_string(&yield_response).expect("response should serialize");
    let decoded_yield_response: EngineResponse =
        serde_json::from_str(&yield_response_json).expect("response should parse");
    assert_eq!(decoded_yield_response, yield_response);

    let fetch_error_request = EngineRequest::Resume {
        run_id: 10,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Error {
                message: "fetch failed for GET https://example.com: network down".into(),
            },
        },
    };
    let fetch_error_request_json =
        serde_json::to_string(&fetch_error_request).expect("request should serialize");
    let decoded_fetch_error_request: EngineRequest =
        serde_json::from_str(&fetch_error_request_json).expect("request should parse");
    assert_eq!(decoded_fetch_error_request, fetch_error_request);

    let fetch_start_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchStart {
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
        },
    };
    let fetch_start_response_json =
        serde_json::to_string(&fetch_start_response).expect("response should serialize");
    let decoded_fetch_start_response: EngineResponse =
        serde_json::from_str(&fetch_start_response_json).expect("response should parse");
    assert_eq!(decoded_fetch_start_response, fetch_start_response);

    let fetch_body_chunk_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchBodyChunk {
            request: FetchBodyChunkRequest {
                session_id: 7,
                chunk: b"payload".to_vec(),
            },
        },
    };
    let fetch_body_chunk_response_json =
        serde_json::to_string(&fetch_body_chunk_response).expect("response should serialize");
    let decoded_fetch_body_chunk_response: EngineResponse =
        serde_json::from_str(&fetch_body_chunk_response_json).expect("response should parse");
    assert_eq!(decoded_fetch_body_chunk_response, fetch_body_chunk_response);

    let fetch_body_complete_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchBodyComplete {
            request: FetchBodyCompleteRequest { session_id: 7 },
        },
    };
    let fetch_body_complete_response_json =
        serde_json::to_string(&fetch_body_complete_response).expect("response should serialize");
    let decoded_fetch_body_complete_response: EngineResponse =
        serde_json::from_str(&fetch_body_complete_response_json).expect("response should parse");
    assert_eq!(
        decoded_fetch_body_complete_response,
        fetch_body_complete_response
    );

    let fetch_body_complete_request = EngineRequest::Resume {
        run_id: 22,
        capability: CapabilityResult::FetchBodyComplete {
            result: FetchBodyCompleteResult::ResponseStart {
                response: FetchResponseStart {
                    status_code: 201,
                    status: "201 Created".into(),
                    url: "https://example.com/upload".into(),
                    headers: Vec::new(),
                },
            },
        },
    };
    let fetch_body_complete_request_json =
        serde_json::to_string(&fetch_body_complete_request).expect("request should serialize");
    let decoded_fetch_body_complete_request: EngineRequest =
        serde_json::from_str(&fetch_body_complete_request_json).expect("request should parse");
    assert_eq!(
        decoded_fetch_body_complete_request,
        fetch_body_complete_request
    );

    let fetch_response_chunk_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchResponseChunk {
            request: FetchResponseChunkRequest {
                session_id: 7,
                max_bytes: 16,
            },
        },
    };
    let fetch_response_chunk_response_json =
        serde_json::to_string(&fetch_response_chunk_response).expect("response should serialize");
    let decoded_fetch_response_chunk_response: EngineResponse =
        serde_json::from_str(&fetch_response_chunk_response_json).expect("response should parse");
    assert_eq!(
        decoded_fetch_response_chunk_response,
        fetch_response_chunk_response
    );

    let fetch_response_chunk_request = EngineRequest::Resume {
        run_id: 22,
        capability: CapabilityResult::FetchResponseChunk {
            result: FetchResponseChunkResult::Chunk {
                chunk: b"done".to_vec(),
                eof: false,
            },
        },
    };
    let fetch_response_chunk_request_json =
        serde_json::to_string(&fetch_response_chunk_request).expect("request should serialize");
    let decoded_fetch_response_chunk_request: EngineRequest =
        serde_json::from_str(&fetch_response_chunk_request_json).expect("request should parse");
    assert_eq!(
        decoded_fetch_response_chunk_request,
        fetch_response_chunk_request
    );

    let fetch_response_close_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchResponseClose {
            request: FetchResponseCloseRequest { session_id: 7 },
        },
    };
    let fetch_response_close_response_json =
        serde_json::to_string(&fetch_response_close_response).expect("response should serialize");
    let decoded_fetch_response_close_response: EngineResponse =
        serde_json::from_str(&fetch_response_close_response_json).expect("response should parse");
    assert_eq!(
        decoded_fetch_response_close_response,
        fetch_response_close_response
    );

    let fetch_response_close_request = EngineRequest::Resume {
        run_id: 22,
        capability: CapabilityResult::FetchResponseClose,
    };
    let fetch_response_close_request_json =
        serde_json::to_string(&fetch_response_close_request).expect("request should serialize");
    let decoded_fetch_response_close_request: EngineRequest =
        serde_json::from_str(&fetch_response_close_request_json).expect("request should parse");
    assert_eq!(
        decoded_fetch_response_close_request,
        fetch_response_close_request
    );

    let fetch_body_abort_response = EngineResponse::CapabilityRequest {
        run_id: 22,
        capability: CapabilityRequest::FetchBodyAbort {
            request: FetchBodyAbortRequest { session_id: 7 },
        },
    };
    let fetch_body_abort_response_json =
        serde_json::to_string(&fetch_body_abort_response).expect("response should serialize");
    let decoded_fetch_body_abort_response: EngineResponse =
        serde_json::from_str(&fetch_body_abort_response_json).expect("response should parse");
    assert_eq!(decoded_fetch_body_abort_response, fetch_body_abort_response);

    let fetch_body_abort_request = EngineRequest::Resume {
        run_id: 22,
        capability: CapabilityResult::FetchBodyAbort,
    };
    let fetch_body_abort_request_json =
        serde_json::to_string(&fetch_body_abort_request).expect("request should serialize");
    let decoded_fetch_body_abort_request: EngineRequest =
        serde_json::from_str(&fetch_body_abort_request_json).expect("request should parse");
    assert_eq!(decoded_fetch_body_abort_request, fetch_body_abort_request);
}

#[test]
fn cancel_request_and_response_round_trip_through_json() {
    let request = EngineRequest::Cancel;
    let request_json = serde_json::to_string(&request).expect("request should serialize");
    let decoded_request: EngineRequest =
        serde_json::from_str(&request_json).expect("request should parse");
    assert_eq!(decoded_request, request);

    let response = EngineResponse::Cancelled {
        category: ErrorCategory::RuntimeCancellation,
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}

#[test]
fn runtime_diagnostic_round_trips_through_json() {
    let response = EngineResponse::RunResult {
        stdout: String::new(),
        diagnostics: vec![Diagnostic {
            message:
                "division by zero in function `explode`\nstack trace:\n  at explode (main.go:6:5)"
                    .into(),
            severity: Severity::Error,
            category: ErrorCategory::RuntimeTrap,
            file_path: Some("main.go".into()),
            position: Some(Position { line: 6, column: 5 }),
            source_span: Some(SourceSpan {
                start: Position { line: 6, column: 5 },
                end: Position {
                    line: 6,
                    column: 16,
                },
            }),
            source_excerpt: Some(SourceExcerpt {
                line: 6,
                text: "    _ = 1 / value".into(),
                highlight_start_column: 5,
                highlight_end_column: 16,
            }),
            suggested_action: None,
            runtime: Some(RuntimeDiagnostic {
                root_message: "division by zero in function `explode`".into(),
                category: ErrorCategory::RuntimeTrap,
                stack_trace: vec![RuntimeStackFrame {
                    function: "explode".into(),
                    instruction_index: 3,
                    source_span: Some(RuntimeSourceSpan {
                        path: "main.go".into(),
                        start: 42,
                        end: 53,
                    }),
                    source_location: Some(RuntimeSourceLocation {
                        path: "main.go".into(),
                        line: 6,
                        column: 5,
                        end_line: 6,
                        end_column: 16,
                    }),
                }],
            }),
        }],
    };

    let json = serde_json::to_string(&response).expect("response should serialize");
    let decoded: EngineResponse = serde_json::from_str(&json).expect("response should parse");
    assert_eq!(decoded, response);
}

#[test]
fn fatal_response_round_trips_through_json() {
    let response = EngineResponse::Fatal {
        message: "invalid engine request json".into(),
        category: ErrorCategory::ProtocolError,
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}
