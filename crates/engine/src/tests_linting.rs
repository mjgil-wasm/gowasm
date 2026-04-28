use gowasm_host_types::{EngineRequest, EngineResponse, Severity, WorkspaceFile};

use super::Engine;

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn lint_request_warns_when_go_source_needs_formatting() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file(
            "main.go",
            "package main\n\nimport \"fmt\"\n\nfunc main() {\nprintln(\"hi\")\n}\n",
        )],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert_eq!(diagnostics.len(), 2);
            assert!(diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity == Severity::Warning));
            assert!(diagnostics
                .iter()
                .all(|diagnostic| diagnostic.file_path.as_deref() == Some("main.go")));
            assert!(diagnostics
                .iter()
                .all(|diagnostic| diagnostic.position.is_some()));
            assert!(diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("run Format")));
            assert!(diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("never references `fmt`")));
            let formatting = diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message.contains("run Format"))
                .expect("formatting warning should exist");
            assert_eq!(
                formatting.position.as_ref().map(|position| position.line),
                Some(6)
            );
            assert_eq!(
                formatting.position.as_ref().map(|position| position.column),
                Some(1)
            );
            let unused_import = diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message.contains("never references `fmt`"))
                .expect("unused-import warning should exist");
            assert_eq!(
                unused_import
                    .position
                    .as_ref()
                    .map(|position| position.line),
                Some(3)
            );
            assert_eq!(
                unused_import
                    .position
                    .as_ref()
                    .map(|position| position.column),
                Some(8)
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn lint_request_reuses_parse_diagnostics_for_invalid_go_source() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file("main.go", "package main\nfunc main( {\n")],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].severity, Severity::Error);
            assert_eq!(diagnostics[0].file_path.as_deref(), Some("main.go"));
            assert!(diagnostics[0].message.contains("cannot lint `main.go`"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn lint_request_warns_when_go_source_imports_the_same_path_twice() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file(
            "main.go",
            "package main\n\nimport (\n\t\"fmt\"\n\t\"fmt\"\n)\n\nfunc main() {\n\tfmt.Println(\"hi\")\n}\n",
        )],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].severity, Severity::Warning);
            assert_eq!(diagnostics[0].file_path.as_deref(), Some("main.go"));
            assert_eq!(
                diagnostics[0]
                    .position
                    .as_ref()
                    .map(|position| position.line),
                Some(5)
            );
            assert_eq!(
                diagnostics[0]
                    .position
                    .as_ref()
                    .map(|position| position.column),
                Some(2)
            );
            assert!(diagnostics[0]
                .message
                .contains("imports `fmt` more than once"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn lint_request_supports_file_scoped_rule_suppressions() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file(
            "main.go",
            "package main\n\n//gowasm:ignore format-drift\n//gowasm:ignore duplicate-import\n//gowasm:ignore unused-import\nimport (\n\"fmt\"\n\"fmt\"\n)\n\nfunc main() {\nprintln(\"hi\")\n}\n",
        )],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn lint_request_only_suppresses_the_named_rule() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file(
            "main.go",
            "package main\n\n//gowasm:ignore unused-import\nimport \"fmt\"\n\nfunc main() {\nprintln(\"hi\")\n}\n",
        )],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0].message.contains("run Format"));
            assert_eq!(
                diagnostics[0]
                    .position
                    .as_ref()
                    .map(|position| position.line),
                Some(7)
            );
            assert_eq!(
                diagnostics[0]
                    .position
                    .as_ref()
                    .map(|position| position.column),
                Some(1)
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn lint_request_does_not_flag_imports_used_in_type_positions() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Lint {
        files: vec![workspace_file(
            "main.go",
            "package main\n\nimport \"example.com/remote/cards\"\n\ntype Report struct {\n\tCard cards.Card\n}\n\nfunc main() {\n\tvar report Report\n\tprintln(report.Card.Name)\n}\n",
        )],
    });

    match response {
        EngineResponse::LintResult { diagnostics } => {
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
