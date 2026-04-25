use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

#[test]
fn buffered_closed_receives_drain_values_before_zero_false() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 16,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::LoadInt { dst: 3, value: 7 },
                Instruction::LoadInt { dst: 4, value: 8 },
                Instruction::ChanSend { chan: 2, value: 3 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::CloseChannel { chan: 2 },
                Instruction::ChanRecvOk {
                    value_dst: 5,
                    ok_dst: 8,
                    chan: 2,
                },
                Instruction::ChanRecvOk {
                    value_dst: 6,
                    ok_dst: 9,
                    chan: 2,
                },
                Instruction::ChanRecvOk {
                    value_dst: 7,
                    ok_dst: 10,
                    chan: 2,
                },
                Instruction::LoadBool {
                    dst: 11,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 12,
                    value: false,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 9,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 13,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 12,
                },
                Instruction::JumpIfFalse {
                    cond: 13,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("buffered closed channel should drain values before false");
}

#[test]
fn close_wakes_blocked_receive_select_with_zero_false() {
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
                    Instruction::LoadBool {
                        dst: 5,
                        value: false,
                    },
                    Instruction::Compare {
                        dst: 6,
                        op: CompareOp::Equal,
                        left: 3,
                        right: 0,
                    },
                    Instruction::JumpIfFalse {
                        cond: 6,
                        target: 999,
                    },
                    Instruction::Compare {
                        dst: 6,
                        op: CompareOp::Equal,
                        left: 4,
                        right: 5,
                    },
                    Instruction::JumpIfFalse {
                        cond: 6,
                        target: 999,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "closer".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 0 },
                    Instruction::CloseChannel { chan: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake blocked receive select with zero false");
}

#[test]
fn close_wakes_blocked_send_select_with_error() {
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
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "closer".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 0 },
                    Instruction::CloseChannel { chan: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail blocked send select");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn blocking_select_rotates_mixed_ready_closed_receives() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 13,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 5, value: 22 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::ChanSend { chan: 3, value: 5 },
                Instruction::CloseChannel { chan: 2 },
                Instruction::CloseChannel { chan: 3 },
                Instruction::LoadBool {
                    dst: 10,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 11,
                    value: false,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 10,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 10,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate mixed ready closed receives");
}

#[test]
fn default_select_rotates_mixed_ready_closed_receives() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 13,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 5, value: 22 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::ChanSend { chan: 3, value: 5 },
                Instruction::CloseChannel { chan: 2 },
                Instruction::CloseChannel { chan: 3 },
                Instruction::LoadBool {
                    dst: 10,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 11,
                    value: false,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 10,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 10,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: Some(8),
                            },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 12,
                    op: CompareOp::Equal,
                    left: 8,
                    right: 11,
                },
                Instruction::JumpIfFalse {
                    cond: 12,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate mixed ready closed receives");
}
