use super::{resolve_stdlib_function, CompareOp, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn runs_a_program_with_fmt_println() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "hello".into(),
                },
                Instruction::LoadInt { dst: 1, value: 42 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![0, 1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 42\n");
}

#[test]
fn uses_explicit_frames_for_nested_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 0,
                code: vec![
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "helper".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "worker".into(),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\n");
}

#[test]
fn copies_arguments_into_parameter_registers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "hello".into(),
                    },
                    Instruction::LoadInt { dst: 1, value: 3 },
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![0, 1],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "echo".into(),
                param_count: 2,
                register_count: 2,
                code: vec![
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0, 1],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 3\n");
}

#[test]
fn moves_values_between_registers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "copied".into(),
                },
                Instruction::Move { dst: 1, src: 0 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "copied\n");
}

#[test]
fn adds_integer_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::LoadInt { dst: 1, value: 3 },
                Instruction::Add {
                    dst: 2,
                    left: 0,
                    right: 1,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn shifts_integer_values_left() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 3 },
                Instruction::ShiftLeft {
                    dst: 2,
                    left: 0,
                    right: 1,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8\n");
}

#[test]
fn shifts_integer_values_right() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 16 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::ShiftRight {
                    dst: 2,
                    left: 0,
                    right: 1,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4\n");
}

#[test]
fn concatenates_string_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "wasm".into(),
                },
                Instruction::Add {
                    dst: 2,
                    left: 0,
                    right: 1,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "gowasm\n");
}

#[test]
fn compares_values_and_formats_booleans() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 3 },
                Instruction::LoadInt { dst: 1, value: 4 },
                Instruction::Compare {
                    dst: 2,
                    op: CompareOp::Less,
                    left: 0,
                    right: 1,
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "go".into(),
                },
                Instruction::Compare {
                    dst: 5,
                    op: CompareOp::Equal,
                    left: 3,
                    right: 4,
                },
                Instruction::Compare {
                    dst: 6,
                    op: CompareOp::NotEqual,
                    left: 2,
                    right: 5,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 5, 6],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false\n");
}

#[test]
fn negates_boolean_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadBool {
                    dst: 0,
                    value: false,
                },
                Instruction::Not { dst: 1, src: 0 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn runs_integer_arithmetic_and_negation() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 10 },
                Instruction::LoadInt { dst: 1, value: 3 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::Multiply {
                    dst: 3,
                    left: 1,
                    right: 2,
                },
                Instruction::Subtract {
                    dst: 4,
                    left: 0,
                    right: 3,
                },
                Instruction::Negate { dst: 5, src: 4 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![5],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-4\n");
}

#[test]
fn jumps_over_false_if_bodies() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadBool {
                    dst: 0,
                    value: false,
                },
                Instruction::JumpIfFalse { cond: 0, target: 4 },
                Instruction::LoadString {
                    dst: 1,
                    value: "then".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "after".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "after\n");
}

#[test]
fn jumps_to_the_end_of_true_if_else_bodies() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadBool {
                    dst: 0,
                    value: true,
                },
                Instruction::JumpIfFalse { cond: 0, target: 5 },
                Instruction::LoadString {
                    dst: 1,
                    value: "then".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::Jump { target: 7 },
                Instruction::LoadString {
                    dst: 1,
                    value: "else".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "then\n");
}

#[test]
fn returns_values_to_the_caller_frame() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![],
                        dst: Some(0),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "answer".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 42 },
                    Instruction::Return { src: Some(0) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn schedules_spawned_goroutines_with_gocall() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "main-start".into(),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0],
                        dst: None,
                    },
                    Instruction::LoadString {
                        dst: 1,
                        value: "worker".into(),
                    },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![1],
                    },
                    Instruction::LoadString {
                        dst: 0,
                        value: "main-end".into(),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "worker".into(),
                param_count: 1,
                register_count: 1,
                code: vec![
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "main-start\nworker\nmain-end\n");
}

#[test]
fn nil_channel_ops_deadlock_when_no_goroutine_can_make_progress() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadNilChannel {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::ChanRecv { dst: 1, chan: 0 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil channel should deadlock");
    assert!(matches!(error, super::VmError::Deadlock));
}

#[test]
fn constructs_and_indexes_slice_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 8 },
                Instruction::LoadInt { dst: 1, value: 9 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::Index {
                    dst: 4,
                    target: 2,
                    index: 3,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn formats_array_values_with_fmt_println() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeArray {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2]\n");
}

#[test]
fn indexes_string_values_as_bytes() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::LoadInt { dst: 2, value: 1 },
                Instruction::Index {
                    dst: 3,
                    target: 0,
                    index: 1,
                },
                Instruction::Index {
                    dst: 4,
                    target: 0,
                    index: 2,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "103 111\n");
}
