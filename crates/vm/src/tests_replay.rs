use super::{
    replay_program_trace, resolve_stdlib_function, CapabilityRequest, Function, Instruction,
    Program, RunOutcome, Vm, TYPE_TIME_DURATION,
};

#[test]
fn replay_trace_round_trips_yield_and_sleep_capabilities() {
    let sleep = resolve_stdlib_function("time", "Sleep").expect("time.Sleep should exist");
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::Retag {
                    dst: 1,
                    src: 0,
                    typ: TYPE_TIME_DURATION,
                },
                Instruction::CallStdlib {
                    function: sleep,
                    args: vec![1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(2);
    vm.enable_replay_capture();

    let first = vm.start_program(&program).expect("program should start");
    assert_eq!(
        first,
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );
    vm.acknowledge_cooperative_yield();

    let second = vm.resume_program(&program).expect("program should resume");
    assert_eq!(
        second,
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos: 1 })
    );
    vm.advance_timers(&program, 1, Some(99))
        .expect("timers should advance");
    assert_eq!(
        vm.resume_program(&program)
            .expect("program should complete"),
        RunOutcome::Completed
    );

    let trace = vm.take_replay_trace().expect("trace should exist");
    replay_program_trace(&program, &trace).expect("replay should succeed");
}
