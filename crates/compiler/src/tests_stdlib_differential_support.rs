use gowasm_vm::Vm;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{compile_workspace, SourceInput};

#[derive(Debug, Deserialize)]
pub(super) struct DifferentialCorpusIndex {
    pub(super) schema_version: u32,
    pub(super) cases: Vec<DifferentialCase>,
}

#[derive(Debug, Deserialize)]
pub(super) struct DifferentialCase {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) entry_path: String,
    pub(super) workspace_files: Vec<String>,
    pub(super) expected_native_go_stdout: String,
}

pub(super) struct WorkspaceFileFixture {
    pub(super) path: String,
    pub(super) contents: String,
}

pub(super) fn load_corpus_index(relative_root: &str) -> DifferentialCorpusIndex {
    serde_json::from_str(
        &fs::read_to_string(corpus_root(relative_root).join("index.json"))
            .expect("stdlib differential corpus index should be readable"),
    )
    .expect("stdlib differential corpus index should deserialize")
}

pub(super) fn corpus_root(relative_root: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../{relative_root}"))
}

pub(super) fn load_workspace_files(
    corpus_root: &Path,
    case_id: &str,
    workspace_files: &[String],
) -> Vec<WorkspaceFileFixture> {
    let workspace_root = corpus_root.join(case_id).join("workspace");
    let mut files = Vec::with_capacity(workspace_files.len());
    for path in workspace_files {
        let full_path = workspace_root.join(path);
        files.push(WorkspaceFileFixture {
            path: path.clone(),
            contents: fs::read_to_string(&full_path).unwrap_or_else(|_| {
                panic!(
                    "stdlib differential workspace file `{}` should be readable",
                    full_path.display()
                )
            }),
        });
    }
    files
}

pub(super) fn run_stdout_differential_case(relative_root: &str, case: DifferentialCase) {
    let root = corpus_root(relative_root);
    let workspace_files = load_workspace_files(&root, &case.id, &case.workspace_files);
    let source_inputs = workspace_files
        .iter()
        .filter(|file| file.path.ends_with(".go"))
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect::<Vec<_>>();

    let program = compile_workspace(&source_inputs, &case.entry_path).unwrap_or_else(|error| {
        panic!(
            "stdlib differential case `{}` should compile: {error}",
            case.name
        )
    });

    let mut vm = Vm::new();
    for file in &workspace_files {
        if file.path.ends_with(".go") {
            continue;
        }
        vm.workspace_files
            .insert(file.path.clone(), file.contents.clone());
    }
    vm.run_program(&program)
        .unwrap_or_else(|_| panic!("stdlib differential case `{}` should run", case.name));
    assert_eq!(
        vm.stdout(),
        case.expected_native_go_stdout,
        "stdlib differential case `{}` diverged from the checked-in native Go output",
        case.name
    );
}
