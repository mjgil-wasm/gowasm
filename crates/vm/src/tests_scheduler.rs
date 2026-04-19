use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{
    resolve_stdlib_function, CapabilityRequest, Function, Instruction, Program, RunOutcome,
    SchedulerState, SelectCaseOp, SelectCaseOpKind, Value, ValueData, Vm, TYPE_TIME_DURATION,
};

fn scheduler_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 0,
            code: vec![Instruction::Return { src: None }],
        }],
    }
}

fn sleep_program() -> Program {
    let sleep = resolve_stdlib_function("time", "Sleep").expect("time.Sleep should exist");
    Program {
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
    }
}

fn yield_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn queue_program(register_count: usize) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "worker".into(),
            param_count: 0,
            register_count,
            code: vec![Instruction::Return { src: None }],
        }],
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "non-string panic payload".into()
}

fn channel_id(value: &Value) -> u64 {
    let ValueData::Channel(channel) = &value.data else {
        panic!("expected channel value");
    };
    channel.id.expect("channel should be live")
}

fn time_unix_nanos(value: &Value) -> i64 {
    let ValueData::Struct(fields) = &value.data else {
        panic!("expected time value");
    };
    let (_, nanos) = fields
        .iter()
        .find(|(name, _)| name == "__time_unix_nanos")
        .expect("time value should carry unix nanos");
    let ValueData::Int(nanos) = nanos.data else {
        panic!("expected unix nanos field to be an int");
    };
    nanos
}

#[test]
fn scheduler_state_tracks_runnable_blocked_wake_and_done() {
    let program = scheduler_program();
    let mut vm = Vm::new();
    assert_eq!(vm.scheduler_state(), SchedulerState::Done);

    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    assert_eq!(vm.scheduler_state(), SchedulerState::Runnable);

    vm.sleep_current_goroutine(5);
    assert_eq!(vm.scheduler_state(), SchedulerState::Blocked);
    assert_eq!(vm.sleeping_goroutines.get(&goroutine), Some(&5));

    vm.advance_timers(&program, 4, None)
        .expect("partial timer advance should succeed");
    assert_eq!(vm.scheduler_state(), SchedulerState::Blocked);
    assert_eq!(vm.sleeping_goroutines.get(&goroutine), Some(&1));

    vm.advance_timers(&program, 1, None)
        .expect("final timer advance should wake the goroutine");
    assert_eq!(vm.scheduler_state(), SchedulerState::Runnable);
    assert!(!vm.sleeping_goroutines.contains_key(&goroutine));

    vm.current_goroutine_mut().frames.clear();
    vm.finish_current_goroutine_if_idle();
    assert_eq!(vm.scheduler_state(), SchedulerState::Done);
}

#[test]
fn scheduler_state_tracks_paused_host_wait_and_cancelled_terminal_state() {
    let program = sleep_program();
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    assert_eq!(
        vm.start_program(&program)
            .expect("sleeping program should pause for a host wait"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos: 1 })
    );
    assert_eq!(vm.scheduler_state(), SchedulerState::PausedHostWait);

    let goroutine = vm.current_goroutine_id();
    assert_eq!(
        vm.current_goroutine().status,
        super::GoroutineStatus::Blocked
    );
    assert!(vm.sleeping_goroutines.contains_key(&goroutine));

    vm.cancel_run();
    assert_eq!(vm.scheduler_state(), SchedulerState::Cancelled);

    vm.wake_goroutine(goroutine);
    assert_eq!(vm.scheduler_state(), SchedulerState::Cancelled);
}

