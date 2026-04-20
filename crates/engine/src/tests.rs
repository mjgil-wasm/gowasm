use gowasm_host_types::{
    CapabilityRequest, CapabilityResult, EngineRequest, EngineResponse, ErrorCategory,
    WorkspaceFile,
};

use super::{handle_request, handle_request_json, Engine, ENGINE_NAME};

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn boot_reports_engine_info() {
    let response = handle_request(EngineRequest::Boot);
    match response {
        EngineResponse::Ready { info } => assert_eq!(info.engine_name, ENGINE_NAME),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_the_compiler_and_vm_path() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func main() {
    fmt.Println("hello", 42)
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
            assert_eq!(stdout, "hello 42\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_a_multi_file_workspace_package() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main

func main() {
    helper()
}
"#,
            ),
            WorkspaceFile {
                path: "helper.go".into(),
                contents: r#"
package main
import "fmt"

func helper() {
    fmt.Println("from workspace", 9)
}
"#
                .into(),
            },
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "from workspace 9\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_local_short_variable_declarations() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func main() {
    label := "hello"
    value := 5
    fmt.Println(label, value)
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
            assert_eq!(stdout, "hello 5\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_parameterized_user_functions() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func greet(name string, count int) {
    fmt.Println(name, count)
}

func main() {
    label := "hello"
    count := 6
    greet(label, count)
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
            assert_eq!(stdout, "hello 6\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_assignment_to_locals_and_parameters() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func greet(name string, count int) {
    label := "start"
    label = name
    count = 10
    fmt.Println(label, count)
}

func main() {
    greet("hello", 4)
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
            assert_eq!(stdout, "hello 10\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_single_value_returns() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func answer() int {
    return 42
}

func main() {
    value := answer()
    fmt.Println(value)
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
            assert_eq!(stdout, "42\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_addition_expressions() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func add(left int, right int) int {
    return left + right
}

func main() {
    label := "go" + "wasm"
    fmt.Println(label, add(4, 5))
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
            assert_eq!(stdout, "gowasm 9\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_injects_host_time_for_time_now() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    fmt.Println(time.Now().UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: Some(1_700_000_000_123_000_000),
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "1700000000123\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_accepts_host_time_in_milliseconds_for_browser_safe_requests() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    fmt.Println(time.Now().UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: Some(1_700_000_000_123),
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "1700000000123\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_multiple_clock_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    fmt.Println(time.Now().UnixMilli())
    fmt.Println(time.Now().UnixMilli())
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

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::ClockNow {
            unix_millis: 1_700_000_000_123,
        },
    });

    match response {
        EngineResponse::CapabilityRequest {
            run_id: resumed_run_id,
            capability,
        } => {
            assert_eq!(resumed_run_id, run_id);
            assert_eq!(capability, CapabilityRequest::ClockNow);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::ClockNow {
            unix_millis: 1_700_000_000_456,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "1700000000123\n1700000000456\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_formats_time_values_with_datetime_layouts() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    value := time.Unix(1700000000, 0)
    fmt.Println(time.DateTime)
    fmt.Println(value.Format(time.DateTime))
    fmt.Println(time.Unix(-1, 0).Format("2006/01/02 15:04:05"))
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
            assert_eq!(
                stdout,
                "2006-01-02 15:04:05\n2023-11-14 22:13:20\n1969/12/31 23:59:59\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_sleep_requests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    time.Sleep(2000000)
    fmt.Println("after sleep")
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_123,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "after sleep\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_cooperative_yields() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"

func main() {
    sum := 0
    for i := 0; i < 50000; i++ {
        sum += i
    }
    fmt.Println(sum)
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let mut run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Yield);
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    for _ in 0..128 {
        match engine.handle_request(EngineRequest::Resume {
            run_id,
            capability: CapabilityResult::Yield,
        }) {
            EngineResponse::CapabilityRequest {
                run_id: next_run_id,
                capability,
            } => {
                assert_eq!(capability, CapabilityRequest::Yield);
                run_id = next_run_id;
            }
            EngineResponse::RunResult {
                stdout,
                diagnostics,
            } => {
                assert_eq!(stdout, "1249975000\n");
                assert!(diagnostics.is_empty());
                return;
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    panic!("cooperatively yielded run should eventually complete");
}

#[test]
fn run_can_pause_and_resume_for_time_after() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    fired := <-time.After(2000000)
    fmt.Println(fired.UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_789,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "1700000000789\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_time_new_timer() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_987,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "1700000000987\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn cancel_request_clears_paused_time_new_timer_runs_and_rejects_future_resume() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "time"

func main() {
    <-time.NewTimer(2000000).C
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    assert_eq!(
        engine.handle_request(EngineRequest::Cancel),
        EngineResponse::Cancelled {
            category: ErrorCategory::RuntimeCancellation,
        }
    );

    match engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_987,
        },
    }) {
        EngineResponse::Fatal { message, category } => {
            assert_eq!(
                message,
                format!("run `{run_id}` is not waiting for a capability result")
            );
            assert_eq!(category, ErrorCategory::ProtocolError);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_stop_time_new_timer_without_host_resume() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    fmt.Println(timer.Stop())
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
            assert_eq!(stdout, "true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_and_resume_for_time_timer_reset() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(5000000)
    fmt.Println(timer.Reset(2000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_001_222,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "true\n1700000001222\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_drains_stale_timer_values_before_rearming_timers() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    time.Sleep(3000000)
    fmt.Println(timer.Reset(4000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_001_444,
        },
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
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_001_445,
        },
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 4 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_001_449,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "false\n1700000001449\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_can_pause_for_context_timeout_clock_and_sleep() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "context"
import "fmt"

func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 2000000)
    defer cancel()

    <-ctx.Done()
    fmt.Println(ctx.Err().Error())
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

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::ClockNow {
            unix_millis: 1_700_000_000_123,
        },
    });

    let run_id = match response {
        EngineResponse::CapabilityRequest { run_id, capability } => {
            assert_eq!(capability, CapabilityRequest::Sleep { duration_millis: 2 });
            run_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::Resume {
        run_id,
        capability: CapabilityResult::Sleep {
            unix_millis: 1_700_000_000_125,
        },
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "context deadline exceeded\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn invalid_json_returns_a_fatal_response() {
    let response_json = handle_request_json("{not json");
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
