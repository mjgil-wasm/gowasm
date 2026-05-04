use super::{
    CompareOp, Function, GoroutineStatus, Instruction, Program, SelectCaseOp, SelectCaseOpKind,
    Value, ValueData, Vm,
};

fn wide_mixed_select_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 21,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 3 },
                Instruction::LoadInt { dst: 3, value: 4 },
                Instruction::LoadInt { dst: 4, value: 5 },
                Instruction::LoadInt { dst: 5, value: 11 },
                Instruction::LoadInt { dst: 6, value: 33 },
                Instruction::LoadInt { dst: 7, value: 22 },
                Instruction::LoadNilChannel {
                    dst: 8,
                    concrete_type: None,
                },
                Instruction::LoadNilChannel {
                    dst: 9,
                    concrete_type: None,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 10,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 11,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 12,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 13,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::ChanSend { chan: 10, value: 5 },
                Instruction::ChanSend { chan: 12, value: 7 },
                Instruction::CloseChannel { chan: 13 },
                Instruction::LoadBool {
                    dst: 14,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 15,
                    value: false,
                },
                Instruction::Select {
                    choice_dst: 16,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 16,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 14,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 16,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 16,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 16,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 16,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 14,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 16,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 17,
                                ok_dst: Some(18),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 16,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 15,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 20, chan: 11 },
                Instruction::Compare {
                    dst: 19,
                    op: CompareOp::Equal,
                    left: 20,
                    right: 6,
                },
                Instruction::JumpIfFalse {
                    cond: 19,
                    target: 999,
                },
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

fn channel_id(value: &Value) -> u64 {
    let ValueData::Channel(channel) = &value.data else {
        panic!("expected channel value");
    };
    channel.id.expect("channel should be live")
}

#[test]
fn blocking_select_rotates_across_wider_nil_live_and_closed_case_sets() {
    let program = wide_mixed_select_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate across wider mixed case sets");
}

#[test]
fn default_select_rotates_across_wider_nil_live_and_closed_case_sets() {
    let program = wide_mixed_select_program(Some(6));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate across wider mixed case sets");
}

#[test]
fn close_wakes_long_receive_wait_queue_with_zero_false_results() {
    let program = queue_program(3);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(0, Value::int(0));
    let channel_id = channel_id(&channel);

    let mut blocked = Vec::new();
    for _ in 0..4 {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("goroutine should spawn");
        let index = vm
            .goroutines
            .iter()
            .position(|candidate| candidate.id == goroutine)
            .expect("goroutine should exist");
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_recv_ok(&program, 1, 2, 0)
            .expect("receive should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        blocked.push(index);
    }

    assert_eq!(vm.channels[channel_id as usize].pending_receivers.len(), 4);
    vm.close_channel_by_id(&program, channel_id)
        .expect("close should wake the blocked receivers");
    assert!(vm.channels[channel_id as usize]
        .pending_receivers
        .is_empty());

    for index in blocked {
        let frame = vm.goroutines[index]
            .frames
            .last()
            .expect("blocked goroutine should still have a frame");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Runnable);
        assert_eq!(frame.registers[1], Value::int(0));
        assert_eq!(frame.registers[2], Value::bool(false));
    }
}

#[test]
fn close_wakes_long_send_wait_queue_with_send_on_closed_errors() {
    let program = queue_program(2);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(0, Value::int(0));
    let channel_id = channel_id(&channel);

    let mut blocked = Vec::new();
    for value in 1..=4 {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("goroutine should spawn");
        let index = vm
            .goroutines
            .iter()
            .position(|candidate| candidate.id == goroutine)
            .expect("goroutine should exist");
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.set_register_on_goroutine(&program, goroutine, 1, Value::int(value))
            .expect("value register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_send(&program, 0, 1)
            .expect("send should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        blocked.push(index);
    }

    assert_eq!(vm.channels[channel_id as usize].pending_sends.len(), 4);
    vm.close_channel_by_id(&program, channel_id)
        .expect("close should wake the blocked senders");
    assert!(vm.channels[channel_id as usize].pending_sends.is_empty());

    for index in blocked {
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Runnable);
        assert!(matches!(
            vm.goroutines[index].pending_error,
            Some(super::VmError::SendOnClosedChannel { .. })
        ));
    }
}
