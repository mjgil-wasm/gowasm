use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn copy_mutates_overlapping_subslices_through_shared_backing() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::LoadInt { dst: 2, value: 3 },
                Instruction::LoadInt { dst: 3, value: 4 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 4,
                    items: vec![0, 1, 2, 3],
                },
                Instruction::LoadInt { dst: 5, value: 1 },
                Instruction::Slice {
                    dst: 6,
                    target: 4,
                    low: Some(5),
                    high: None,
                },
                Instruction::Copy {
                    target: 6,
                    src: 4,
                    count_dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 4, 6],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 [1 1 2 3] [1 2 3]\n");
}
