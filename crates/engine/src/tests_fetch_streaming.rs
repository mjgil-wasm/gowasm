use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchBodyChunkRequest,
    FetchBodyCompleteRequest, FetchBodyCompleteResult, FetchHeader, FetchResponse,
    FetchResponseChunkRequest, FetchResponseChunkResult, FetchResponseCloseRequest,
    FetchResponseStart, FetchResult, FetchStartRequest, WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

fn expect_streamed_fetch_start(
    response: EngineResponse,
    method: &str,
    url: &str,
    headers: Vec<FetchHeader>,
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
                        context_deadline_unix_millis: None,
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

fn resume_single_chunk_streamed_fetch_read(
    engine: &mut Engine,
    run_id: u64,
    session_id: u64,
    upload_chunk: &[u8],
    response_start: FetchResponseStart,
    response_chunk: &[u8],
    expected_read_size: u32,
) -> EngineResponse {
    let response = resume_single_chunk_streamed_fetch(
        engine,
        run_id,
        session_id,
        upload_chunk,
        response_start,
    );

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::FetchResponseChunk {
                    request: FetchResponseChunkRequest {
                        session_id,
                        max_bytes: expected_read_size,
                    },
                }
            );
            run_id
        }
        other => panic!("unexpected response after fetch_body_complete resume: {other:?}"),
    };

    engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchResponseChunk {
            result: FetchResponseChunkResult::Chunk {
                chunk: response_chunk.to_vec(),
                eof: false,
            },
        },
    })
}

fn resume_streamed_fetch_and_close_response(
    engine: &mut Engine,
    run_id: u64,
    session_id: u64,
    upload_chunk: &[u8],
    response_start: FetchResponseStart,
) -> EngineResponse {
    let response = resume_single_chunk_streamed_fetch(
        engine,
        run_id,
        session_id,
        upload_chunk,
        response_start,
    );

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::FetchResponseClose {
                    request: FetchResponseCloseRequest { session_id },
                }
            );
            run_id
        }
        other => panic!("unexpected response before response close: {other:?}"),
    };

    engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchResponseClose,
    })
}

fn resume_streamed_fetch_abort_after_chunk(
    engine: &mut Engine,
    run_id: u64,
    session_id: u64,
    chunk: &[u8],
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
                        chunk: chunk.to_vec(),
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
                CapabilityRequest::FetchBodyAbort {
                    request: gowasm_host_types::FetchBodyAbortRequest { session_id },
                }
            );
            run_id
        }
        other => panic!("unexpected response after fetch_body_chunk resume: {other:?}"),
    };

    engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::FetchBodyAbort,
    })
}

