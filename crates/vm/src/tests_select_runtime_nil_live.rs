use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

#[test]
fn blocking_select_ignores_nil_send_when_live_receive_is_ready() {
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
                            kind: SelectCaseOpKind::Send { value: 4 },
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
        .expect("blocking select should ignore nil send when live receive is ready");
}

#[test]
fn default_select_ignores_nil_send_when_live_receive_is_ready() {
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
                            kind: SelectCaseOpKind::Send { value: 4 },
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
        .expect("default select should ignore nil send when live receive is ready");
}
