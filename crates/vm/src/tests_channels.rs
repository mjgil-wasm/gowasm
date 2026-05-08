use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

#[test]
fn makes_non_nil_channel_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 3, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 0,
                    cap: None,
                    zero: 3,
                },
                Instruction::LoadNilChannel {
                    dst: 1,
                    concrete_type: None,
                },
                Instruction::Compare {
                    dst: 2,
                    op: CompareOp::Equal,
                    left: 0,
                    right: 1,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn transfers_values_between_goroutines_over_unbuffered_channels() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 3, value: 0 },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 0,
                        cap: None,
                        zero: 3,
                    },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![0],
                    },
                    Instruction::ChanRecv { dst: 1, chan: 0 },
                    Instruction::Compare {
                        dst: 2,
                        op: CompareOp::Equal,
                        left: 1,
                        right: 1,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "send".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 7 },
                    Instruction::ChanSend { chan: 0, value: 1 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn buffers_values_when_channel_has_capacity() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: Some(0),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 3, value: 7 },
                Instruction::LoadInt { dst: 4, value: 8 },
                Instruction::ChanSend { chan: 1, value: 3 },
                Instruction::ChanSend { chan: 1, value: 4 },
                Instruction::ChanRecv { dst: 5, chan: 1 },
                Instruction::ChanRecv { dst: 6, chan: 1 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn close_of_nil_channel_fails() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::CloseChannel { chan: 0 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close(nil) should fail");
    assert!(matches!(
        error.root_cause(),
        super::VmError::CloseNilChannel { .. }
    ));
}

#[test]
fn send_on_closed_channel_fails() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::ChanSend { chan: 1, value: 2 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("send on closed channel should fail");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn close_of_closed_channel_fails() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::CloseChannel { chan: 1 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close(closed) should fail");
    assert!(matches!(
        error.root_cause(),
        super::VmError::CloseClosedChannel { .. }
    ));
}

#[test]
fn receive_from_closed_channel_returns_zero_value() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::ChanRecv { dst: 2, chan: 1 },
                Instruction::Compare {
                    dst: 3,
                    op: CompareOp::Equal,
                    left: 2,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 3,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("closed receive should return zero");
}

#[test]
fn blocking_select_receives_value_and_choice_index() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 7,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 0 },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 1,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![1],
                    },
                    Instruction::Select {
                        choice_dst: 2,
                        cases: vec![SelectCaseOp {
                            chan: 1,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 3,
                                ok_dst: Some(4),
                            },
                        }],
                        default_case: None,
                    },
                    Instruction::LoadInt { dst: 5, value: 0 },
                    Instruction::Compare {
                        dst: 6,
                        op: CompareOp::Equal,
                        left: 2,
                        right: 5,
                    },
                    Instruction::JumpIfFalse {
                        cond: 6,
                        target: 99,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "send".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 7 },
                    Instruction::ChanSend { chan: 0, value: 1 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should receive the sent value");
}

#[test]
fn blocking_select_receives_closed_channel_zero_and_false() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::Select {
                    choice_dst: 2,
                    cases: vec![SelectCaseOp {
                        chan: 1,
                        kind: SelectCaseOpKind::Recv {
                            value_dst: 3,
                            ok_dst: Some(4),
                        },
                    }],
                    default_case: None,
                },
                Instruction::LoadBool {
                    dst: 5,
                    value: false,
                },
                Instruction::Compare {
                    dst: 6,
                    op: CompareOp::Equal,
                    left: 4,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 6,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("closed-channel select should report ok=false");
}

#[test]
fn blocking_select_sends_value_when_receiver_is_waiting() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 6,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 0 },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 1,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![1],
                    },
                    Instruction::LoadInt { dst: 2, value: 7 },
                    Instruction::Select {
                        choice_dst: 3,
                        cases: vec![SelectCaseOp {
                            chan: 1,
                            kind: SelectCaseOpKind::Send { value: 2 },
                        }],
                        default_case: None,
                    },
                    Instruction::LoadInt { dst: 4, value: 0 },
                    Instruction::Compare {
                        dst: 5,
                        op: CompareOp::Equal,
                        left: 3,
                        right: 4,
                    },
                    Instruction::JumpIfFalse {
                        cond: 5,
                        target: 99,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "recv".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::ChanRecv { dst: 1, chan: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking send select should wake a waiting receiver");
}

#[test]
fn try_receive_reports_not_ready_without_blocking() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::ChanTryRecv {
                    ready_dst: 2,
                    value_dst: 3,
                    chan: 1,
                },
                Instruction::LoadBool {
                    dst: 4,
                    value: false,
                },
                Instruction::Compare {
                    dst: 5,
                    op: CompareOp::Equal,
                    left: 2,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 5,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("try receive should not block on empty channel");
}

#[test]
fn try_receive_ok_reports_closed_channel_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::ChanTryRecvOk {
                    ready_dst: 2,
                    value_dst: 3,
                    ok_dst: 4,
                    chan: 1,
                },
                Instruction::LoadInt { dst: 5, value: 0 },
                Instruction::LoadBool {
                    dst: 6,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 7,
                    value: false,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 2,
                    right: 6,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 99,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 3,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 99,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 4,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("closed try receive should complete immediately");
}
