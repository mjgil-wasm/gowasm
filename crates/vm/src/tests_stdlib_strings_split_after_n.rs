use super::{resolve_stdlib_function, CompareOp, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_split_after_n() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "SplitAfterN")
        .expect("strings.SplitAfterN should be registered")
}

#[test]
fn executes_strings_split_after_n_queries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 17,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go,wasm,zig".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ",".into(),
                },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::LoadInt { dst: 3, value: -1 },
                Instruction::LoadInt { dst: 4, value: 0 },
                Instruction::LoadString {
                    dst: 5,
                    value: "abc".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "".into(),
                },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::CallStdlib {
                    function: strings_split_after_n(),
                    args: vec![0, 1, 2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: strings_split_after_n(),
                    args: vec![0, 1, 3],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: strings_split_after_n(),
                    args: vec![0, 1, 4],
                    dst: Some(10),
                },
                Instruction::IsNil { dst: 11, src: 10 },
                Instruction::CallStdlib {
                    function: strings_split_after_n(),
                    args: vec![5, 6, 2],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: strings_split_after_n(),
                    args: vec![5, 6, 7],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![8, 9, 11, 12, 13],
                    dst: None,
                },
                Instruction::LoadNilSlice {
                    dst: 14,
                    concrete_type: None,
                },
                Instruction::Compare {
                    dst: 15,
                    left: 10,
                    right: 14,
                    op: CompareOp::Equal,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![15],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[go, wasm,zig] [go, wasm, zig] true [a bc] [abc]\ntrue\n"
    );
}
