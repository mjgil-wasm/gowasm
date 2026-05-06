use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn unicode_to_title() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "ToTitle").expect("unicode.ToTitle should be registered")
}

fn unicode_to_upper() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "ToUpper").expect("unicode.ToUpper should be registered")
}

fn unicode_to_lower() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "ToLower").expect("unicode.ToLower should be registered")
}

fn unicode_to() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "To").expect("unicode.To should be registered")
}

fn unicode_simple_fold() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "SimpleFold")
        .expect("unicode.SimpleFold should be registered")
}

#[test]
fn executes_unicode_title_mapping() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 15,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 454 },
                Instruction::LoadInt { dst: 1, value: 452 },
                Instruction::LoadInt { dst: 2, value: 453 },
                Instruction::LoadInt { dst: 3, value: 97 },
                Instruction::LoadInt { dst: 4, value: 65 },
                Instruction::LoadInt { dst: 5, value: -1 },
                Instruction::LoadInt {
                    dst: 6,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![0],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![3],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![4],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![5],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_to_title(),
                    args: vec![6],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 8, 9, 10, 11, 12, 13],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "453 453 453 65 65 -1 1114112\n");
}

#[test]
fn executes_unicode_simple_upper_and_lower_mappings() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 17,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 223 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 7_838,
                },
                Instruction::LoadInt {
                    dst: 2,
                    value: 8_561,
                },
                Instruction::LoadInt {
                    dst: 3,
                    value: 8_546,
                },
                Instruction::LoadInt { dst: 4, value: 837 },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_to_lower(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_to_lower(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![4],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![5, 6, 7, 8, 9],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7838 223 8545 8562 921\n");
}

#[test]
fn executes_unicode_to_mapping() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 23,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 97 },
                Instruction::LoadInt { dst: 2, value: 1 },
                Instruction::LoadInt { dst: 3, value: 65 },
                Instruction::LoadInt { dst: 4, value: 2 },
                Instruction::LoadInt { dst: 5, value: 454 },
                Instruction::LoadInt { dst: 6, value: -1 },
                Instruction::LoadInt { dst: 7, value: 65 },
                Instruction::LoadInt { dst: 8, value: 0 },
                Instruction::LoadInt { dst: 9, value: -1 },
                Instruction::LoadInt { dst: 10, value: 1 },
                Instruction::LoadInt {
                    dst: 11,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![0, 1],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![2, 3],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![4, 5],
                    dst: Some(14),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![6, 7],
                    dst: Some(15),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![8, 9],
                    dst: Some(16),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
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
    assert_eq!(vm.stdout(), "65 97 453 65533 -1 1114112\n");
}

#[test]
fn executes_unicode_to_simple_case_mappings() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 20,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::LoadInt { dst: 1, value: 223 },
                Instruction::LoadInt { dst: 2, value: 1 },
                Instruction::LoadInt {
                    dst: 3,
                    value: 7_838,
                },
                Instruction::LoadInt { dst: 4, value: 0 },
                Instruction::LoadInt {
                    dst: 5,
                    value: 8_561,
                },
                Instruction::LoadInt { dst: 6, value: 1 },
                Instruction::LoadInt {
                    dst: 7,
                    value: 8_546,
                },
                Instruction::LoadInt { dst: 8, value: 0 },
                Instruction::LoadInt { dst: 9, value: 837 },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![0, 1],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![2, 3],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![4, 5],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![6, 7],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: unicode_to(),
                    args: vec![8, 9],
                    dst: Some(14),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![10, 11, 12, 13, 14],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7838 223 8545 8562 921\n");
}

#[test]
fn executes_unicode_simple_fold_cycle() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 15,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 65 },
                Instruction::LoadInt { dst: 1, value: 97 },
                Instruction::LoadInt { dst: 2, value: 75 },
                Instruction::LoadInt { dst: 3, value: 107 },
                Instruction::LoadInt {
                    dst: 4,
                    value: 8490,
                },
                Instruction::LoadInt { dst: 5, value: 49 },
                Instruction::LoadInt { dst: 6, value: -2 },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![0],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![3],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![4],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![5],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![6],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 8, 9, 10, 11, 12, 13],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "97 65 107 8490 75 49 -2\n");
}

#[test]
fn executes_unicode_non_ascii_simple_fold_cycles() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 15,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 931 },
                Instruction::LoadInt { dst: 1, value: 962 },
                Instruction::LoadInt { dst: 2, value: 963 },
                Instruction::LoadInt { dst: 3, value: 452 },
                Instruction::LoadInt { dst: 4, value: 453 },
                Instruction::LoadInt { dst: 5, value: 454 },
                Instruction::LoadInt { dst: 6, value: 223 },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![0],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![3],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![4],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![5],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_simple_fold(),
                    args: vec![6],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 8, 9, 10, 11, 12, 13],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "962 963 931 453 454 452 7838\n");
}
