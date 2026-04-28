use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchHeader, FetchRequest,
    FetchResponse, FetchResult, WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_can_pause_and_resume_for_http_post_form_helpers() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "net/http"
import "net/url"

func main() {
    values := make(url.Values)
    values["z"] = []string{"last"}
    values["a"] = []string{"two words"}

    resp, err := http.PostForm("https://example.com/form", values)
    fmt.Println(err == nil, resp != nil)

    client := http.DefaultClient
    clientResp, clientErr := client.PostForm("https://example.com/client", values)
    fmt.Println(clientErr == nil, clientResp != nil)
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
                        method: "POST".into(),
                        url: "https://example.com/form".into(),
                        headers: vec![FetchHeader {
                            name: "Content-Type".into(),
                            values: vec!["application/x-www-form-urlencoded".into()],
                        }],
                        body: b"a=two+words&z=last".to_vec(),
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
                    url: "https://example.com/form".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                },
            },
        },
    });

    match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(run_id, 0);
            assert_eq!(
                capability,
                CapabilityRequest::Fetch {
                    request: FetchRequest {
                        method: "POST".into(),
                        url: "https://example.com/client".into(),
                        headers: vec![FetchHeader {
                            name: "Content-Type".into(),
                            values: vec!["application/x-www-form-urlencoded".into()],
                        }],
                        body: b"a=two+words&z=last".to_vec(),
                        context_deadline_unix_millis: None,
                    },
                }
            );
        }
        other => panic!("unexpected response after first resume: {other:?}"),
    }

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Fetch {
            result: FetchResult::Response {
                response: FetchResponse {
                    status_code: 204,
                    status: "204 No Content".into(),
                    url: "https://example.com/client".into(),
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
            assert_eq!(stdout, "true true\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response after second resume: {other:?}"),
    }
}
