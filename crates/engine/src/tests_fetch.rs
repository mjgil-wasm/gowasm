use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, ErrorCategory, FetchHeader,
    FetchRequest, FetchResponse, FetchResult, WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_can_resume_http_get_fetch_errors_as_net_http_errors() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/unreachable")
    fmt.Println(resp == nil, err)
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
                    request: FetchRequest {
                        method: "GET".into(),
                        url: "https://example.com/unreachable".into(),
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
            result: FetchResult::Error {
                message: "fetch failed for GET https://example.com/unreachable: network down"
                    .into(),
            },
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "true net/http: fetch failed for GET https://example.com/unreachable: network down\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_emits_request_context_deadline_on_buffered_fetch_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "net/http"
import "time"

func main() {
    ctx, cancel := context.WithDeadline(context.Background(), time.UnixMilli(25))
    defer cancel()

    req, _ := http.NewRequestWithContext(ctx, "GET", "https://example.com/deadline", nil)
    _, _ = http.DefaultClient.Do(req)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: Some(10),
    });

    match response {
        EngineResponse::CapabilityRequest { capability, .. } => {
            assert_eq!(
                capability,
                CapabilityRequest::Fetch {
                    request: FetchRequest {
                        method: "GET".into(),
                        url: "https://example.com/deadline".into(),
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: Some(25),
                    },
                }
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn cancel_request_clears_paused_fetch_runs_and_rejects_future_resume() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "net/http"

func main() {
    http.Get("https://example.com/cancel-me")
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
                    request: FetchRequest {
                        method: "GET".into(),
                        url: "https://example.com/cancel-me".into(),
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

    assert_eq!(
        engine.handle_request(EngineRequest::Cancel),
        EngineResponse::Cancelled {
            category: ErrorCategory::RuntimeCancellation,
        }
    );

    match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 200,
                    status: "200 OK".into(),
                    url: "https://example.com/cancel-me".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        },
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
fn run_normalizes_empty_fetch_status_text_to_go_status() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/status")
    fmt.Println(err == nil, resp.Status, resp.StatusCode)
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
                    request: FetchRequest {
                        method: "GET".into(),
                        url: "https://example.com/status".into(),
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
                    status: String::new(),
                    url: "https://example.com/status".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true 200 OK 200\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_http_head_fetch_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Head("https://example.com/check?q=1")
    fmt.Println(err == nil, resp != nil)

    values := resp.Header.Values("X-Reply")
    fmt.Println(resp.Status, resp.StatusCode, len(values), values[0], resp.Body != nil)

    buf := []byte("!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, readErr != nil)
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
                    request: FetchRequest {
                        method: "HEAD".into(),
                        url: "https://example.com/check?q=1".into(),
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
                    url: "https://example.com/check?q=1".into(),
                    headers: vec![FetchHeader {
                        name: "x-reply".into(),
                        values: vec!["ok".into()],
                    }],
                    body: Vec::new(),
                },
            },
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true true\n204 No Content 204 1 ok true\n0 true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_returns_canceled_request_context_errors_before_fetch() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    ctx, cancel := context.WithCancel(context.Background())
    cancel()

    req, reqErr := http.NewRequestWithContext(ctx, "GET", "https://example.com/cancelled", nil)
    fmt.Println(reqErr == nil, req != nil)

    resp, err := http.DefaultClient.Do(req)
    fmt.Println(resp == nil, err == context.Canceled, err)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true true\ntrue true context canceled\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_returns_deadline_exceeded_request_context_errors_before_fetch() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "fmt"
import "net/http"
import "time"

func main() {
    ctx, cancel := context.WithDeadline(context.Background(), time.UnixMilli(10))
    defer cancel()

    req, reqErr := http.NewRequestWithContext(ctx, "GET", "https://example.com/expired", nil)
    fmt.Println(reqErr == nil, req != nil)

    resp, err := http.DefaultClient.Do(req)
    fmt.Println(resp == nil, err == context.DeadlineExceeded, err)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(20_000_000),
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true true\ntrue true context deadline exceeded\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
