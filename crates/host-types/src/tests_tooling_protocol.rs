use super::{
    Diagnostic, EngineRequest, EngineResponse, TestResultDetails, TestRunnerKind, WorkspaceFile,
};

#[test]
fn format_request_and_response_round_trip_through_json() {
    let request = EngineRequest::Format {
        files: vec![
            WorkspaceFile {
                path: "main.go".into(),
                contents: "package main\n".into(),
            },
            WorkspaceFile {
                path: "notes.txt".into(),
                contents: "keep me\n".into(),
            },
        ],
    };
    let request_json = serde_json::to_string(&request).expect("request should serialize");
    let decoded_request: EngineRequest =
        serde_json::from_str(&request_json).expect("request should parse");
    assert_eq!(decoded_request, request);

    let response = EngineResponse::FormatResult {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: "package main\n".into(),
        }],
        diagnostics: vec![Diagnostic::error("cannot format invalid source")],
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}

#[test]
fn lint_request_and_response_round_trip_through_json() {
    let request = EngineRequest::Lint {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: "package main\n".into(),
        }],
    };
    let request_json = serde_json::to_string(&request).expect("request should serialize");
    let decoded_request: EngineRequest =
        serde_json::from_str(&request_json).expect("request should parse");
    assert_eq!(decoded_request, request);

    let response = EngineResponse::LintResult {
        diagnostics: vec![Diagnostic::error("lint failed")],
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}

#[test]
fn test_requests_and_response_round_trip_through_json() {
    let package_request = EngineRequest::TestPackage {
        files: vec![WorkspaceFile {
            path: "calc.go".into(),
            contents: "package calc\n".into(),
        }],
        target_path: "calc.go".into(),
        filter: Some("TestAdd".into()),
    };
    let package_request_json =
        serde_json::to_string(&package_request).expect("request should serialize");
    let decoded_package_request: EngineRequest =
        serde_json::from_str(&package_request_json).expect("request should parse");
    assert_eq!(decoded_package_request, package_request);

    let snippet_request = EngineRequest::TestSnippet {
        files: vec![WorkspaceFile {
            path: "main.go".into(),
            contents: "package main\nfunc main() {}\n".into(),
        }],
        entry_path: "main.go".into(),
    };
    let snippet_request_json =
        serde_json::to_string(&snippet_request).expect("request should serialize");
    let decoded_snippet_request: EngineRequest =
        serde_json::from_str(&snippet_request_json).expect("request should parse");
    assert_eq!(decoded_snippet_request, snippet_request);

    let response = EngineResponse::TestResult {
        runner: TestRunnerKind::Package,
        passed: true,
        stdout: "PASS TestAdd\nPASS\n".into(),
        diagnostics: Vec::new(),
        details: TestResultDetails {
            subject_path: "calc.go".into(),
            planned_tests: vec!["TestAdd".into()],
            completed_tests: vec!["TestAdd".into()],
            active_test: None,
        },
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}
