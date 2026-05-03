use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, ErrorCategory,
    WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

fn budget_exhaustion_program() -> WorkspaceFile {
    main_file(
        r#"
package main

func main() {
    for {
    }
}
"#,
    )
}

fn sleep_then_budget_exhaustion_program() -> WorkspaceFile {
    main_file(
        r#"
package main

import "time"

func main() {
    time.Sleep(1000000)
    for {
    }
}
"#,
    )
}

fn deadlock_program() -> WorkspaceFile {
    main_file(
        r#"
package main

func main() {
    done := make(chan bool)
    <-done
}
"#,
    )
}

#[test]
fn run_returns_structured_runtime_diagnostics_for_direct_failures() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main

func explode() {
    value := 0
    _ = 1 / value
}

func main() {
    explode()
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert!(diagnostic
                .message
                .contains("division by zero in function `explode`"));
            assert_eq!(diagnostic.category, ErrorCategory::RuntimeTrap);
            assert_eq!(diagnostic.file_path.as_deref(), Some("main.go"));
            assert_eq!(
                diagnostic.position.as_ref().map(|position| position.line),
                Some(6)
            );
            assert_eq!(
                diagnostic.source_span.as_ref().map(|span| (
                    span.start.line,
                    span.start.column,
                    span.end.line,
                    span.end.column
                )),
                Some((6, 5, 6, 17))
            );
            assert_eq!(
                diagnostic
                    .source_excerpt
                    .as_ref()
                    .map(|excerpt| excerpt.line),
                Some(6)
            );
            assert_eq!(diagnostic.suggested_action, None);
            let runtime = diagnostic
                .runtime
                .as_ref()
                .expect("runtime failures should carry structured diagnostics");
            assert!(runtime
                .root_message
                .contains("division by zero in function `explode`"));
            assert_eq!(runtime.category, ErrorCategory::RuntimeTrap);
            assert_eq!(runtime.stack_trace.len(), 2);
            assert_eq!(runtime.stack_trace[0].function, "explode");
            assert_eq!(runtime.stack_trace[1].function, "main");
            let location = runtime.stack_trace[0]
                .source_location
                .as_ref()
                .expect("faulting frame should resolve a source location");
            assert_eq!(location.path, "main.go");
            assert_eq!(location.line, 6);
            assert_eq!(location.column, 5);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn resume_returns_structured_runtime_diagnostics_after_capability_waits() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main

import "time"

func explode() {
    value := 0
    _ = 1 / value
}

func main() {
    time.Sleep(1000000)
    explode()
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(0),
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 1 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep { unix_millis: 1 },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert!(diagnostic
                .message
                .contains("division by zero in function `explode`"));
            assert_eq!(diagnostic.category, ErrorCategory::RuntimeTrap);
            let runtime = diagnostic
                .runtime
                .as_ref()
                .expect("resume-time runtime failures should carry structured diagnostics");
            assert!(runtime
                .root_message
                .contains("division by zero in function `explode`"));
            assert_eq!(runtime.category, ErrorCategory::RuntimeTrap);
            assert_eq!(runtime.stack_trace.len(), 2);
            assert_eq!(runtime.stack_trace[0].function, "explode");
            assert_eq!(runtime.stack_trace[1].function, "main");
            let location = runtime.stack_trace[0]
                .source_location
                .as_ref()
                .expect("faulting frame should resolve a source location");
            assert_eq!(location.path, "main.go");
            assert_eq!(location.line, 8);
            assert_eq!(location.column, 5);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_returns_budget_exhaustion_diagnostics_when_budget_is_exceeded() {
    let mut engine = Engine::with_instruction_budget(50);
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![budget_exhaustion_program()],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert!(diagnostic.message.contains("instruction budget exhausted"));
            assert_eq!(diagnostic.category, ErrorCategory::RuntimeBudgetExhaustion);
            assert_eq!(diagnostic.file_path.as_deref(), Some("main.go"));
            assert_eq!(
                diagnostic.suggested_action.as_deref(),
                Some("Increase the instruction budget or reduce work per run.")
            );
            let runtime = diagnostic
                .runtime
                .as_ref()
                .expect("budget failures should carry structured runtime diagnostics");
            assert!(runtime
                .root_message
                .contains("instruction budget exhausted"));
            assert_eq!(runtime.category, ErrorCategory::RuntimeBudgetExhaustion);
            assert_eq!(runtime.stack_trace[0].function, "main");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn resume_returns_budget_exhaustion_diagnostics_after_capability_waits() {
    let mut engine = Engine::with_instruction_budget(50);
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![sleep_then_budget_exhaustion_program()],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(0),
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 1 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep { unix_millis: 1 },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert!(diagnostic.message.contains("instruction budget exhausted"));
            assert_eq!(diagnostic.category, ErrorCategory::RuntimeBudgetExhaustion);
            let runtime = diagnostic
                .runtime
                .as_ref()
                .expect("resume-time budget failures should carry structured diagnostics");
            assert!(runtime
                .root_message
                .contains("instruction budget exhausted"));
            assert_eq!(runtime.category, ErrorCategory::RuntimeBudgetExhaustion);
            assert_eq!(runtime.stack_trace[0].function, "main");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_returns_deadlock_diagnostics_with_deadlock_category() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![deadlock_program()],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert!(diagnostic.message.contains("all goroutines are blocked"));
            assert_eq!(diagnostic.category, ErrorCategory::RuntimeDeadlock);
            assert_eq!(
                diagnostic.suggested_action.as_deref(),
                Some("Unblock the program by adding a ready goroutine, channel partner, or default path.")
            );
            assert!(
                diagnostic.runtime.is_none(),
                "deadlock failures should not synthesize a runtime stack without frames"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn invalid_json_returns_a_protocol_error_category() {
    let response_json = Engine::new().handle_request_json("{not json");
    let response: EngineResponse =
        serde_json::from_str(&response_json).expect("fatal response should deserialize");
    match response {
        EngineResponse::Fatal { message, category } => {
            assert!(message.contains("invalid engine request json"));
            assert_eq!(category, ErrorCategory::ProtocolError);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
