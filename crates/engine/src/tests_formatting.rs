use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::{formatting::format_workspace_files, Engine};

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn format_request_reindents_go_files_and_preserves_other_workspace_files() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Format {
        files: vec![
            workspace_file(
                "main.go",
                "package main\n\nfunc main() {\nprintln(\"hi\")  // keep comment   \nswitch true {\ncase true:\nprintln(\"nested\")\n}\n}\n",
            ),
            workspace_file("notes.txt", "keep me\n"),
        ],
    });

    match response {
        EngineResponse::FormatResult { files, diagnostics } => {
            assert!(diagnostics.is_empty());
            assert_eq!(
                files,
                vec![
                    workspace_file(
                        "main.go",
                        "package main\n\nfunc main() {\n\tprintln(\"hi\")  // keep comment\n\tswitch true {\n\tcase true:\n\t\tprintln(\"nested\")\n\t}\n}\n",
                    ),
                    workspace_file("notes.txt", "keep me\n"),
                ],
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn format_request_reports_diagnostics_and_leaves_invalid_go_files_unchanged() {
    let mut engine = Engine::new();
    let files = vec![workspace_file("main.go", "package main\nfunc main( {\n")];
    let response = engine.handle_request(EngineRequest::Format {
        files: files.clone(),
    });

    match response {
        EngineResponse::FormatResult {
            files: formatted_files,
            diagnostics,
        } => {
            assert_eq!(formatted_files, files);
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].file_path.as_deref(), Some("main.go"));
            assert!(diagnostics[0].message.contains("cannot format `main.go`"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn workspace_formatter_indents_multiline_import_blocks() {
    let (files, diagnostics) = format_workspace_files(&[workspace_file(
        "main.go",
        "package main\nimport (\n\"fmt\"\n)\n",
    )]);

    assert!(diagnostics.is_empty());
    assert_eq!(
        files,
        vec![workspace_file(
            "main.go",
            "package main\nimport (\n\t\"fmt\"\n)\n"
        )],
    );
}

#[test]
fn workspace_formatter_preserves_multiline_raw_string_contents() {
    let (files, diagnostics) = format_workspace_files(&[workspace_file(
        "main.go",
        "package main\n\nfunc main() {\npayload := `{\n  \"items\": [\n    1,\n    2\n  ]\n}`\nprintln(payload)\n}\n",
    )]);

    assert!(diagnostics.is_empty());
    assert_eq!(
        files,
        vec![workspace_file(
            "main.go",
            "package main\n\nfunc main() {\n\tpayload := `{\n  \"items\": [\n    1,\n    2\n  ]\n}`\n\tprintln(payload)\n}\n"
        )],
    );
}

#[test]
fn workspace_formatter_ignores_line_comment_delimiters_for_indent_depth() {
    let (files, diagnostics) = format_workspace_files(&[workspace_file(
        "main.go",
        "package main\n\nfunc main() {\n// {\nprintln(\"hi\")\n}\n",
    )]);

    assert!(diagnostics.is_empty());
    assert_eq!(
        files,
        vec![workspace_file(
            "main.go",
            "package main\n\nfunc main() {\n\t// {\n\tprintln(\"hi\")\n}\n"
        )],
    );
}

#[test]
fn workspace_formatter_matches_golden_for_imports_generics_and_comments() {
    let source = "package main\n\nimport (\n\"fmt\"\n)\n\nfunc choose[T any](value T) T {\n// keep\n// comment\nreturn value\n}\n\nfunc main() {\nresult := choose[int](1)\nfmt.Println(result)\n}\n";
    let (files, diagnostics) = format_workspace_files(&[workspace_file("main.go", source)]);

    assert!(diagnostics.is_empty());
    assert_eq!(
        files,
        vec![workspace_file(
            "main.go",
            include_str!("testdata/formatting/imports_generics_and_comments.golden.go")
        )],
    );
}

#[test]
fn workspace_formatter_matches_golden_for_multiline_composites_and_calls() {
    let source = "package main\n\nfunc main() {\nvalue := map[string][]int{\n\"numbers\": []int{\n1,\n2,\n},\n}\nprintln(\nvalue[\"numbers\"][0],\n)\n}\n";
    let (files, diagnostics) = format_workspace_files(&[workspace_file("main.go", source)]);

    assert!(diagnostics.is_empty());
    assert_eq!(
        files,
        vec![workspace_file(
            "main.go",
            include_str!("testdata/formatting/multiline_composites_and_calls.golden.go")
        )],
    );
}
