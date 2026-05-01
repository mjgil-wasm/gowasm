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

struct CompatCase {
    name: &'static str,
    files: Vec<WorkspaceFile>,
    host_time_unix_millis: Option<i64>,
    steps: Vec<CompatStep>,
    expected_stdout: &'static str,
}

enum CompatStep {
    ClockNow {
        unix_millis: i64,
    },
    Sleep {
        duration_millis: i64,
        unix_millis: i64,
    },
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
fn capability_backed_stdlib_cases_match_harness_expectations() {
    for case in capability_cases() {
        run_case(case);
    }
}

#[test]
fn io_fs_capability_case_matches_harness_expectations() {
    let case = capability_cases()
        .into_iter()
        .find(|case| case.name == "io/fs")
        .expect("io/fs capability case should exist");
    run_case(case);
}

#[test]
fn os_capability_case_matches_harness_expectations() {
    let case = capability_cases()
        .into_iter()
        .find(|case| case.name == "os")
        .expect("os capability case should exist");
    run_case(case);
}

fn capability_cases() -> Vec<CompatCase> {
    vec![
        CompatCase {
            name: "time",
            files: vec![main_file(
                r#"
package main
import "fmt"
import "time"

func main() {
    fmt.Println(time.Now().UnixMilli())
    time.Sleep(5 * time.Millisecond)
    fmt.Println(time.Now().UnixMilli())
}
"#,
            )],
            host_time_unix_millis: None,
            steps: vec![
                CompatStep::ClockNow { unix_millis: 100 },
                CompatStep::Sleep {
                    duration_millis: 5,
                    unix_millis: 105,
                },
                CompatStep::ClockNow { unix_millis: 105 },
            ],
            expected_stdout: "100\n105\n",
        },
        CompatCase {
            name: "net/http",
            files: vec![main_file(
                r#"
package main
import (
    "fmt"
    "net/http"
)

func main() {
    resp, err := http.Get("https://example.com/data")
    if err != nil {
        fmt.Println("fetch-failed", err)
        return
    }
    buf := make([]byte, 5)
    n, _ := resp.Body.Read(buf)
    closeErr := resp.Body.Close()
    fmt.Println(resp.StatusCode, string(buf[:n]), closeErr == nil)
}
"#,
            )],
            host_time_unix_millis: None,
            steps: vec![CompatStep::Fetch {
                method: "GET",
                url: "https://example.com/data",
                response_status_code: 200,
                response_status: "200 OK",
                response_url: "https://example.com/data",
                response_headers: vec![FetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["text/plain".into()],
                }],
                response_body: b"hello".to_vec(),
            }],
            expected_stdout: "200 hello true\n",
        },
        CompatCase {
            name: "io/fs",
            files: vec![
                main_file(
                    r#"
package main
import "fmt"
import "io/fs"
import "os"
import "strings"

func main() {
    root := os.DirFS("assets")
    data, err := fs.ReadFile(root, "config.txt")
    info, statErr := fs.Stat(root, "config.txt")
    entries, readDirErr := fs.ReadDir(root, ".")
    matches, globErr := fs.Glob(root, "*.txt")
    sub, subErr := fs.Sub(root, "nested")
    child, childErr := fs.ReadFile(sub, "child.txt")
    walked := make([]string, 0, 3)
    walkErr := fs.WalkDir(root, ".", func(path string, d fs.DirEntry, err error) error {
        if err != nil {
            return err
        }
        walked = append(walked, path)
        return nil
    })

    file, openErr := root.Open("config.txt")
    buf := make([]byte, 2)
    n, readErr := file.Read(buf)
    closeErr := file.Close()
    closedInfo, closedErr := file.Stat()

    fmt.Println(string(data), err == nil, string(child), subErr == nil && childErr == nil)
    fmt.Println(
        info.Size(),
        info.Mode().IsRegular(),
        statErr == nil,
        len(entries),
        entries[0].Name(),
        entries[1].Name(),
        readDirErr == nil,
    )
    fmt.Println(
        len(matches),
        matches[0],
        globErr == nil,
        strings.Join(walked, ","),
        walkErr == nil,
    )
    fmt.Println(
        openErr == nil,
        n,
        readErr == nil,
        closeErr == nil,
        closedInfo == nil,
        closedErr != nil,
    )
}
"#,
                ),
                WorkspaceFile {
                    path: "assets/config.txt".into(),
                    contents: "alpha".into(),
                },
                WorkspaceFile {
                    path: "assets/nested/child.txt".into(),
                    contents: "child".into(),
                },
            ],
            host_time_unix_millis: None,
            steps: Vec::new(),
            expected_stdout: concat!(
                "alpha true child true\n",
                "5 true true 2 config.txt nested true\n",
                "1 config.txt true .,config.txt,nested,nested/child.txt true\n",
                "true 2 true true true true\n",
            ),
        },
        CompatCase {
            name: "os",
            files: vec![main_file(
                r#"
package main
import (
    "fmt"
    "os"
)

func main() {
    mkdirErr := os.MkdirAll("/tmpdir/sub", 493)
    writeErr := os.WriteFile("/tmpdir/out.txt", []byte("beta"), 420)
    data, err := os.ReadFile("/tmpdir/out.txt")
    entries, readErr := os.ReadDir("/tmpdir")
    missingErr := os.WriteFile("/missing/out.txt", []byte("x"), 420)
    fmt.Println(string(data), err == nil && mkdirErr == nil && writeErr == nil)
    fmt.Println(len(entries), entries[0].Name(), entries[1].Name(), readErr == nil)
    fmt.Println(missingErr != nil)
}
"#,
            )],
            host_time_unix_millis: None,
            steps: Vec::new(),
            expected_stdout: "beta true\n2 out.txt sub true\ntrue\n",
        },
    ]
}

fn run_case(case: CompatCase) {
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
                "case `{}` expected a capability request, got {other:?}",
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
                "case `{}` produced unexpected stdout with diagnostics: {diagnostics:?}",
                case.name,
            );
            assert!(
                diagnostics.is_empty(),
                "case `{}` produced unexpected diagnostics: {diagnostics:?}",
                case.name
            );
        }
        other => panic!("case `{}` expected a run result, got {other:?}", case.name),
    }
}

fn assert_and_build_capability_result(
    case_name: &str,
    capability: CapabilityRequest,
    step: CompatStep,
) -> CapabilityResult {
    match step {
        CompatStep::ClockNow { unix_millis } => {
            assert_eq!(
                capability,
                CapabilityRequest::ClockNow,
                "case `{case_name}` expected clock_now capability"
            );
            CapabilityResult::ClockNow { unix_millis }
        }
        CompatStep::Sleep {
            duration_millis,
            unix_millis,
        } => {
            assert_eq!(
                capability,
                CapabilityRequest::Sleep { duration_millis },
                "case `{case_name}` expected sleep capability"
            );
            CapabilityResult::Sleep { unix_millis }
        }
        CompatStep::Fetch {
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
                "case `{case_name}` expected fetch capability"
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
