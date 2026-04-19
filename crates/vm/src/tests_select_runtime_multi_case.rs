use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

fn multi_case_nil_live_closed_select_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 16,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 5, value: 22 },
                Instruction::LoadNilChannel {
                    dst: 6,
                    concrete_type: None,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 7,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 8,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 9,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::ChanSend { chan: 7, value: 4 },
                Instruction::CloseChannel { chan: 9 },
                Instruction::LoadBool {
                    dst: 13,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 14,
                    value: false,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 11,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 12,
                    right: 13,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(12),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 11,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 12,
                    right: 14,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 11, chan: 8 },
                Instruction::Compare {
                    dst: 15,
                    op: CompareOp::Equal,
                    left: 11,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 15,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn multi_case_nil_live_closed_send_error_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 15,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 5, value: 22 },
                Instruction::LoadNilChannel {
                    dst: 6,
                    concrete_type: None,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 7,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 8,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 9,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::ChanSend { chan: 7, value: 4 },
                Instruction::CloseChannel { chan: 9 },
                Instruction::LoadBool {
                    dst: 12,
                    value: true,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 14,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 14,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 14,
                    op: CompareOp::Equal,
                    left: 11,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 14,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 14,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 12,
                },
                Instruction::JumpIfFalse {
                    cond: 14,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 14,
                    op: CompareOp::Equal,
                    left: 10,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 14,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 10,
                    cases: vec![
                        SelectCaseOp {
                            chan: 6,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 11,
                                ok_dst: Some(13),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn multi_case_receive_close_wakeup_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 11,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 0 },
                    Instruction::LoadInt { dst: 1, value: 1 },
                    Instruction::LoadNilChannel {
                        dst: 2,
                        concrete_type: None,
                    },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 3,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 4,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::LoadInt { dst: 9, value: 9 },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![3],
                    },
                    Instruction::Select {
                        choice_dst: 5,
                        cases: vec![
                            SelectCaseOp {
                                chan: 2,
                                kind: SelectCaseOpKind::Recv {
                                    value_dst: 6,
                                    ok_dst: Some(7),
                                },
                            },
                            SelectCaseOp {
                                chan: 3,
                                kind: SelectCaseOpKind::Recv {
                                    value_dst: 6,
                                    ok_dst: Some(7),
                                },
                            },
                            SelectCaseOp {
                                chan: 4,
                                kind: SelectCaseOpKind::Send { value: 9 },
                            },
                        ],
                        default_case: None,
                    },
                    Instruction::LoadBool {
                        dst: 8,
                        value: false,
                    },
                    Instruction::Compare {
                        dst: 10,
                        op: CompareOp::Equal,
                        left: 5,
                        right: 1,
                    },
                    Instruction::JumpIfFalse {
                        cond: 10,
                        target: 999,
                    },
                    Instruction::Compare {
                        dst: 10,
                        op: CompareOp::Equal,
                        left: 6,
                        right: 0,
                    },
                    Instruction::JumpIfFalse {
                        cond: 10,
                        target: 999,
                    },
                    Instruction::Compare {
                        dst: 10,
                        op: CompareOp::Equal,
                        left: 7,
                        right: 8,
                    },
                    Instruction::JumpIfFalse {
                        cond: 10,
                        target: 999,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "closer".into(),
                param_count: 1,
                register_count: 1,
                code: vec![
                    Instruction::CloseChannel { chan: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    }
}

fn multi_case_send_close_wakeup_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 8,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 0 },
                    Instruction::LoadNilChannel {
                        dst: 1,
                        concrete_type: None,
                    },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 2,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::MakeChannel {
                        concrete_type: None,
                        dst: 3,
                        cap: None,
                        zero: 0,
                    },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![3],
                    },
                    Instruction::LoadInt { dst: 4, value: 7 },
                    Instruction::Select {
                        choice_dst: 5,
                        cases: vec![
                            SelectCaseOp {
                                chan: 1,
                                kind: SelectCaseOpKind::Recv {
                                    value_dst: 6,
                                    ok_dst: None,
                                },
                            },
                            SelectCaseOp {
                                chan: 2,
                                kind: SelectCaseOpKind::Recv {
                                    value_dst: 6,
                                    ok_dst: None,
                                },
                            },
                            SelectCaseOp {
                                chan: 3,
                                kind: SelectCaseOpKind::Send { value: 4 },
                            },
                        ],
                        default_case: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "closer".into(),
                param_count: 1,
                register_count: 1,
                code: vec![
                    Instruction::CloseChannel { chan: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    }
}

#[test]
fn blocking_select_rotates_across_nil_live_and_closed_cases() {
    let program = multi_case_nil_live_closed_select_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate across nil, live, and closed cases");
}

#[test]
fn default_select_rotates_across_nil_live_and_closed_cases() {
    let program = multi_case_nil_live_closed_select_program(Some(4));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate across nil, live, and closed cases");
}

#[test]
fn blocking_select_errors_when_multi_case_rotation_reaches_closed_send() {
    let program = multi_case_nil_live_closed_send_error_program(None);

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("blocking select should fail once rotation reaches the closed send");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn default_select_errors_when_multi_case_rotation_reaches_closed_send() {
    let program = multi_case_nil_live_closed_send_error_program(Some(4));

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("default select should fail once rotation reaches the closed send");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn close_wakes_multi_case_receive_select_with_zero_false() {
    let program = multi_case_receive_close_wakeup_program();

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake the blocked receive case in a multi-case select");
}

#[test]
fn close_wakes_multi_case_send_select_with_error() {
    let program = multi_case_send_close_wakeup_program();

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail the blocked send case in a multi-case select");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}
