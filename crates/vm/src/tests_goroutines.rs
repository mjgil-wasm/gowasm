use super::{resolve_stdlib_function, Function, Instruction, MethodBinding, Program, TypeId, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn schedules_spawned_goroutines_with_gocallclosure() {
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
                    Instruction::LoadString {
                        dst: 0,
                        value: "worker".into(),
                    },
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 1,
                        function: 1,
                        captures: vec![],
                    },
                    Instruction::GoCallClosure {
                        callee: 1,
                        args: vec![0],
                    },
                    Instruction::LoadString {
                        dst: 2,
                        value: "main".into(),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![2],
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
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn schedules_spawned_goroutines_with_gocallmethod() {
    let runner_type = TypeId(100);
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: runner_type,
            target_receiver_type: runner_type,
            name: "Run".into(),
            function: 1,
            param_types: Vec::new(),
            result_types: Vec::new(),
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::MakeStruct {
                        dst: 0,
                        typ: runner_type,
                        fields: vec![],
                    },
                    Instruction::GoCallMethod {
                        receiver: 0,
                        method: "Run".into(),
                        args: vec![],
                    },
                    Instruction::LoadString {
                        dst: 1,
                        value: "main".into(),
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
                name: "worker".into(),
                param_count: 1,
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
    assert_eq!(vm.stdout(), "worker\nmain\n");
}
