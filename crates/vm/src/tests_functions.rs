use super::{resolve_stdlib_function, Function, Instruction, Program, Value, Vm, TYPE_POINTER};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn calls_function_values() {
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
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 0,
                        function: 1,
                        captures: vec![],
                    },
                    Instruction::CallClosure {
                        callee: 0,
                        args: vec![],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "greet".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "hello".into(),
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
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn rejects_non_function_call_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::CallClosure {
                    callee: 0,
                    args: vec![],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    assert!(matches!(
        error.root_cause(),
        super::VmError::InvalidFunctionValue { .. }
    ));
}

#[test]
fn formats_function_values() {
    assert_eq!(
        super::format_value(&Value::function(1, Vec::new())),
        "<func>"
    );
}

#[test]
fn executes_deferred_function_values() {
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
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 0,
                        function: 1,
                        captures: vec![],
                    },
                    Instruction::DeferClosure {
                        callee: 0,
                        args: vec![],
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "later".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadString {
                        dst: 0,
                        value: "later".into(),
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
    assert_eq!(vm.stdout(), "later\n");
}

#[test]
fn calls_closures_with_heap_boxed_captures_after_return() {
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
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![],
                        dst: Some(0),
                    },
                    Instruction::CallClosure {
                        callee: 0,
                        args: vec![],
                        dst: Some(1),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![1],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "make".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 41 },
                    Instruction::BoxHeap {
                        dst: 1,
                        src: 0,
                        typ: TYPE_POINTER,
                    },
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 2,
                        function: 2,
                        captures: vec![1],
                    },
                    Instruction::Return { src: Some(2) },
                ],
            },
            Function {
                name: "next".into(),
                param_count: 1,
                register_count: 4,
                code: vec![
                    Instruction::Deref { dst: 1, src: 0 },
                    Instruction::LoadInt { dst: 2, value: 1 },
                    Instruction::Add {
                        dst: 1,
                        left: 1,
                        right: 2,
                    },
                    Instruction::StoreIndirect { target: 0, src: 1 },
                    Instruction::Return { src: Some(1) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn compares_non_nil_function_values_against_nil() {
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
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 0,
                        function: 1,
                        captures: vec![],
                    },
                    Instruction::LoadNil { dst: 1 },
                    Instruction::Compare {
                        dst: 2,
                        op: super::CompareOp::Equal,
                        left: 0,
                        right: 1,
                    },
                    Instruction::Compare {
                        dst: 3,
                        op: super::CompareOp::NotEqual,
                        left: 0,
                        right: 1,
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![2, 3],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "run".into(),
                param_count: 0,
                register_count: 0,
                code: vec![Instruction::Return { src: None }],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false true\n");
}