#[test]
fn cooperative_yield_requests_pause_and_resume_without_rewinding_progress() {
    let program = yield_program();
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(2);

    assert_eq!(
        vm.start_program(&program)
            .expect("yielding program should pause for a host yield"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );
    assert_eq!(vm.scheduler_state(), SchedulerState::PausedHostWait);
    assert_eq!(vm.globals[0], Value::int(1));

    assert_eq!(
        vm.resume_program(&program)
            .expect("yielding program should pause a second time"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );
    assert_eq!(vm.globals[0], Value::int(2));

    assert_eq!(
        vm.resume_program(&program)
            .expect("yielding program should complete"),
        RunOutcome::Completed
    );
    assert_eq!(vm.globals[0], Value::int(2));
}

#[test]
fn wake_goroutine_does_not_revive_done_goroutines() {
    let program = scheduler_program();
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");

    vm.current_goroutine_mut().frames.clear();
    vm.finish_current_goroutine_if_idle();
    assert_eq!(vm.current_goroutine().status, super::GoroutineStatus::Done);
    assert_eq!(vm.scheduler_state(), SchedulerState::Done);

    vm.wake_goroutine(goroutine);
    assert_eq!(vm.current_goroutine().status, super::GoroutineStatus::Done);
    assert_eq!(vm.scheduler_state(), SchedulerState::Done);
}

#[test]
fn advance_to_next_runnable_rotates_deterministically_and_skips_blocked_goroutines() {
    let program = scheduler_program();
    let mut vm = Vm::new();
    let first = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("first goroutine should spawn");
    let second = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("second goroutine should spawn");
    let third = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("third goroutine should spawn");

    assert_eq!(vm.current_goroutine_id(), first);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), second);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), third);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), first);

    let blocked_index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == second)
        .expect("blocked goroutine should exist");
    vm.goroutines[blocked_index].status = super::GoroutineStatus::Blocked;

    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), third);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), first);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), third);

    vm.wake_goroutine(second);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), first);
    assert!(vm.advance_to_next_runnable());
    assert_eq!(vm.current_goroutine_id(), second);
}

#[test]
fn scheduler_rotation_stays_deterministic_under_many_runnable_goroutines() {
    let program = scheduler_program();
    let mut vm = Vm::new();
    let mut goroutines = Vec::new();
    for _ in 0..8 {
        goroutines.push(
            vm.spawn_goroutine(&program, program.entry_function, Vec::new())
                .expect("goroutine should spawn"),
        );
    }

    let mut observed = Vec::new();
    for _ in 0..24 {
        assert!(vm.advance_to_next_runnable());
        observed.push(vm.current_goroutine_id());
    }

    let mut expected = Vec::new();
    for round in 0..24 {
        expected.push(goroutines[(round + 1) % goroutines.len()]);
    }
    assert_eq!(observed, expected);
}

#[test]
fn cooperative_yield_pause_sequence_is_repeatable_across_runs() {
    fn run_sequence(program: &Program) -> Vec<RunOutcome> {
        let mut vm = Vm::new();
        vm.enable_capability_requests();
        vm.set_instruction_yield_interval(2);
        let mut outcomes = Vec::new();
        outcomes.push(
            vm.start_program(program)
                .expect("yielding program should start cleanly"),
        );
        while outcomes.last() != Some(&RunOutcome::Completed) {
            outcomes.push(
                vm.resume_program(program)
                    .expect("yielding program should resume cleanly"),
            );
        }
        assert_eq!(vm.globals[0], Value::int(2));
        outcomes
    }

    let program = yield_program();
    let first = run_sequence(&program);
    let second = run_sequence(&program);

    assert_eq!(
        first,
        vec![
            RunOutcome::CapabilityRequest(CapabilityRequest::Yield),
            RunOutcome::CapabilityRequest(CapabilityRequest::Yield),
            RunOutcome::Completed,
        ]
    );
    assert_eq!(second, first);
}

#[test]
fn cancelled_paused_runs_ignore_later_timer_advances_and_resume_requests() {
    let program = sleep_program();
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    assert_eq!(
        vm.start_program(&program)
            .expect("sleeping program should pause for a host wait"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos: 1 })
    );
    let goroutine = vm.current_goroutine_id();
    vm.cancel_run();
    assert_eq!(vm.scheduler_state(), SchedulerState::Cancelled);

    vm.advance_timers(&program, 10, Some(99))
        .expect("timer advancement after cancel should be harmless");
    assert_eq!(vm.scheduler_state(), SchedulerState::Cancelled);
    assert!(!vm.sleeping_goroutines.contains_key(&goroutine));

    assert_eq!(
        vm.resume_program(&program)
            .expect("cancelled runs should not resume execution"),
        RunOutcome::Completed
    );
}