#[test]
fn run_can_pause_and_resume_for_http_client_do_fetch_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
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
    req, reqErr := http.NewRequest("PATCH", "https://example.com/api?q=1#frag", &customReader{data: "payload"})
    fmt.Println(reqErr == nil, req != nil)

    client := http.DefaultClient
    req.Header.Add("X-Test", "one")
    req.Header.Add("x-test", "two")
    req.Header.Set("Content-Type", "text/plain")

    resp, err := client.Do(req)
    fmt.Println(err == nil, resp != nil)
    fmt.Println(resp.Status, resp.StatusCode, resp.Header.Get("X-Reply"), resp.Body != nil)

    buf := []byte("!!!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), readErr == nil)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let (run_id, session_id) = expect_streamed_fetch_start(
        response,
        "PATCH",
        "https://example.com/api?q=1#frag",
        vec![
            FetchHeader {
                name: "Content-Type".into(),
                values: vec!["text/plain".into()],
            },
            FetchHeader {
                name: "X-Test".into(),
                values: vec!["one".into(), "two".into()],
            },
        ],
    );

    let response = resume_single_chunk_streamed_fetch_read(
        &mut engine,
        run_id,
        session_id,
        b"payload",
        FetchResponseStart {
            status_code: 202,
            status: "202 Accepted".into(),
            url: "https://example.com/api?q=1#frag".into(),
            headers: vec![FetchHeader {
                name: "x-reply".into(),
                values: vec!["ok".into()],
            }],
        },
        b"done",
        4,
    );

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "true true\ntrue true\n202 Accepted 202 ok true\n4 done true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_http_client_helper_fetch_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
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
    client := http.DefaultClient

    getResp, getErr := client.Get("https://example.com/get?q=1")
    fmt.Println(getErr == nil, getResp != nil)

    headResp, headErr := client.Head("https://example.com/head?q=1")
    fmt.Println(headErr == nil, headResp != nil)

    postResp, postErr := client.Post("https://example.com/post?q=1", "text/plain", &customReader{data: "payload"})
    fmt.Println(postErr == nil, postResp != nil)
    fmt.Println(postResp.Status, postResp.StatusCode, postResp.Header.Get("X-Reply"), postResp.Body != nil)

    buf := []byte("!!!!")
    n, readErr := postResp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), readErr == nil)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::Fetch {
                    request: gowasm_host_types::FetchRequest {
                        method: "GET".into(),
                        url: "https://example.com/get?q=1".into(),
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: None,
                    },
                }
            );
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 200,
                    status: "200 OK".into(),
                    url: "https://example.com/get?q=1".into(),
                    headers: Vec::new(),
                    body: b"get".to_vec(),
                },
            },
        },
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                CapabilityRequest::Fetch {
                    request: gowasm_host_types::FetchRequest {
                        method: "HEAD".into(),
                        url: "https://example.com/head?q=1".into(),
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: None,
                    },
                }
            );
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 204,
                    status: "204 No Content".into(),
                    url: "https://example.com/head?q=1".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        },
    });

    let (run_id, session_id) = expect_streamed_fetch_start(
        response,
        "POST",
        "https://example.com/post?q=1",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
    );

    let response = resume_single_chunk_streamed_fetch_read(
        &mut engine,
        run_id,
        session_id,
        b"payload",
        FetchResponseStart {
            status_code: 201,
            status: "201 Created".into(),
            url: "https://example.com/post?q=1".into(),
            headers: vec![FetchHeader {
                name: "x-reply".into(),
                values: vec!["ok".into()],
            }],
        },
        b"yes",
        4,
    );

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "true true\ntrue true\ntrue true\n201 Created 201 ok true\n3 yes true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_http_post_fetch_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
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
    resp, err := http.Post("https://example.com/upload?q=1", "text/plain", &customReader{data: "payload"})
    fmt.Println(err == nil, resp != nil)

    values := resp.Header.Values("X-Reply")
    fmt.Println(resp.Status, resp.StatusCode, len(values), values[1], resp.Body != nil)

    buf := []byte("!!!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), readErr == nil)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let (run_id, session_id) = expect_streamed_fetch_start(
        response,
        "POST",
        "https://example.com/upload?q=1",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
    );

    let response = resume_single_chunk_streamed_fetch_read(
        &mut engine,
        run_id,
        session_id,
        b"payload",
        FetchResponseStart {
            status_code: 201,
            status: "201 Created".into(),
            url: "https://example.com/upload?q=1".into(),
            headers: vec![FetchHeader {
                name: "x-reply".into(),
                values: vec!["one".into(), "two".into()],
            }],
        },
        b"done",
        4,
    );

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "true true\n201 Created 201 2 two true\n4 done true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_close_streamed_http_response_bodies() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
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
        return n, errors.New("EOF")
    }
    return n, nil
}

func main() {
    resp, err := http.Post("https://example.com/close", "text/plain", &customReader{data: "payload"})
    fmt.Println(err == nil, resp != nil)

    closeErr := resp.Body.Close()
    fmt.Println(resp.Status, resp.StatusCode, closeErr == nil)

    buf := []byte("!!!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, readErr)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let (run_id, session_id) = expect_streamed_fetch_start(
        response,
        "POST",
        "https://example.com/close",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
    );

    let response = resume_streamed_fetch_and_close_response(
        &mut engine,
        run_id,
        session_id,
        b"payload",
        FetchResponseStart {
            status_code: 202,
            status: "202 Accepted".into(),
            url: "https://example.com/close".into(),
            headers: Vec::new(),
        },
    );

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "true true\n202 Accepted 202 true\n0 http: read on closed response body\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_returns_request_body_read_errors_after_stream_abort() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "errors"
import "fmt"
import "net/http"

type flakyReader struct {
    reads int
}

func (r *flakyReader) Read(p []byte) (int, error) {
    if r.reads == 0 {
        p[0] = 'o'
        p[1] = 'k'
        r.reads = r.reads + 1
        return 2, nil
    }
    return 0, errors.New("boom")
}

func main() {
    resp, err := http.Post("https://example.com/upload", "text/plain", &flakyReader{})
    fmt.Println(resp == nil, err)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let (run_id, session_id) = expect_streamed_fetch_start(
        response,
        "POST",
        "https://example.com/upload",
        vec![FetchHeader {
            name: "Content-Type".into(),
            values: vec!["text/plain".into()],
        }],
    );

    let response = resume_streamed_fetch_abort_after_chunk(&mut engine, run_id, session_id, b"ok");

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true boom\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
