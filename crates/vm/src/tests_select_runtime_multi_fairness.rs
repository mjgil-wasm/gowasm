use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

fn broader_multi_case_fairness_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 19,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 4 },
                Instruction::LoadInt { dst: 5, value: 11 },
                Instruction::LoadInt { dst: 6, value: 22 },
                Instruction::LoadInt { dst: 7, value: 33 },
                Instruction::LoadNilChannel {
                    dst: 8,
                    concrete_type: None,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 9,
                    cap: Some(1),
                    zero: 0,
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
                Instruction::ChanSend { chan: 9, value: 5 },
                Instruction::ChanSend { chan: 11, value: 7 },
                Instruction::CloseChannel { chan: 12 },
                Instruction::LoadBool {
                    dst: 16,
                    value: true,
                },
                Instruction::LoadBool {
                    dst: 17,
                    value: false,
                },
                Instruction::Select {
                    choice_dst: 13,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 15,
                    right: 16,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 13,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 13,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 15,
                    right: 16,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 13,
                    cases: vec![
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Send { value: 6 },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 14,
                                ok_dst: Some(15),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 15,
                    right: 17,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 14, chan: 10 },
                Instruction::Compare {
                    dst: 18,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 6,
                },
                Instruction::JumpIfFalse {
                    cond: 18,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn broader_multi_case_closed_send_error_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 17,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 5, value: 22 },
                Instruction::LoadInt { dst: 6, value: 33 },
                Instruction::LoadNilChannel {
                    dst: 7,
                    concrete_type: None,
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
                Instruction::ChanSend { chan: 8, value: 4 },
                Instruction::ChanSend { chan: 10, value: 6 },
                Instruction::CloseChannel { chan: 11 },
                Instruction::LoadBool {
                    dst: 15,
                    value: true,
                },
                Instruction::Select {
                    choice_dst: 12,
                    cases: vec![
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 12,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 15,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 12,
                    cases: vec![
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 12,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 12,
                    cases: vec![
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 12,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 13,
                    right: 6,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 16,
                    op: CompareOp::Equal,
                    left: 14,
                    right: 15,
                },
                Instruction::JumpIfFalse {
                    cond: 16,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 12,
                    cases: vec![
                        SelectCaseOp {
                            chan: 7,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 8,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 9,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                        SelectCaseOp {
                            chan: 10,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 13,
                                ok_dst: Some(14),
                            },
                        },
                        SelectCaseOp {
                            chan: 11,
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

#[test]
fn blocking_select_rotates_across_multiple_live_send_and_receive_cases() {
    let program = broader_multi_case_fairness_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate across multiple live send/receive cases");
}

#[test]
fn default_select_rotates_across_multiple_live_send_and_receive_cases() {
    let program = broader_multi_case_fairness_program(Some(5));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate across multiple live send/receive cases");
}

#[test]
fn blocking_select_errors_when_broader_multi_case_rotation_reaches_closed_send() {
    let program = broader_multi_case_closed_send_error_program(None);

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err(
        "blocking select should fail once the broader multi-case rotation reaches closed send",
    );
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn default_select_errors_when_broader_multi_case_rotation_reaches_closed_send() {
    let program = broader_multi_case_closed_send_error_program(Some(5));

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err(
        "default select should fail once the broader multi-case rotation reaches closed send",
    );
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}