#[test]
fn cancelled_blocked_select_runs_resume_as_completed_instead_of_deadlocking() {
    let program = queue_program(5);
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");
    let first = vm.alloc_channel_value(0, Value::int(0));
    let second = vm.alloc_channel_value(0, Value::int(0));

    vm.set_register_on_goroutine(&program, goroutine, 0, first)
        .expect("first channel register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 1, second)
        .expect("second channel register should be writable");
    vm.current_goroutine = index;
    vm.execute_select(
        &program,
        2,
        &[
            SelectCaseOp {
                chan: 0,
                kind: SelectCaseOpKind::Recv {
                    value_dst: 3,
                    ok_dst: Some(4),
                },
            },
            SelectCaseOp {
                chan: 1,
                kind: SelectCaseOpKind::Recv {
                    value_dst: 3,
                    ok_dst: Some(4),
                },
            },
        ],
        None,
    )
    .expect("mixed select should block");

    assert_eq!(vm.scheduler_state(), SchedulerState::Blocked);
    assert_eq!(vm.goroutines[index].status, super::GoroutineStatus::Blocked);
    assert!(vm.goroutines[index].active_select.is_some());

    vm.cancel_run();
    assert_eq!(vm.scheduler_state(), SchedulerState::Cancelled);

    assert_eq!(
        vm.resume_program(&program)
            .expect("cancelled blocked select run should not deadlock"),
        RunOutcome::Completed
    );
}

#[test]
fn wait_queue_invariants_allow_stale_select_waiters_after_close_wakeup() {
    let program = queue_program(5);
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");
    let first = vm.alloc_channel_value(0, Value::int(0));
    let first_id = channel_id(&first);
    let second = vm.alloc_channel_value(0, Value::int(0));
    let second_id = channel_id(&second);

    vm.set_register_on_goroutine(&program, goroutine, 0, first)
        .expect("first channel register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 1, second)
        .expect("second channel register should be writable");
    vm.current_goroutine = index;
    vm.execute_select(
        &program,
        2,
        &[
            SelectCaseOp {
                chan: 0,
                kind: SelectCaseOpKind::Recv {
                    value_dst: 3,
                    ok_dst: Some(4),
                },
            },
            SelectCaseOp {
                chan: 1,
                kind: SelectCaseOpKind::Recv {
                    value_dst: 3,
                    ok_dst: Some(4),
                },
            },
        ],
        None,
    )
    .expect("mixed select should block");

    vm.close_channel_by_id(&program, second_id)
        .expect("close should wake the select");
    assert_eq!(vm.goroutines[index].active_select, None);
    assert_eq!(vm.channels[first_id as usize].pending_receivers.len(), 1);
    vm.assert_channel_wait_queue_invariants();
}

#[test]
fn wait_queue_invariants_reject_duplicate_non_select_waiters() {
    let program = queue_program(1);
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");
    let channel = vm.alloc_channel_value(0, Value::int(0));
    let channel_id = channel_id(&channel);

    vm.current_goroutine = index;
    vm.block_current_goroutine();
    let duplicate = super::channels::PendingRecv {
        goroutine,
        dst: 0,
        ok_dst: None,
        select: None,
    };
    vm.channels[channel_id as usize]
        .pending_receivers
        .extend([duplicate.clone(), duplicate]);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_channel_wait_queue_invariants()
    }))
    .expect_err("duplicate waiter corruption should panic");
    assert!(panic_message(panic).contains("duplicate non-select waiter"));
}

#[test]
fn wait_queue_invariants_reject_blocked_active_select_without_waiters() {
    let program = queue_program(0);
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");

    vm.current_goroutine = index;
    vm.block_current_goroutine();
    vm.set_current_goroutine_select(Some(99));

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_channel_wait_queue_invariants()
    }))
    .expect_err("missing active-select waiters should panic");
    assert!(panic_message(panic).contains("blocked select goroutine"));
}

