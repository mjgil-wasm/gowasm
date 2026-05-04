use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn runs_deferred_stdlib_calls_in_lifo_order() {
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
                    value: "first".into(),
                },
                Instruction::DeferStdlib {
                    function: fmt_println(),
                    args: vec![0],
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "second".into(),
                },
                Instruction::DeferStdlib {
                    function: fmt_println(),
                    args: vec![1],
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "second\nfirst\n");
}

#[test]
fn captures_deferred_function_arguments_at_defer_time() {
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
                    Instruction::LoadInt { dst: 0, value: 1 },
                    Instruction::DeferFunction {
                        function: 1,
                        args: vec![0],
                    },
                    Instruction::LoadInt { dst: 0, value: 9 },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "show".into(),
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
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn runs_deferred_calls_before_unhandled_panic() {
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
                    value: "cleanup".into(),
                },
                Instruction::DeferStdlib {
                    function: fmt_println(),
                    args: vec![0],
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "boom".into(),
                },
                Instruction::Panic { src: 1 },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert_eq!(vm.stdout(), "cleanup\n");
    assert!(matches!(
        error.root_cause(),
        super::VmError::UnhandledPanic { .. }
    ));
    let text = error.to_string();
    assert!(text.contains("panic in function `main`: boom"));
    assert!(text.contains("stack trace:"));
    assert!(text.contains("at main"));
}

#[test]
fn unhandled_panics_render_leaf_first_stack_traces() {
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
                name: "explode".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "boom".into(),
                    },
                    Instruction::Panic { src: 0 },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(matches!(
        error.root_cause(),
        super::VmError::UnhandledPanic { .. }
    ));
    let text = error.to_string();
    let explode = text
        .find("at explode")
        .expect("stack trace should include the panicking frame");
    let main = text
        .find("at main")
        .expect("stack trace should include the caller frame");
    assert!(explode < main, "stack trace should be leaf-first: {text}");
}

#[test]
fn recover_clears_a_panicking_caller_from_a_deferred_frame() {
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
                    Instruction::DeferFunction {
                        function: 1,
                        args: vec![],
                    },
                    Instruction::LoadString {
                        dst: 1,
                        value: "boom".into(),
                    },
                    Instruction::Panic { src: 1 },
                ],
            },
            Function {
                name: "cleanup".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::Recover { dst: 0 },
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
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "boom\n");
}

#[test]
fn recover_returns_named_results_from_implicit_return_registers() {
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
                name: "demo".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "before".into(),
                    },
                    Instruction::DeferFunction {
                        function: 2,
                        args: vec![],
                    },
                    Instruction::LoadString {
                        dst: 1,
                        value: "boom".into(),
                    },
                    Instruction::Panic { src: 1 },
                    Instruction::Return { src: Some(0) },
                ],
            },
            Function {
                name: "cleanup".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::Recover { dst: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "before\n");
}
