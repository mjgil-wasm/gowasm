use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

fn dense_ready_select_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 22,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 4 },
                Instruction::LoadInt { dst: 5, value: 5 },
                Instruction::LoadInt { dst: 6, value: 11 },
                Instruction::LoadInt { dst: 7, value: 22 },
                Instruction::LoadInt { dst: 8, value: 33 },
                Instruction::LoadInt { dst: 9, value: 44 },
                Instruction::LoadInt { dst: 10, value: 55 },
                Instruction::LoadNilChannel {
                    dst: 11,
                    concrete_type: None,
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
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 14,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 15,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 16,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::ChanSend { chan: 12, value: 6 },
                Instruction::ChanSend { chan: 14, value: 8 },
                Instruction::ChanSend {
                    chan: 16,
                    value: 10,
                },
                Instruction::CloseChannel { chan: 16 },
                Instruction::LoadBool {
                    dst: 20,
                    value: true,
                },
                Instruction::Select {
                    choice_dst: 17,
                    cases: vec![
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Send { value: 7 },
                        },
                        SelectCaseOp {
                            chan: 14,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 15,
                            kind: SelectCaseOpKind::Send { value: 9 },
                        },
                        SelectCaseOp {
                            chan: 16,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 6,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 19,
                    right: 20,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 17,
                    cases: vec![
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Send { value: 7 },
                        },
                        SelectCaseOp {
                            chan: 14,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 15,
                            kind: SelectCaseOpKind::Send { value: 9 },
                        },
                        SelectCaseOp {
                            chan: 16,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 2,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 17,
                    cases: vec![
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Send { value: 7 },
                        },
                        SelectCaseOp {
                            chan: 14,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 15,
                            kind: SelectCaseOpKind::Send { value: 9 },
                        },
                        SelectCaseOp {
                            chan: 16,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 8,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 19,
                    right: 20,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 17,
                    cases: vec![
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Send { value: 7 },
                        },
                        SelectCaseOp {
                            chan: 14,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 15,
                            kind: SelectCaseOpKind::Send { value: 9 },
                        },
                        SelectCaseOp {
                            chan: 16,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 17,
                    cases: vec![
                        SelectCaseOp {
                            chan: 11,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 12,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 13,
                            kind: SelectCaseOpKind::Send { value: 7 },
                        },
                        SelectCaseOp {
                            chan: 14,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                        SelectCaseOp {
                            chan: 15,
                            kind: SelectCaseOpKind::Send { value: 9 },
                        },
                        SelectCaseOp {
                            chan: 16,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 18,
                                ok_dst: Some(19),
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 17,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 10,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 19,
                    right: 20,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 18, chan: 13 },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 18, chan: 15 },
                Instruction::Compare {
                    dst: 21,
                    op: CompareOp::Equal,
                    left: 18,
                    right: 9,
                },
                Instruction::JumpIfFalse {
                    cond: 21,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

#[test]
fn blocking_select_rotates_across_dense_ready_sets() {
    let program = dense_ready_select_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate across dense ready sets");
}

#[test]
fn default_select_rotates_across_dense_ready_sets() {
    let program = dense_ready_select_program(Some(6));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate across dense ready sets");
}