#[test]
fn resume_program_requests_the_shortest_duration_across_mixed_timer_sources() {
    let program = queue_program(0);
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    let context_id = 7;
    vm.context_values.insert(context_id, Default::default());

    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");
    vm.current_goroutine = index;
    vm.sleep_current_goroutine(9);

    let timer_channel = vm.alloc_channel_value(1, Value::int(0));
    vm.schedule_time_channel_send(channel_id(&timer_channel), 5);
    vm.schedule_context_deadline(context_id, 7);

    assert_eq!(
        vm.resume_program(&program)
            .expect("blocked timers should request a host sleep"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos: 5 })
    );
}

#[test]
fn advance_timers_wakes_and_fires_mixed_timer_sources_in_order() {
    let program = queue_program(0);
    let mut vm = Vm::new();

    let sleep_first = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("first goroutine should spawn");
    let first_index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == sleep_first)
        .expect("first goroutine should exist");
    vm.current_goroutine = first_index;
    vm.sleep_current_goroutine(3);

    let sleep_second = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("second goroutine should spawn");
    let second_index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == sleep_second)
        .expect("second goroutine should exist");
    vm.current_goroutine = second_index;
    vm.sleep_current_goroutine(6);

    let first_timer = vm.alloc_channel_value(1, Value::int(0));
    let first_timer_id = channel_id(&first_timer);
    let second_timer = vm.alloc_channel_value(1, Value::int(0));
    let second_timer_id = channel_id(&second_timer);
    vm.schedule_time_channel_send(first_timer_id, 3);
    vm.schedule_time_channel_send(second_timer_id, 6);

    let done_channel = vm.alloc_channel_value(0, Value::int(0));
    let done_channel_id = channel_id(&done_channel);
    let context_id = 11;
    vm.context_values.insert(
        context_id,
        super::ContextState {
            done_channel_id: Some(done_channel_id),
            ..Default::default()
        },
    );
    vm.schedule_context_deadline(context_id, 6);

    vm.advance_timers(&program, 3, Some(123))
        .expect("first timer advance should succeed");
    assert_eq!(
        vm.goroutines[first_index].status,
        super::GoroutineStatus::Runnable
    );
    assert_eq!(
        vm.goroutines[second_index].status,
        super::GoroutineStatus::Blocked
    );
    assert_eq!(vm.sleeping_goroutines.get(&sleep_second), Some(&3));
    assert_eq!(vm.time_channel_timers.len(), 1);
    assert_eq!(vm.time_channel_timers[0].channel_id, second_timer_id);
    assert_eq!(vm.time_channel_timers[0].remaining_nanos, 3);
    assert_eq!(vm.context_deadline_timers.len(), 1);
    assert_eq!(vm.context_deadline_timers[0].remaining_nanos, 3);
    assert_eq!(
        time_unix_nanos(
            vm.channels[first_timer_id as usize]
                .buffer
                .front()
                .expect("first timer channel should receive a value")
        ),
        123
    );
    assert!(vm.channels[done_channel_id as usize]
        .pending_receivers
        .is_empty());
    assert!(vm.context_values[&context_id].err.is_none());

    vm.advance_timers(&program, 3, Some(456))
        .expect("second timer advance should succeed");
    assert_eq!(
        vm.goroutines[second_index].status,
        super::GoroutineStatus::Runnable
    );
    assert!(!vm.sleeping_goroutines.contains_key(&sleep_second));
    assert!(vm.time_channel_timers.is_empty());
    assert!(vm.context_deadline_timers.is_empty());
    assert_eq!(
        time_unix_nanos(
            vm.channels[second_timer_id as usize]
                .buffer
                .front()
                .expect("second timer channel should receive a value")
        ),
        456
    );
    assert!(vm.channels[done_channel_id as usize].closed);
    assert_eq!(
        vm.context_values[&context_id]
            .err
            .as_ref()
            .expect("deadline should populate an error"),
        &Value::error("context deadline exceeded")
    );
}
