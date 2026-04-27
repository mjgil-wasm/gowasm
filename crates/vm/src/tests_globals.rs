use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn loads_and_stores_global_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 9 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::LoadGlobal { dst: 1, global: 0 },
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
    assert_eq!(vm.stdout(), "9\n");
}
