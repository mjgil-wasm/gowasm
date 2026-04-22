use std::fs;
use std::path::PathBuf;

use gowasm_host_types::{EngineRequest, EngineResponse, ErrorCategory, WorkspaceFile};
use serde::Deserialize;

use super::Engine;

#[derive(Debug, Deserialize)]
struct NegativeSupportCorpus {
    schema_version: u32,
    cases: Vec<NegativeSupportCase>,
}

#[derive(Debug, Deserialize)]
struct NegativeSupportCase {
    id: String,
    entry_path: String,
    files: Vec<NegativeSupportFile>,
    expected_message_substring: String,
    expected_category: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct NegativeSupportFile {
    path: String,
    contents: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_corpus() -> NegativeSupportCorpus {
    let path = repo_root().join("testdata/negative-support/index.json");
    let contents = fs::read_to_string(path).expect("negative support corpus should be readable");
    serde_json::from_str(&contents).expect("negative support corpus should parse")
}

#[test]
fn engine_run_reports_negative_support_corpus_as_compile_errors() {
    let corpus = load_corpus();
    assert_eq!(corpus.schema_version, 1);

    for case in corpus.cases {
        let files = case
            .files
            .iter()
            .map(|file| WorkspaceFile {
                path: file.path.clone(),
                contents: file.contents.clone(),
            })
            .collect::<Vec<_>>();
        let response = Engine::new().handle_request(EngineRequest::Run {
            files,
            entry_path: case.entry_path.clone(),
            host_time_unix_nanos: None,
            host_time_unix_millis: None,
        });
        match response {
            EngineResponse::RunResult {
                stdout,
                diagnostics,
            } => {
                assert!(
                    stdout.is_empty(),
                    "negative support run should not emit stdout for {}",
                    case.id
                );
                let diagnostic = diagnostics.first().unwrap_or_else(|| {
                    panic!(
                        "negative support run should emit diagnostics for {}",
                        case.id
                    )
                });
                assert_eq!(diagnostic.category, ErrorCategory::CompileError);
                assert_eq!(case.expected_category, "compile_error");
                assert!(
                    diagnostic
                        .message
                        .contains(case.expected_message_substring.as_str()),
                    "unexpected diagnostic for {}: {}",
                    case.id,
                    diagnostic.message
                );
                assert!(
                    case.tags.contains(&"runtime".to_string()),
                    "negative support runtime corpus case {} should carry the runtime tag",
                    case.id
                );
            }
            other => panic!("unexpected response for {}: {other:?}", case.id),
        }
    }
}
