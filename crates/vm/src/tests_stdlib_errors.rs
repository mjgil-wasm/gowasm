use super::{resolve_stdlib_function, Function, Instruction, Program, Vm, VmError};

fn errors_join() -> super::StdlibFunctionId {
    resolve_stdlib_function("errors", "Join").expect("errors.Join should be registered")
}

fn errors_new() -> super::StdlibFunctionId {
    resolve_stdlib_function("errors", "New").expect("errors.New should be registered")
}

fn errors_as() -> super::StdlibFunctionId {
    resolve_stdlib_function("errors", "As").expect("errors.As should be registered")
}

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn executes_errors_join_with_nil_elision() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadNil { dst: 0 },
                Instruction::LoadString {
                    dst: 1,
                    value: "first".into(),
                },
                Instruction::CallStdlib {
                    function: errors_new(),
                    args: vec![1],
                    dst: Some(2),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "second".into(),
                },
                Instruction::CallStdlib {
                    function: errors_new(),
                    args: vec![3],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: errors_join(),
                    args: vec![],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: errors_join(),
                    args: vec![0],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: errors_join(),
                    args: vec![2, 0, 4],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![5, 6],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil> <nil>\nfirst\nsecond\n");
}

#[test]
fn rejects_non_error_values_in_errors_join() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 5 },
                Instruction::CallStdlib {
                    function: errors_join(),
                    args: vec![0],
                    dst: Some(1),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    assert_eq!(
        error.root_cause(),
        &VmError::InvalidErrorValue {
            function: "main".into(),
        }
    );
}

#[test]
fn errors_as_is_registered() {
    let function = errors_as();
    assert_eq!(function.0, 509);
}
