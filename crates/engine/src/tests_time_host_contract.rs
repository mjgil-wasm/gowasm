use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, WorkspaceFile,
};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_orders_mixed_timer_sources_across_host_resumes() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main

import (
    "context"
    "fmt"
    "time"
)

func main() {
    ctx, _ := context.WithDeadline(
        context.Background(),
        time.UnixMilli(1700000000107),
    )

    timer := time.NewTimer(5000000)
    first := <-time.After(3000000)
    fmt.Println("after", first.UnixMilli())

    second := <-timer.C
    fmt.Println("timer", second.UnixMilli())

    <-ctx.Done()
    fmt.Println(ctx.Err() == context.DeadlineExceeded)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::ClockNow);
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let run_id = match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::ClockNow {
            unix_millis: 1_700_000_000_100,
        },
    }) {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 3 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let run_id = match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_103,
        },
    }) {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let run_id = match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_105,
        },
    }) {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_107,
        },
    }) {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "after 1700000000103\ntimer 1700000000105\ntrue\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
