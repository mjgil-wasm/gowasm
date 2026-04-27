use gowasm_host_types::{
    FetchResult, VmReplayCapabilityRequest, VmReplayCapabilityResponse, VmReplayConfig,
    VmReplayEvent, VmReplayTerminal, VmReplayTrace,
};

#[test]
fn vm_replay_trace_round_trips_through_json() {
    let trace = VmReplayTrace {
        config: VmReplayConfig {
            capability_requests_enabled: true,
            instruction_yield_interval: 2,
            instruction_budget: Some(123),
            gc_allocation_threshold: Some(7),
            fixed_time_now_override_unix_nanos: Some(99),
            initial_rng_seed: 42,
        },
        events: vec![
            VmReplayEvent::SchedulerPick { goroutine_id: 1 },
            VmReplayEvent::Instruction {
                goroutine_id: 1,
                function: 0,
                instruction_index: 3,
            },
            VmReplayEvent::CapabilityRequest {
                capability: VmReplayCapabilityRequest::Sleep { duration_nanos: 5 },
            },
            VmReplayEvent::CapabilityResponse {
                response: VmReplayCapabilityResponse::Sleep {
                    elapsed_nanos: 5,
                    fired_at_unix_nanos: Some(10),
                },
            },
            VmReplayEvent::RandomSeed { seed: 42 },
            VmReplayEvent::RandomAdvance { state: 77 },
            VmReplayEvent::CapabilityResponse {
                response: VmReplayCapabilityResponse::Fetch {
                    result: FetchResult::Error {
                        message: "boom".into(),
                    },
                },
            },
        ],
        stdout: "ok\n".into(),
        terminal: VmReplayTerminal::Completed,
    };

    let json = serde_json::to_string(&trace).expect("trace should serialize");
    let decoded: VmReplayTrace = serde_json::from_str(&json).expect("trace should deserialize");
    assert_eq!(decoded, trace);
}
