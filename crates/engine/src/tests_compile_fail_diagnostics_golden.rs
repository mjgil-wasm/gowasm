use gowasm_compiler::module_cache_source_path;
use gowasm_host_types::{
    Diagnostic, EngineRequest, EngineResponse, ErrorCategory, Position, Severity, SourceExcerpt,
    SourceSpan, WorkspaceFile,
};
use serde::Serialize;

use super::Engine;

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[derive(Debug, Serialize)]
struct DiagnosticGolden {
    message: String,
    severity: &'static str,
    category: &'static str,
    file_path: Option<String>,
    position: Option<Position>,
    source_span: Option<SourceSpan>,
    source_excerpt: Option<SourceExcerpt>,
    suggested_action: Option<String>,
}

fn compile_snapshot(files: Vec<WorkspaceFile>, entry_path: &str) -> String {
    let response = Engine::new().handle_request(EngineRequest::Compile {
        files,
        entry_path: entry_path.into(),
    });
    let diagnostic = compile_diagnostic(&response);
    serde_json::to_string_pretty(&snapshot_diagnostic(diagnostic))
        .expect("compile diagnostics golden snapshot should serialize")
}

fn compile_diagnostic(response: &EngineResponse) -> &Diagnostic {
    match response {
        EngineResponse::Diagnostics { diagnostics } => diagnostics
            .first()
            .expect("compile-fail snapshot should produce one diagnostic"),
        other => panic!("unexpected response for compile diagnostics golden snapshot: {other:?}"),
    }
}

fn snapshot_diagnostic(diagnostic: &Diagnostic) -> DiagnosticGolden {
    DiagnosticGolden {
        message: diagnostic.message.clone(),
        severity: snapshot_severity(&diagnostic.severity),
        category: snapshot_category(diagnostic.category),
        file_path: diagnostic.file_path.clone(),
        position: diagnostic.position.clone(),
        source_span: diagnostic.source_span.clone(),
        source_excerpt: diagnostic.source_excerpt.clone(),
        suggested_action: diagnostic.suggested_action.clone(),
    }
}

fn snapshot_severity(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn snapshot_category(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::Uncategorized => "uncategorized",
        ErrorCategory::CompileError => "compile_error",
        ErrorCategory::Tooling => "tooling",
        ErrorCategory::ProtocolError => "protocol_error",
        ErrorCategory::HostError => "host_error",
        ErrorCategory::RuntimePanic => "runtime_panic",
        ErrorCategory::RuntimeTrap => "runtime_trap",
        ErrorCategory::RuntimeBudgetExhaustion => "runtime_budget_exhaustion",
        ErrorCategory::RuntimeDeadlock => "runtime_deadlock",
        ErrorCategory::RuntimeCancellation => "runtime_cancellation",
        ErrorCategory::RuntimeExit => "runtime_exit",
    }
}

#[test]
fn compile_fail_assignment_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() {
    label := "go"
    var count int
    count = label
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/assignment_type_mismatch.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_call_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() {
    value := 1
    value()
}
"#,
        )],
        "main.go",
    );
    let expected = include_str!("testdata/compile_fail_diagnostics/non_callable_call.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_return_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() int {
    label := "go"
    return label
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/return_type_mismatch.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_map_key_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() {
    var values map[[]int]int
    _ = values
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/map_key_not_comparable.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_channel_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() {
    var recv <-chan int
    recv <- 1
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/channel_direction_send.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_interface_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

type Shape interface {
    Area() int
}

func main() {
    value := 1
    var shape Shape = value
    _ = shape
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/interface_assignment.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_generic_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func use[T comparable](value T) {}

func main() {
    use[[]int]([]int{1})
}
"#,
        )],
        "main.go",
    );
    let expected = include_str!("testdata/compile_fail_diagnostics/generic_constraint.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_import_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

import "example.com/missing"

func main() {}
"#,
        )],
        "main.go",
    );
    let expected = include_str!("testdata/compile_fail_diagnostics/unresolved_import.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_import_cycle_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"package main

import "example.com/app/lib"

func main() {}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"package lib

import "example.com/app"
"#,
            ),
        ],
        "main.go",
    );
    let expected = include_str!("testdata/compile_fail_diagnostics/import_cycle.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_missing_module_manifest_diagnostic_matches_golden() {
    let remote_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");
    let actual = compile_snapshot(
        vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"package main

import "example.com/remote/greeter"

func main() {}
"#,
            ),
            workspace_file(
                remote_go_file_path.as_str(),
                r#"package greeter

func Message() string { return "hi" }
"#,
            ),
        ],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/missing_module_manifest.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_mismatched_module_root_diagnostic_matches_golden() {
    let remote_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let remote_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");
    let actual = compile_snapshot(
        vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"package main

import "example.com/remote/greeter"

func main() {}
"#,
            ),
            workspace_file(
                remote_go_mod_path.as_str(),
                "module example.com/not-remote\n\ngo 1.21\n",
            ),
            workspace_file(
                remote_go_file_path.as_str(),
                r#"package greeter

func Message() string { return "hi" }
"#,
            ),
        ],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/mismatched_module_root.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_conflicting_module_versions_diagnostic_matches_golden() {
    let first_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let first_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "first/first.go");
    let second_go_mod_path = module_cache_source_path("example.com/remote", "v1.3.0", "go.mod");
    let second_go_file_path =
        module_cache_source_path("example.com/remote", "v1.3.0", "second/second.go");
    let actual = compile_snapshot(
        vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"package main

import "example.com/remote/first"

func main() {}
"#,
            ),
            workspace_file(
                first_go_mod_path.as_str(),
                "module example.com/remote\n\ngo 1.21\n",
            ),
            workspace_file(
                first_go_file_path.as_str(),
                r#"package first

import "example.com/remote/second"
"#,
            ),
            workspace_file(
                second_go_mod_path.as_str(),
                "module example.com/remote\n\ngo 1.21\n",
            ),
            workspace_file(second_go_file_path.as_str(), "package second\n"),
        ],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/conflicting_module_versions.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_unsupported_module_feature_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![
            workspace_file(
                "go.mod",
                r#"module example.com/app

go 1.21

require example.com/remote v1.2.3
"#,
            ),
            workspace_file("main.go", "package main\n\nfunc main() {}\n"),
        ],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/unsupported_module_feature.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn compile_fail_const_diagnostic_matches_golden() {
    let actual = compile_snapshot(
        vec![workspace_file(
            "main.go",
            r#"package main

func main() {
    var b byte = 300
    _ = b
}
"#,
        )],
        "main.go",
    );
    let expected =
        include_str!("testdata/compile_fail_diagnostics/const_representability.golden.json");
    assert_eq!(actual, expected.trim_end());
}
