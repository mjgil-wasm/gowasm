use super::{
    resolve_stdlib_function, Function, Instruction, MethodBinding, Program, TypeId, Vm, VmError,
};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn constructs_and_reads_struct_fields() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeStruct {
                    dst: 2,
                    typ: TypeId(100),
                    fields: vec![("x".into(), 0), ("y".into(), 1)],
                },
                Instruction::GetField {
                    dst: 3,
                    target: 2,
                    field: "y".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 {1 2}\n");
}

#[test]
fn rejects_missing_struct_fields() {
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
                Instruction::MakeStruct {
                    dst: 1,
                    typ: TypeId(100),
                    fields: vec![("x".into(), 0)],
                },
                Instruction::GetField {
                    dst: 0,
                    target: 1,
                    field: "y".into(),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    assert_eq!(
        error.root_cause(),
        &VmError::UnknownField {
            function: "main".into(),
            field: "y".into(),
        }
    );
}

#[test]
fn updates_existing_struct_fields() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeStruct {
                    dst: 2,
                    typ: TypeId(100),
                    fields: vec![("x".into(), 0), ("y".into(), 1)],
                },
                Instruction::LoadInt { dst: 3, value: 9 },
                Instruction::SetField {
                    target: 2,
                    field: "x".into(),
                    src: 3,
                },
                Instruction::GetField {
                    dst: 0,
                    target: 2,
                    field: "x".into(),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![0, 2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 {9 2}\n");
}

#[test]
fn dispatches_methods_by_receiver_type() {
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: TypeId(100),
            target_receiver_type: TypeId(100),
            name: "sum".into(),
            function: 1,
            param_types: Vec::new(),
            result_types: vec!["int".into()],
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 2 },
                    Instruction::LoadInt { dst: 1, value: 5 },
                    Instruction::MakeStruct {
                        dst: 2,
                        typ: TypeId(100),
                        fields: vec![("x".into(), 0), ("y".into(), 1)],
                    },
                    Instruction::CallMethod {
                        receiver: 2,
                        method: "sum".into(),
                        args: vec![],
                        dst: Some(3),
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![3],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "Point.sum".into(),
                param_count: 1,
                register_count: 4,
                code: vec![
                    Instruction::GetField {
                        dst: 1,
                        target: 0,
                        field: "x".into(),
                    },
                    Instruction::GetField {
                        dst: 2,
                        target: 0,
                        field: "y".into(),
                    },
                    Instruction::Add {
                        dst: 3,
                        left: 1,
                        right: 2,
                    },
                    Instruction::Return { src: Some(3) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}
