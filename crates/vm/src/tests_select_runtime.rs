use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

#[test]
fn nil_receive_select_with_default_chooses_default() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::Select {
                    choice_dst: 1,
                    cases: vec![SelectCaseOp {
                        chan: 0,
                        kind: SelectCaseOpKind::Recv {
                            value_dst: 2,
                            ok_dst: None,
                        },
                    }],
                    default_case: Some(1),
                },
                Instruction::Compare {
                    dst: 4,
                    op: CompareOp::Equal,
                    left: 1,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 4,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("nil receive select with default should choose default");
}

#[test]
fn nil_send_select_with_default_chooses_default() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::Select {
                    choice_dst: 1,
                    cases: vec![SelectCaseOp {
                        chan: 0,
                        kind: SelectCaseOpKind::Send { value: 2 },
                    }],
                    default_case: Some(1),
                },
                Instruction::Compare {
                    dst: 4,
                    op: CompareOp::Equal,
                    left: 1,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 4,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("nil send select with default should choose default");
}

#[test]
fn nil_receive_select_without_default_deadlocks() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::Select {
                    choice_dst: 1,
                    cases: vec![SelectCaseOp {
                        chan: 0,
                        kind: SelectCaseOpKind::Recv {
                            value_dst: 2,
                            ok_dst: None,
                        },
                    }],
                    default_case: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil receive select without default should deadlock");
    assert!(matches!(error, super::VmError::Deadlock));
}

#[test]
fn nil_send_select_without_default_deadlocks() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::Select {
                    choice_dst: 1,
                    cases: vec![SelectCaseOp {
                        chan: 0,
                        kind: SelectCaseOpKind::Send { value: 2 },
                    }],
                    default_case: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil send select without default should deadlock");
    assert!(matches!(error, super::VmError::Deadlock));
}

#[test]
fn blocking_select_rotates_mixed_ready_send_and_receive_cases() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 11,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
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
                Instruction::LoadInt { dst: 8, value: 1 },
                Instruction::LoadInt { dst: 9, value: 33 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::ChanSend { chan: 2, value: 9 },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 8,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 7, chan: 3 },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 7, chan: 2 },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 9,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should rotate mixed send and receive cases");
}

#[test]
fn default_select_rotates_mixed_ready_send_and_receive_cases() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 11,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
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
                Instruction::LoadInt { dst: 8, value: 1 },
                Instruction::LoadInt { dst: 9, value: 33 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::ChanSend { chan: 2, value: 9 },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 1,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Select {
                    choice_dst: 6,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 7,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 5 },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 8,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 7, chan: 3 },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 7, chan: 2 },
                Instruction::Compare {
                    dst: 10,
                    op: CompareOp::Equal,
                    left: 7,
                    right: 9,
                },
                Instruction::JumpIfFalse {
                    cond: 10,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should rotate mixed send and receive cases");
}

#[test]
fn blocking_select_ignores_nil_receive_when_live_receive_is_ready() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::ChanSend { chan: 3, value: 4 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 6,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 6,
                                ok_dst: None,
                            },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should ignore nil receive when live receive is ready");
}

#[test]
fn default_select_ignores_nil_receive_when_live_receive_is_ready() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::ChanSend { chan: 3, value: 4 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 6,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 6,
                                ok_dst: None,
                            },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should ignore nil receive when live receive is ready");
}

#[test]
fn blocking_select_ignores_nil_send_when_live_send_is_ready() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Send { value: 4 },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 4 },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 6, chan: 3 },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should ignore nil send when live send is ready");
}

#[test]
fn default_select_ignores_nil_send_when_live_send_is_ready() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Send { value: 4 },
                        },
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 4 },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 7,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 6, chan: 3 },
                Instruction::Compare {
                    dst: 8,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 8,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should ignore nil send when live send is ready");
}
