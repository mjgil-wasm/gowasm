use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

#[test]
fn blocking_select_ignores_nil_receive_when_live_send_is_ready() {
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
                Instruction::LoadInt { dst: 4, value: 42 },
                Instruction::LoadInt { dst: 7, value: 1 },
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
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should ignore nil receive when live send is ready");
}

#[test]
fn default_select_ignores_nil_receive_when_live_send_is_ready() {
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
                Instruction::LoadInt { dst: 4, value: 42 },
                Instruction::LoadInt { dst: 7, value: 1 },
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
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should ignore nil receive when live send is ready");
}

#[test]
fn all_nil_select_takes_default() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadNilChannel {
                    dst: 1,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 2, value: 99 },
                Instruction::LoadInt { dst: 5, value: 2 },
                Instruction::Select {
                    choice_dst: 3,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 4,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 1,
                            kind: SelectCaseOpKind::Send { value: 2 },
                        },
                    ],
                    default_case: Some(2),
                },
                Instruction::Compare {
                    dst: 6,
                    op: CompareOp::Equal,
                    left: 3,
                    right: 5,
                },
                Instruction::JumpIfFalse {
                    cond: 6,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("all-nil select with default should take the default case");
}

#[test]
fn all_nil_select_without_default_deadlocks() {
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
                Instruction::LoadNilChannel {
                    dst: 1,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 2, value: 99 },
                Instruction::Select {
                    choice_dst: 3,
                    cases: vec![
                        SelectCaseOp {
                            chan: 0,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 4,
                                ok_dst: None,
                            },
                        },
                        SelectCaseOp {
                            chan: 1,
                            kind: SelectCaseOpKind::Send { value: 2 },
                        },
                    ],
                    default_case: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let result = vm.run_program(&program);
    assert!(
        result.is_err(),
        "all-nil select without default should deadlock"
    );
}
