use gowasm_vm::Vm;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::{compile_workspace, SourceInput};

#[derive(Debug, Deserialize)]
struct DifferentialCorpusIndex {
    schema_version: u32,
    cases: Vec<DifferentialCase>,
}

#[derive(Debug, Deserialize)]
struct DifferentialCase {
    id: String,
    name: String,
    entry_path: String,
    workspace_files: Vec<String>,
    expected_native_go_stdout: String,
}

struct WorkspaceFileFixture {
    path: String,
    contents: String,
}

#[test]
fn regexp_differential_corpus_matches_checked_in_native_go_outputs() {
    let index = load_corpus_index();
    assert_eq!(
        index.schema_version, 1,
        "unexpected regexp differential schema"
    );
    assert!(
        !index.cases.is_empty(),
        "regexp differential corpus should contain representative cases"
    );

    for case in index.cases {
        run_case(case);
    }
}

fn load_corpus_index() -> DifferentialCorpusIndex {
    serde_json::from_str(
        &fs::read_to_string(corpus_root().join("index.json"))
            .expect("regexp differential corpus index should be readable"),
    )
    .expect("regexp differential corpus index should deserialize")
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/regexp-differential")
}

fn run_case(case: DifferentialCase) {
    let workspace_files = load_workspace_files(&case.id, &case.workspace_files);
    let source_inputs = workspace_files
        .iter()
        .filter(|file| file.path.ends_with(".go"))
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect::<Vec<_>>();

    let program = compile_workspace(&source_inputs, &case.entry_path)
        .unwrap_or_else(|_| panic!("regexp differential case `{}` should compile", case.name));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .unwrap_or_else(|_| panic!("regexp differential case `{}` should run", case.name));
    assert_eq!(
        vm.stdout(),
        case.expected_native_go_stdout,
        "regexp differential case `{}` diverged from the checked-in native Go output",
        case.name
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
                    "regexp differential workspace file `{}` should be readable",
                    full_path.display()
                )
            }),
        });
    }
    files
}
