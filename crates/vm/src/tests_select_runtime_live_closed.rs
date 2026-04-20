use super::{CompareOp, Function, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Vm};

fn live_receive_wins_over_closed_send_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::CloseChannel { chan: 3 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
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
                    default_case,
                },
                Instruction::Compare {
                    dst: 7,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 7,
                    target: 999,
                },
                Instruction::Compare {
                    dst: 7,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 7,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn closed_send_before_live_receive_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::ChanSend { chan: 2, value: 4 },
                Instruction::CloseChannel { chan: 3 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 3,
                            kind: SelectCaseOpKind::Send { value: 4 },
                        },
                        SelectCaseOp {
                            chan: 2,
                            kind: SelectCaseOpKind::Recv {
                                value_dst: 6,
                                ok_dst: None,
                            },
                        },
                    ],
                    default_case,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn live_send_wins_over_closed_receive_program(default_case: Option<usize>) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 3,
                    cap: Some(1),
                    zero: 0,
                },
                Instruction::LoadInt { dst: 4, value: 11 },
                Instruction::CloseChannel { chan: 3 },
                Instruction::Select {
                    choice_dst: 5,
                    cases: vec![
                        SelectCaseOp {
                            chan: 2,
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
                    default_case,
                },
                Instruction::Compare {
                    dst: 7,
                    op: CompareOp::Equal,
                    left: 5,
                    right: 0,
                },
                Instruction::JumpIfFalse {
                    cond: 7,
                    target: 999,
                },
                Instruction::ChanRecv { dst: 6, chan: 2 },
                Instruction::Compare {
                    dst: 7,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 7,
                    target: 999,
                },
                Instruction::Return { src: None },
            ],
        }],
    }
}

#[test]
fn blocking_select_chooses_live_receive_before_closed_send() {
    let program = live_receive_wins_over_closed_send_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should choose live receive before closed send");
}

#[test]
fn default_select_chooses_live_receive_before_closed_send() {
    let program = live_receive_wins_over_closed_send_program(Some(2));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should choose live receive before closed send");
}

#[test]
fn blocking_select_errors_when_closed_send_precedes_live_receive() {
    let program = closed_send_before_live_receive_program(None);

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("blocking select should fail when closed send is encountered first");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn default_select_errors_when_closed_send_precedes_live_receive() {
    let program = closed_send_before_live_receive_program(Some(2));

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("default select should fail when closed send is encountered first");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn blocking_select_chooses_live_send_before_closed_receive() {
    let program = live_send_wins_over_closed_receive_program(None);

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("blocking select should choose live send before closed receive");
}

#[test]
fn default_select_chooses_live_send_before_closed_receive() {
    let program = live_send_wins_over_closed_receive_program(Some(2));

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("default select should choose live send before closed receive");
}
