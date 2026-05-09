use gowasm_host_types::{CapabilityResult, EngineRequest, EngineResponse, WorkspaceFile};

use super::Engine;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn engine_run_resume_transcript_replays_browser_style_sleep_flow() {
    let request = EngineRequest::Run {
        files: vec![main_file(
            r#"
package main

import (
    "fmt"
    "time"
)

func main() {
    time.Sleep(1000000)
    time.Sleep(1000000)
    fmt.Println("replayed browser flow")
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    };

    let mut engine = Engine::new();
    let first = engine.handle_request(request.clone());
    let run_id = match &first {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(
                capability,
                &gowasm_host_types::CapabilityRequest::Sleep { duration_millis: 1 }
            );
            *run_id
        }
        other => panic!("unexpected first response: {other:?}"),
    };

    let second_request = EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep { unix_millis: 1 },
    };
    let second = engine.handle_request(second_request.clone());
    match &second {
        EngineResponse::CapabilityRequest {
            run_id: resumed,
            capability,
        } => {
            assert_eq!(*resumed, run_id);
            assert_eq!(
                capability,
                &gowasm_host_types::CapabilityRequest::Sleep { duration_millis: 1 }
            );
        }
        other => panic!("unexpected second response: {other:?}"),
    }

    let third_request = EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep { unix_millis: 2 },
    };
    let third = engine.handle_request(third_request.clone());
    match &third {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "replayed browser flow\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected third response: {other:?}"),
    }

    let transcript = vec![
        (request, first),
        (second_request, second),
        (third_request, third),
    ];

    let mut replay_engine = Engine::new();
    for (request, expected_response) in transcript {
        let actual_response = replay_engine.handle_request(request);
        assert_eq!(actual_response, expected_response);
    }
}
