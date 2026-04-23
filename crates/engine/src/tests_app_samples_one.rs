use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchHeader, FetchRequest,
    FetchResponse, FetchResult, WorkspaceFile,
};

use super::Engine;

struct SampleCase {
    name: &'static str,
    files: Vec<WorkspaceFile>,
    host_time_unix_millis: Option<i64>,
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
fn app_style_sample_suite_one_runs_end_to_end() {
    for case in sample_cases() {
        run_case(case);
    }
}

fn sample_cases() -> Vec<SampleCase> {
    vec![
        SampleCase {
            name: "profile sync",
            files: vec![
                WorkspaceFile {
                    path: "main.go".into(),
                    contents: r#"
package main

import "fmt"

func main() {
    summary, err := buildSummary()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(summary)
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
    "net/http"
    "os"
    "path/filepath"
    "time"
)

type ProfileSummary struct {
    Banner string
    Status string
    Body string
    FetchedAt int64
}

func buildSummary() (string, error) {
    os.Setenv("HOME", "/users/alice")
    configRoot, err := os.UserConfigDir()
    if err != nil {
        return "", err
    }

    bannerPath := filepath.Join(configRoot, "demo", "banner.txt")
    banner, err := os.ReadFile(bannerPath)
    if err != nil {
        return "", err
    }

    resp, err := http.Get("https://example.com/profile")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 32)
    n, _ := resp.Body.Read(buf)
    payload, err := json.MarshalIndent(ProfileSummary{
        Banner: string(banner),
        Status: resp.Status,
        Body: string(buf[:n]),
        FetchedAt: time.Now().UnixMilli(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
"#
                    .into(),
                },
                WorkspaceFile {
                    path: "users/alice/.config/demo/banner.txt".into(),
                    contents: "hello-config".into(),
                },
            ],
            host_time_unix_millis: Some(900),
            steps: vec![SampleStep::Fetch {
                method: "GET",
                url: "https://example.com/profile",
                response_status_code: 200,
                response_status: "200 OK",
                response_url: "https://example.com/profile",
                response_headers: vec![FetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["text/plain".into()],
                }],
                response_body: b"remote-profile".to_vec(),
            }],
            expected_stdout: "{\n  \"Banner\": \"hello-config\",\n  \"Status\": \"200 OK\",\n  \"Body\": \"remote-profile\",\n  \"FetchedAt\": 900\n}\n",
        },
        SampleCase {
            name: "workspace manifest",
            files: vec![
                WorkspaceFile {
                    path: "main.go".into(),
                    contents: r#"
package main

import "fmt"

func main() {
    manifest, err := buildManifest()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(manifest)
}
"#
                    .into(),
                },
                WorkspaceFile {
                    path: "manifest.go".into(),
                    contents: r#"
package main

import (
    "encoding/json"
    fs "io/fs"
    "os"
    "sort"
)

type ManifestFile struct {
    Name string
    Body string
}

type Manifest struct {
    Root string
    Files []ManifestFile
}

func buildManifest() (string, error) {
    dirfs := os.DirFS("/notes")
    matches, err := fs.Glob(dirfs, "*.txt")
    if err != nil {
        return "", err
    }

    sort.Strings(matches)
    files := make([]ManifestFile, 0, len(matches))
    for _, name := range matches {
        body, err := fs.ReadFile(dirfs, name)
        if err != nil {
            return "", err
        }
        files = append(files, ManifestFile{
            Name: name,
            Body: string(body),
        })
    }

    payload, err := json.MarshalIndent(Manifest{
        Root: "/notes",
        Files: files,
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
"#
                    .into(),
                },
                WorkspaceFile {
                    path: "notes/beta.txt".into(),
                    contents: "second".into(),
                },
                WorkspaceFile {
                    path: "notes/alpha.txt".into(),
                    contents: "first".into(),
                },
            ],
            host_time_unix_millis: None,
            steps: Vec::new(),
            expected_stdout: "{\n  \"Root\": \"/notes\",\n  \"Files\": [\n    {\n      \"Name\": \"alpha.txt\",\n      \"Body\": \"first\"\n    },\n    {\n      \"Name\": \"beta.txt\",\n      \"Body\": \"second\"\n    }\n  ]\n}\n",
        },
        SampleCase {
            name: "remote json config",
            files: vec![
                WorkspaceFile {
                    path: "main.go".into(),
                    contents: r#"
package main

import "fmt"

func main() {
    summary, err := loadSummary()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(summary)
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
    "net/http"
    "os"
    "path/filepath"
    "time"
)

type RemoteProfile struct {
    Name string
    Count int
}

type Summary struct {
    Banner string
    Name string
    Count int
    ReadAt int64
}

func loadSummary() (string, error) {
    os.Setenv("HOME", "/users/alice")
    configRoot, err := os.UserConfigDir()
    if err != nil {
        return "", err
    }

    bannerPath := filepath.Join(configRoot, "demo", "banner.txt")
    banner, err := os.ReadFile(bannerPath)
    if err != nil {
        return "", err
    }

    resp, err := http.Get("https://example.com/profile.json")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 64)
    n, _ := resp.Body.Read(buf)

    var remote RemoteProfile
    if err := json.Unmarshal(buf[:n], &remote); err != nil {
        return "", err
    }

    payload, err := json.MarshalIndent(Summary{
        Banner: string(banner),
        Name: remote.Name,
        Count: remote.Count,
        ReadAt: time.Now().UnixMilli(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
"#
                    .into(),
                },
                WorkspaceFile {
                    path: "users/alice/.config/demo/banner.txt".into(),
                    contents: "hello-config".into(),
                },
            ],
            host_time_unix_millis: Some(1200),
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
                response_body: br#"{"Name":"Ada","Count":7}"#.to_vec(),
            }],
            expected_stdout: "{\n  \"Banner\": \"hello-config\",\n  \"Name\": \"Ada\",\n  \"Count\": 7,\n  \"ReadAt\": 1200\n}\n",
        },
    ]
}

fn run_case(case: SampleCase) {
    let mut engine = Engine::new();
    let mut response = engine.handle_request(EngineRequest::Run {
        files: case.files,
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: case.host_time_unix_millis,
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
