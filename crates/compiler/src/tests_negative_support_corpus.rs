use std::fs;
use std::path::PathBuf;

use crate::{compile_workspace, CompileError, SourceInput};
use serde::Deserialize;

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
fn negative_support_corpus_rejects_documented_non_goals() {
    let corpus = load_corpus();
    assert_eq!(corpus.schema_version, 1);

    for case in corpus.cases {
        let owned_sources = case
            .files
            .iter()
            .map(|file| SourceInput {
                path: file.path.as_str(),
                source: file.contents.as_str(),
            })
            .collect::<Vec<_>>();
        let error = compile_workspace(&owned_sources, &case.entry_path)
            .expect_err("negative support cases should fail to compile");
        assert!(
            error
                .to_string()
                .contains(case.expected_message_substring.as_str()),
            "unexpected diagnostic for {}: {error}",
            case.id
        );
        assert_eq!(
            case.expected_category, "compile_error",
            "negative support corpus currently only tracks compile errors"
        );
    }
}

#[test]
fn negative_support_corpus_covers_required_exclusion_tags() {
    let corpus = load_corpus();
    let tags = corpus
        .cases
        .iter()
        .flat_map(|case| case.tags.iter().map(String::as_str))
        .collect::<std::collections::HashSet<_>>();

    for required in [
        "cgo",
        "plugin",
        "os_exec",
        "arbitrary_fs",
        "unsafe",
        "reflect_mutation",
    ] {
        assert!(
            tags.contains(required),
            "negative support corpus should cover `{required}`"
        );
    }
}

#[test]
fn negative_support_corpus_promotes_documented_non_goal_imports_to_unsupported_diagnostics() {
    let cases = [
        (
            "import \"C\"",
            "unsupported syntax: cgo via `import \"C\"` is outside the supported subset",
        ),
        (
            "import \"plugin\"",
            "unsupported syntax: package `plugin` is outside the supported subset",
        ),
        (
            "import \"os/exec\"",
            "unsupported syntax: package `os/exec` is outside the supported subset",
        ),
        (
            "import \"unsafe\"",
            "unsupported syntax: package `unsafe` is outside the supported subset",
        ),
    ];

    for (import_decl, expected) in cases {
        let source = format!("package main\n\n{import_decl}\n\nfunc main() {{}}\n");
        let error = compile_workspace(
            &[SourceInput {
                path: "main.go",
                source: source.as_str(),
            }],
            "main.go",
        )
        .expect_err("unsupported imports should fail");
        match error {
            CompileError::Unsupported { detail } => {
                let full = format!("unsupported syntax: {detail}");
                assert_eq!(full, expected);
            }
            other => panic!("expected unsupported diagnostic, got {other:?}"),
        }
    }
}
