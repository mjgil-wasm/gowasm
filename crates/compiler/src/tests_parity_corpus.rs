use gowasm_vm::{CapabilityRequest, FetchHeader, FetchRequest, FetchResponse, RunOutcome, Vm};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::{compile_workspace, SourceInput};

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
    compiler_vm: ExpectedOutcome,
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

struct WorkspaceFileFixture {
    path: String,
    contents: String,
}

#[test]
fn parity_corpus_cases_pass_through_direct_compiler_and_vm_execution() {
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

    assert_eq!(
        expected_outcomes.compiler_vm.status,
        OutcomeStatus::Pass,
        "compiler/vm parity case `{}` is tracked as failing via {:?}",
        name,
        expected_outcomes.compiler_vm.failure_task
    );

    let workspace_files = load_workspace_files(&id, &workspace_files);
    let source_inputs = workspace_files
        .iter()
        .filter(|file| file.path.ends_with(".go") || file.path.ends_with("go.mod"))
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect::<Vec<_>>();

    let program =
        compile_workspace(&source_inputs, &entry_path).expect("parity corpus should compile");

    let mut vm = Vm::new();
    if let Some(unix_millis) = host_time_unix_millis {
        vm.set_time_now_override_unix_nanos(unix_millis * 1_000_000);
    }
    for file in &workspace_files {
        vm.workspace_files
            .insert(file.path.clone(), file.contents.clone());
    }
    vm.enable_capability_requests();

    let mut outcome = vm
        .start_program(&program)
        .expect("parity corpus program should start");

    for step in steps {
        match (outcome, step) {
            (
                RunOutcome::CapabilityRequest(CapabilityRequest::Fetch { request }),
                CorpusStep::Fetch {
                    method,
                    url,
                    response_status_code,
                    response_status,
                    response_url,
                    response_headers,
                    response_body,
                },
            ) => {
                assert_eq!(
                    request,
                    FetchRequest {
                        method,
                        url,
                        headers: Vec::new(),
                        body: Vec::new(),
                        context_deadline_unix_millis: None,
                    },
                    "compiler/vm parity case `{}` issued an unexpected fetch request",
                    name
                );
                vm.set_fetch_response(FetchResponse {
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
                });
                outcome = vm
                    .resume_program(&program)
                    .expect("parity corpus program should resume after fetch");
            }
            (other, step) => panic!(
                "compiler/vm parity case `{}` saw mismatched outcome {other:?} for step {step:?}",
                name
            ),
        }
    }

    match outcome {
        RunOutcome::Completed => {}
        other => panic!(
            "compiler/vm parity case `{}` expected completion, got {other:?}",
            name
        ),
    }

    assert_eq!(
        vm.stdout(),
        expected_stdout,
        "compiler/vm parity case `{}` produced unexpected stdout",
        name
    );
}

fn load_workspace_files(case_id: &str, workspace_files: &[String]) -> Vec<WorkspaceFileFixture> {
    let workspace_root = corpus_root().join(case_id).join("workspace");
    let mut files = Vec::with_capacity(workspace_files.len());
    for path in workspace_files {
        let full_path = workspace_root.join(path);
        files.push(WorkspaceFileFixture {
            path: path.clone(),
            contents: fs::read_to_string(&full_path).unwrap_or_else(|_| {
                panic!(
                    "workspace file `{}` should be readable",
                    full_path.display()
                )
            }),
        });
    }
    files
}
