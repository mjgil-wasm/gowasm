use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchHeader, FetchRequest,
    FetchResponse, FetchResult, WorkspaceFile,
};

use super::Engine;

struct SampleCase {
    name: &'static str,
    files: Vec<WorkspaceFile>,
    steps: Vec<SampleStep>,
    expected_stdout: &'static str,
}

enum SampleStep {
    Fetch {
        method: &'static str,
        url: &'static str,
        response_status_code: i64,
        response_status: &'static str,
        response_url: &'static str,
        response_headers: Vec<FetchHeader>,
        response_body: Vec<u8>,
    },
}

#[test]
fn reflect_and_tag_app_samples_run_end_to_end() {
    for case in sample_cases() {
        run_case(case);
    }
}

fn sample_cases() -> Vec<SampleCase> {
    vec![SampleCase {
        name: "remote tagged profile report",
        files: vec![
            WorkspaceFile {
                path: "main.go".into(),
                contents: r#"
package main

import "fmt"

func main() {
    report, err := buildReport()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(report)
}
"#
                .into(),
            },
            WorkspaceFile {
                path: "app.go".into(),
                contents: r#"
package main

import (
    "encoding/json"
    "fmt"
    "net/http"
    "reflect"
)

type RemoteProfile struct {
    Name string `json:"name"`
    Hidden string `json:"-"`
    Alias string `json:"alias_name"`
}

type Report struct {
    Summary string `json:"summary"`
    FieldTag string `json:"field_tag"`
    Alias string `json:"alias,omitempty"`
}

func buildReport() (string, error) {
    resp, err := http.Get("https://example.com/profile.json")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 64)
    n, _ := resp.Body.Read(buf)

    profile := RemoteProfile{Hidden: "keep"}
    if err := json.Unmarshal(buf[:n], &profile); err != nil {
        return "", err
    }

    payload, err := json.MarshalIndent(Report{
        Summary: fmt.Sprintf("%+v", profile),
        FieldTag: string(reflect.TypeOf(profile).Field(0).Tag),
        Alias: reflect.ValueOf(profile).Field(2).String(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
"#
                .into(),
            },
        ],
        steps: vec![SampleStep::Fetch {
            method: "GET",
            url: "https://example.com/profile.json",
            response_status_code: 200,
            response_status: "200 OK",
            response_url: "https://example.com/profile.json",
            response_headers: vec![FetchHeader {
                name: "Content-Type".into(),
                values: vec!["application/json".into()],
            }],
            response_body: br#"{"name":"Ada","Hidden":"skip","alias_name":"go"}"#.to_vec(),
        }],
        expected_stdout: concat!(
            "{\n",
            "  \"summary\": \"{Name:Ada Hidden:keep Alias:go}\",\n",
            "  \"field_tag\": \"json:\\\"name\\\"\",\n",
            "  \"alias\": \"go\"\n",
            "}\n",
        ),
    }]
}

fn run_case(case: SampleCase) {
    let mut engine = Engine::new();
    let mut response = engine.handle_request(EngineRequest::Run {
        files: case.files,
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    for step in case.steps {
        let (run_id, capability) = match response {
            EngineResponse::CapabilityRequest { run_id, capability } => (run_id, capability),
            other => panic!(
                "sample `{}` expected a capability request, got {other:?}",
                case.name
            ),
        };
        response = engine.handle_request(EngineRequest::Resume {
            run_id,
            capability: assert_and_build_capability_result(case.name, capability, step),
        });
    }

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout, case.expected_stdout,
                "sample `{}` produced unexpected stdout",
                case.name
            );
            assert!(
                diagnostics.is_empty(),
                "sample `{}` produced unexpected diagnostics: {diagnostics:?}",
                case.name
            );
        }
        other => panic!(
            "sample `{}` expected a run result, got {other:?}",
            case.name
        ),
    }
}

fn assert_and_build_capability_result(
    case_name: &str,
    capability: CapabilityRequest,
    step: SampleStep,
) -> CapabilityResult {
    match step {
        SampleStep::Fetch {
            method,
            url,
            response_status_code,
            response_status,
            response_url,
            response_headers,
            response_body,
        } => {
            assert_eq!(
                capability,
                CapabilityRequest::Fetch {
                    request: FetchRequest {
                        method: method.into(),
                        url: url.into(),
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: None,
                    },
                },
                "sample `{case_name}` expected fetch capability"
            );
            CapabilityResult::Fetch {
                result: FetchResult::Response {
                    response: FetchResponse {
                        status_code: response_status_code,
                        status: response_status.into(),
                        url: response_url.into(),
                        headers: response_headers,
                        body: response_body,
                    },
                },
            }
        }
    }
}
