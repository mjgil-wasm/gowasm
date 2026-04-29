use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchBodyChunkRequest,
    FetchBodyCompleteRequest, FetchBodyCompleteResult, FetchHeader, FetchResponseStart,
    FetchStartRequest, WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

fn expect_streamed_fetch_start_with_deadline(
    response: EngineResponse,
    method: &str,
    url: &str,
    headers: Vec<FetchHeader>,
    context_deadline_unix_millis: Option<i64>,
) -> (u64, u64) {
    match response {
        EngineResponse::CapabilityRequest { run_id, capability } => match capability {
            CapabilityRequest::FetchStart { request } => {
                let session_id = request.session_id;
                assert_eq!(
                    request,
                    FetchStartRequest {
                        session_id,
                        method: method.into(),
                        url: url.into(),
                        headers,
                        context_deadline_unix_millis,
                    }
                );
                (run_id, session_id)
            }
            other => panic!("unexpected capability request: {other:?}"),
        },
        other => panic!("unexpected response: {other:?}"),
    }
}

fn resume_single_chunk_streamed_fetch(
    engine: &mut Engine,
    run_id: u64,
    session_id: u64,
    upload_chunk: &[u8],
    response_start: FetchResponseStart,
) -> EngineResponse {
    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchStart,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::FetchBodyChunk {
                    request: FetchBodyChunkRequest {
                        session_id,
                        chunk: upload_chunk.to_vec(),
                    },
                }
            );
            run_id
        }
        other => panic!("unexpected response after fetch_start resume: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchBodyChunk,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::FetchBodyComplete {
                    request: FetchBodyCompleteRequest { session_id },
                }
            );
            run_id
        }
        other => panic!("unexpected response after fetch_body_chunk resume: {other:?}"),
    };

    engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchBodyComplete {
            result: FetchBodyCompleteResult::ResponseStart {
                response: response_start,
            },
        },
    })
}

#[test]
fn run_emits_request_context_deadline_on_streamed_fetch_start() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "errors"
import "net/http"
import "time"

type customReader struct {
    data string
    offset int
}

func (r *customReader) Read(p []byte) (int, error) {
    if r.offset >= len(r.data) {
        return 0, errors.New("EOF")
    }

    remaining := []byte(r.data[r.offset:])
    if len(remaining) > len(p) {
        remaining = remaining[:len(p)]
    }

    n := copy(p, remaining)
    r.offset += n
    if r.offset >= len(r.data) {
        return n, nil
    }
    return n, nil
}

func main() {
    ctx, cancel := context.WithDeadline(context.Background(), time.UnixMilli(25))
    defer cancel()

    req, _ := http.NewRequestWithContext(ctx, "POST", "https://example.com/upload-deadline", &customReader{data: "payload"})
    req.Header.Set("Content-Type", "text/plain")
    _, _ = http.DefaultClient.Do(req)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: Some(10),
    });

    let _ = expect_streamed_fetch_start_with_deadline(
        response,
        "POST",
        "https://example.com/upload-deadline",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
        Some(25),
    );
}

#[test]
fn run_returns_canceled_request_context_errors_before_streamed_response_reads() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "errors"
import "fmt"
import "net/http"

type customReader struct {
    data string
    offset int
}

func (r *customReader) Read(p []byte) (int, error) {
    if r.offset >= len(r.data) {
        return 0, errors.New("EOF")
    }

    remaining := []byte(r.data[r.offset:])
    if len(remaining) > len(p) {
        remaining = remaining[:len(p)]
    }

    n := copy(p, remaining)
    r.offset += n
    if r.offset >= len(r.data) {
        return n, nil
    }
    return n, nil
}

func main() {
    ctx, cancel := context.WithCancel(context.Background())
    req, _ := http.NewRequestWithContext(ctx, "POST", "https://example.com/cancel-after-start", &customReader{data: "payload"})
    req.Header.Set("Content-Type", "text/plain")

    resp, err := http.DefaultClient.Do(req)
    fmt.Println(err == nil, resp != nil)

    cancel()

    buf := []byte("!!!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, readErr == context.Canceled, readErr)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let (run_id, session_id) = expect_streamed_fetch_start_with_deadline(
        response,
        "POST",
        "https://example.com/cancel-after-start",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
        None,
    );

    let response = resume_single_chunk_streamed_fetch(
        &mut engine,
        run_id,
        session_id,
        b"payload",
        FetchResponseStart {
            status_code: 200,
            status: "200 OK".into(),
            url: "https://example.com/cancel-after-start".into(),
            headers: Vec::new(),
        },
    );

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true true\n0 true context canceled\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
