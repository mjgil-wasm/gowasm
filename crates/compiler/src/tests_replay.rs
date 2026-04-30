use super::compile_source;
use gowasm_vm::{replay_program_trace, CapabilityRequest, RunOutcome, Vm};

fn record_trace(program: &gowasm_vm::Program) -> gowasm_host_types::VmReplayTrace {
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(2);
    vm.enable_replay_capture();

    let mut now = 1_700_000_000_000_000_000i64;
    let mut outcome = vm.start_program(program).expect("program should start");
    loop {
        match outcome {
            RunOutcome::Completed => break,
            RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
                now += duration_nanos;
                vm.advance_timers(program, duration_nanos, Some(now))
                    .expect("timers should advance");
                outcome = vm.resume_program(program).expect("program should resume");
            }
            RunOutcome::CapabilityRequest(CapabilityRequest::Yield) => {
                vm.acknowledge_cooperative_yield();
                outcome = vm.resume_program(program).expect("program should resume");
            }
            other => panic!("unexpected capability request: {other:?}"),
        }
    }

    vm.take_replay_trace().expect("trace should be captured")
}

#[test]
fn replay_trace_reproduces_scheduler_channel_select_and_rng_execution() {
    let source = r#"
package main

import (
    "fmt"
    "math/rand"
    "time"
)

func main() {
    rand.Seed(7)
    fast := make(chan int, 1)
    slow := make(chan int, 1)

    go func() {
        fast <- rand.Intn(100)
    }()

    go func() {
        time.Sleep(1)
        slow <- rand.Intn(100)
    }()

    select {
    case value := <-slow:
        fmt.Println("slow", value)
    case value := <-fast:
        fmt.Println("fast", value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let trace = record_trace(&program);

    assert!(
        trace.events.iter().any(|event| matches!(
            event,
            gowasm_host_types::VmReplayEvent::SchedulerPick { .. }
        )),
        "trace should record scheduler picks"
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| matches!(event, gowasm_host_types::VmReplayEvent::SelectStart { .. })),
        "trace should record select fairness offsets"
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| matches!(event, gowasm_host_types::VmReplayEvent::RandomSeed { .. })),
        "trace should record rand.Seed"
    );
    assert!(
        trace.events.iter().any(|event| matches!(
            event,
            gowasm_host_types::VmReplayEvent::RandomAdvance { .. }
        )),
        "trace should record RNG advances"
    );
    assert!(
        trace.events.iter().any(|event| matches!(
            event,
            gowasm_host_types::VmReplayEvent::CapabilityRequest {
                capability: gowasm_host_types::VmReplayCapabilityRequest::Sleep { .. }
            }
        )),
        "trace should record sleep capability requests"
    );
    assert!(
        trace.events.iter().any(|event| matches!(
            event,
            gowasm_host_types::VmReplayEvent::CapabilityResponse {
                response: gowasm_host_types::VmReplayCapabilityResponse::Yield
            }
        )),
        "trace should record yield responses"
    );

    replay_program_trace(&program, &trace).expect("replay should reproduce the trace");
}
