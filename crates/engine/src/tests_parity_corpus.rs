use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, FetchHeader, FetchRequest,
    FetchResponse, FetchResult, WorkspaceFile,
};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use super::Engine;

#[derive(Debug, Deserialize)]
struct CorpusIndex {
    schema_version: u32,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    name: String,
    entry_path: String,
    host_time_unix_millis: Option<i64>,
    workspace_files: Vec<String>,
    expected_stdout: String,
    steps: Vec<CorpusStep>,
    expected_outcomes: ExpectedOutcomes,
}

#[derive(Debug, Deserialize)]
struct ExpectedOutcomes {
    engine: ExpectedOutcome,
}

#[derive(Debug, Deserialize)]
struct ExpectedOutcome {
    status: OutcomeStatus,
    failure_task: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OutcomeStatus {
    Pass,
    Fail,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CorpusStep {
    Fetch {
        method: String,
        url: String,
        response_status_code: i64,
        response_status: String,
        response_url: String,
        response_headers: Vec<CorpusHeader>,
        response_body: String,
    },
}

#[derive(Debug, Deserialize)]
struct CorpusHeader {
    name: String,
    values: Vec<String>,
}

#[test]
fn checked_in_parity_corpus_runs_through_engine() {
    let index = load_corpus_index();
    assert_eq!(index.schema_version, 2, "unexpected parity corpus schema");
    assert!(
        !index.cases.is_empty(),
        "parity corpus should contain representative cases"
    );

    for case in index.cases {
        run_case(case);
    }
}

fn load_corpus_index() -> CorpusIndex {
    serde_json::from_str(
        &fs::read_to_string(corpus_root().join("index.json"))
            .expect("parity corpus index should be readable"),
    )
    .expect("parity corpus index should deserialize")
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/parity-corpus")
}

fn run_case(case: CorpusCase) {
    let CorpusCase {
        id,
        name,
        entry_path,
        host_time_unix_millis,
        workspace_files,
        expected_stdout,
        steps,
        expected_outcomes,
    } = case;

    assert!(
        expected_outcomes.engine.status == OutcomeStatus::Pass,
        "engine parity case `{}` is tracked as failing via {:?}",
        name,
        expected_outcomes.engine.failure_task
    );

    let mut engine = Engine::new();
    let mut response = engine.handle_request(EngineRequest::Run {
        files: load_workspace_files(&id, &workspace_files),
        entry_path,
        host_time_unix_nanos: None,
        host_time_unix_millis,
    });

    for step in steps {
        let (run_id, capability) = match response {
            EngineResponse::CapabilityRequest { run_id, capability } => (run_id, capability),
            other => panic!(
                "corpus case `{}` expected a capability request, got {other:?}",
                name
            ),
        };
        response = engine.handle_request(EngineRequest::Resume {
            run_id,
            capability: assert_and_build_capability_result(&name, capability, step),
        });
    }

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout, expected_stdout,
                "corpus case `{}` produced unexpected stdout",
                name
            );
            assert!(
                diagnostics.is_empty(),
                "corpus case `{}` produced unexpected diagnostics: {diagnostics:?}",
                name
            );
        }
        other => panic!(
            "corpus case `{}` expected a run result, got {other:?}",
            name
        ),
    }
}

fn load_workspace_files(case_id: &str, workspace_files: &[String]) -> Vec<WorkspaceFile> {
    let workspace_root = corpus_root().join(case_id).join("workspace");
    workspace_files
        .iter()
        .map(|path| WorkspaceFile {
            path: path.clone(),
            contents: fs::read_to_string(workspace_root.join(path))
                .expect("workspace file should be readable"),
        })
        .collect()
}

fn assert_and_build_capability_result(
    case_name: &str,
    capability: CapabilityRequest,
    step: CorpusStep,
) -> CapabilityResult {
    match step {
        CorpusStep::Fetch {
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
                        method: method.clone(),
                        url: url.clone(),
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: None,
                    },
                },
                "corpus case `{case_name}` expected fetch capability"
            );
            CapabilityResult::Fetch {
                result: FetchResult::Response {
                    response: FetchResponse {
                        status_code: response_status_code,
                        status: response_status,
                        url: response_url,
                        headers: response_headers
                            .into_iter()
                            .map(|header| FetchHeader {
                                name: header.name,
                                values: header.values,
                            })
                            .collect(),
                        body: response_body.into_bytes(),
                    },
                },
            }
        }
    }
}
