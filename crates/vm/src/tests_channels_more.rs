use super::{CompareOp, Function, Instruction, Program, Vm};

#[test]
fn try_send_reports_not_ready_without_blocking() {
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
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::ChanTrySend {
                    ready_dst: 3,
                    chan: 1,
                    value: 2,
                },
                Instruction::LoadBool {
                    dst: 4,
                    value: false,
                },
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
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("try send should not block on unready channel");
}

#[test]
fn try_send_buffers_ready_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 2,
                    cap: Some(0),
                    zero: 1,
                },
                Instruction::LoadInt { dst: 3, value: 7 },
                Instruction::ChanTrySend {
                    ready_dst: 4,
                    chan: 2,
                    value: 3,
                },
                Instruction::LoadBool {
                    dst: 5,
                    value: true,
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
                Instruction::ChanRecv { dst: 6, chan: 2 },
                Instruction::Compare {
                    dst: 4,
                    op: CompareOp::Equal,
                    left: 6,
                    right: 3,
                },
                Instruction::JumpIfFalse {
                    cond: 4,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("ready try send should buffer values");
}

#[test]
fn close_wakes_blocked_receiver_with_zero_value() {
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
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake blocked receivers");
}

#[test]
fn close_wakes_blocked_sender_with_error() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
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
                    Instruction::LoadInt { dst: 2, value: 7 },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![1, 2],
                    },
                    Instruction::CloseChannel { chan: 1 },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "sender".into(),
                param_count: 2,
                register_count: 2,
                code: vec![
                    Instruction::ChanSend { chan: 0, value: 1 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail blocked senders");
    assert!(matches!(
        error.root_cause(),
        super::VmError::SendOnClosedChannel { .. }
    ));
}

#[test]
fn closed_channel_receive_ok_reports_false() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::ChanRecvOk {
                    value_dst: 2,
                    ok_dst: 3,
                    chan: 1,
                },
                Instruction::LoadBool {
                    dst: 4,
                    value: false,
                },
                Instruction::Compare {
                    dst: 4,
                    op: CompareOp::Equal,
                    left: 3,
                    right: 4,
                },
                Instruction::JumpIfFalse {
                    cond: 4,
                    target: 99,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("comma-ok receive on closed channel should report false");
}
