use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_equal_fold() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "EqualFold").expect("strings.EqualFold should be registered")
}

#[test]
fn executes_strings_equal_fold_queries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 18,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "Go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "Σ".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "ς".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "ǅ".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "ǆ".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "Straße".into(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "straße".into(),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "Go".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "ga".into(),
                },
                Instruction::LoadString {
                    dst: 10,
                    value: "ß".into(),
                },
                Instruction::LoadString {
                    dst: 11,
                    value: "ss".into(),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![0, 1],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![2, 3],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![4, 5],
                    dst: Some(14),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![6, 7],
                    dst: Some(15),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![8, 9],
                    dst: Some(16),
                },
                Instruction::CallStdlib {
                    function: strings_equal_fold(),
                    args: vec![10, 11],
                    dst: Some(17),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![12, 13, 14, 15, 16, 17],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true true false false\n");
}
