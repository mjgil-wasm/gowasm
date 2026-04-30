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
fn app_style_sample_suite_two_runs_end_to_end() {
    for case in sample_cases() {
        run_case(case);
    }
}

fn sample_cases() -> Vec<SampleCase> {
    vec![
        SampleCase {
            name: "concurrent feed poller",
            files: vec![
                WorkspaceFile {
                    path: "main.go".into(),
                    contents: r#"
package main

import "fmt"

func main() {
    report, err := runFeedPoller()
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
                    path: "poller.go".into(),
                    contents: r#"
package main

import (
    "encoding/json"
    "net/http"
    "sort"
    "sync"
)

type FeedResult struct {
    Name string
    Status string
    Body string
}

type FeedReport struct {
    Count int
    Results []FeedResult
}

func runFeedPoller() (string, error) {
    results := make([]FeedResult, 0, 2)
    firstDone := make(chan bool, 1)
    errCh := make(chan error, 2)
    var mu sync.Mutex
    var wg sync.WaitGroup

    wg.Add(2)
    go func() {
        defer wg.Done()
        defer func() {
            firstDone <- true
        }()
        resp, err := http.Get("https://example.com/feed-alpha")
        if err != nil {
            errCh <- err
            return
        }
        defer resp.Body.Close()

        buf := make([]byte, 32)
        n, _ := resp.Body.Read(buf)

        mu.Lock()
        results = append(results, FeedResult{
            Name: "alpha",
            Status: resp.Status,
            Body: string(buf[:n]),
        })
        mu.Unlock()
    }()

    go func() {
        defer wg.Done()
        <-firstDone

        resp, err := http.Get("https://example.com/feed-beta")
        if err != nil {
            errCh <- err
            return
        }
        defer resp.Body.Close()

        buf := make([]byte, 32)
        n, _ := resp.Body.Read(buf)

        mu.Lock()
        results = append(results, FeedResult{
            Name: "beta",
            Status: resp.Status,
            Body: string(buf[:n]),
        })
        mu.Unlock()
    }()

    wg.Wait()
    select {
    case err := <-errCh:
        return "", err
    default:
    }
    sort.Slice(results, func(i int, j int) bool {
        return results[i].Name < results[j].Name
    })

    payload, err := json.MarshalIndent(FeedReport{
        Count: len(results),
        Results: results,
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
            steps: vec![
                SampleStep::Fetch {
                    method: "GET",
                    url: "https://example.com/feed-alpha",
                    response_status_code: 200,
                    response_status: "200 OK",
                    response_url: "https://example.com/feed-alpha",
                    response_headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["text/plain".into()],
                    }],
                    response_body: b"feed-alpha".to_vec(),
                },
                SampleStep::Fetch {
                    method: "GET",
                    url: "https://example.com/feed-beta",
                    response_status_code: 200,
                    response_status: "200 OK",
                    response_url: "https://example.com/feed-beta",
                    response_headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["text/plain".into()],
                    }],
                    response_body: b"feed-beta".to_vec(),
                },
            ],
            expected_stdout: "{\n  \"Count\": 2,\n  \"Results\": [\n    {\n      \"Name\": \"alpha\",\n      \"Status\": \"200 OK\",\n      \"Body\": \"feed-alpha\"\n    },\n    {\n      \"Name\": \"beta\",\n      \"Status\": \"200 OK\",\n      \"Body\": \"feed-beta\"\n    }\n  ]\n}\n",
        },
        SampleCase {
            name: "concurrent workspace cache",
            files: vec![
                WorkspaceFile {
                    path: "main.go".into(),
                    contents: r#"
package main

import "fmt"

func main() {
    report, err := buildWorkspaceCache()
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
                    path: "cache.go".into(),
                    contents: r#"
package main

import (
    "encoding/json"
    fs "io/fs"
    "os"
    "sort"
    "sync"
)

type CacheFile struct {
    Name string
    Body string
}

type CacheReport struct {
    Count int
    Files []CacheFile
}

func buildWorkspaceCache() (string, error) {
    dirfs := os.DirFS("/notes")
    ready := make(chan string, 2)
    errCh := make(chan error, 2)
    cache := map[string]string{}
    names := []string{"alpha.txt", "beta.txt"}
    var mu sync.RWMutex
    var wg sync.WaitGroup

    for _, name := range names {
        name := name
        wg.Add(1)
        go func() {
            defer wg.Done()

            body, err := fs.ReadFile(dirfs, name)
            if err != nil {
                errCh <- err
                return
            }

            mu.Lock()
            cache[name] = string(body)
            mu.Unlock()
            ready <- name
        }()
    }

    wg.Wait()
    select {
    case err := <-errCh:
        return "", err
    default:
    }
    seen := []string{<-ready, <-ready}
    sort.Strings(seen)

    mu.RLock()
    files := make([]CacheFile, 0, len(seen))
    for _, name := range seen {
        files = append(files, CacheFile{
            Name: name,
            Body: cache[name],
        })
    }
    mu.RUnlock()

    payload, err := json.MarshalIndent(CacheReport{
        Count: len(files),
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
                    path: "notes/alpha.txt".into(),
                    contents: "first".into(),
                },
                WorkspaceFile {
                    path: "notes/beta.txt".into(),
                    contents: "second".into(),
                },
            ],
            steps: Vec::new(),
            expected_stdout: "{\n  \"Count\": 2,\n  \"Files\": [\n    {\n      \"Name\": \"alpha.txt\",\n      \"Body\": \"first\"\n    },\n    {\n      \"Name\": \"beta.txt\",\n      \"Body\": \"second\"\n    }\n  ]\n}\n",
        },
    ]
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
