use gowasm_host_types::{
    Diagnostic, EngineRequest, EngineResponse, ErrorCategory, Position, Severity, SourceExcerpt,
    SourceSpan, WorkspaceFile,
};
use serde::Serialize;

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[derive(Debug, Serialize)]
struct ResponseGolden {
    kind: &'static str,
    stdout: Option<String>,
    diagnostics: Vec<DiagnosticGolden>,
}

#[derive(Debug, Serialize)]
struct DiagnosticGolden {
    message_head: String,
    severity: &'static str,
    category: &'static str,
    file_path: Option<String>,
    position: Option<Position>,
    source_span: Option<SourceSpan>,
    source_excerpt: Option<SourceExcerpt>,
    suggested_action: Option<String>,
    runtime: Option<RuntimeGolden>,
}

#[derive(Debug, Serialize)]
struct RuntimeGolden {
    root_message: String,
    category: &'static str,
    stack_functions: Vec<String>,
}

fn compile_snapshot(source: &str) -> String {
    let response = Engine::new().handle_request(EngineRequest::Compile {
        files: vec![main_file(source)],
        entry_path: "main.go".into(),
    });
    response_snapshot(&response)
}

fn run_snapshot(source: &str) -> String {
    let response = Engine::new().handle_request(EngineRequest::Run {
        files: vec![main_file(source)],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    response_snapshot(&response)
}

fn budget_snapshot(source: &str, budget: u64) -> String {
    let response = Engine::with_instruction_budget(budget).handle_request(EngineRequest::Run {
        files: vec![main_file(source)],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    response_snapshot(&response)
}

fn response_snapshot(response: &EngineResponse) -> String {
    serde_json::to_string_pretty(&snapshot_response(response))
        .expect("golden diagnostics snapshot should serialize")
}

fn snapshot_response(response: &EngineResponse) -> ResponseGolden {
    match response {
        EngineResponse::Diagnostics { diagnostics } => ResponseGolden {
            kind: "diagnostics",
            stdout: None,
            diagnostics: diagnostics.iter().map(snapshot_diagnostic).collect(),
        },
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => ResponseGolden {
            kind: "run_result",
            stdout: Some(stdout.clone()),
            diagnostics: diagnostics.iter().map(snapshot_diagnostic).collect(),
        },
        other => panic!("unexpected response for diagnostics golden snapshot: {other:?}"),
    }
}

fn snapshot_diagnostic(diagnostic: &Diagnostic) -> DiagnosticGolden {
    DiagnosticGolden {
        message_head: diagnostic
            .message
            .lines()
            .next()
            .unwrap_or_default()
            .to_string(),
        severity: snapshot_severity(&diagnostic.severity),
        category: snapshot_category(diagnostic.category),
        file_path: diagnostic.file_path.clone(),
        position: diagnostic.position.clone(),
        source_span: diagnostic.source_span.clone(),
        source_excerpt: diagnostic.source_excerpt.clone(),
        suggested_action: diagnostic.suggested_action.clone(),
        runtime: diagnostic.runtime.as_ref().map(|runtime| RuntimeGolden {
            root_message: runtime.root_message.clone(),
            category: snapshot_category(runtime.category),
            stack_functions: runtime
                .stack_trace
                .iter()
                .map(|frame| frame.function.clone())
                .collect(),
        }),
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
fn compiler_non_function_call_snapshot_matches_golden() {
    let actual = compile_snapshot(
        r#"package main

func main() {
    value := 1
    value()
}
"#,
    );
    let expected = include_str!("testdata/diagnostics/compiler_non_function_call.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn runtime_division_by_zero_snapshot_matches_golden() {
    let actual = run_snapshot(
        r#"package main

func explode() {
    value := 0
    _ = 1 / value
}

func main() {
    explode()
}
"#,
    );
    let expected = include_str!("testdata/diagnostics/runtime_division_by_zero.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn panic_stack_snapshot_matches_golden() {
    let actual = run_snapshot(
        r#"package main

func explode() {
    panic("boom")
}

func main() {
    explode()
}
"#,
    );
    let expected = include_str!("testdata/diagnostics/panic_unhandled.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn panic_replacement_snapshot_matches_golden() {
    let actual = run_snapshot(
        r#"package main

func cleanup() {
    panic("replacement")
}

func main() {
    defer cleanup()
    panic("initial")
}
"#,
    );
    let expected = include_str!("testdata/diagnostics/panic_replacement.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn goroutine_panic_snapshot_matches_golden() {
    let actual = run_snapshot(
        r#"package main

func worker() {
    panic("boom")
}

func main() {
    done := make(chan bool)
    go worker()
    <-done
}
"#,
    );
    let expected = include_str!("testdata/diagnostics/panic_goroutine.golden.json");
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn budget_exhaustion_snapshot_matches_golden() {
    let actual = budget_snapshot(
        r#"package main

func main() {
    for {
    }
}
"#,
        50,
    );
    let expected = include_str!("testdata/diagnostics/runtime_budget_exhaustion.golden.json");
    assert_eq!(actual, expected.trim_end());
}
