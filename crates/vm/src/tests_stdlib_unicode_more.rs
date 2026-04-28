use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn unicode_is_letter() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsLetter").expect("unicode.IsLetter should be registered")
}

fn unicode_is_mark() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsMark").expect("unicode.IsMark should be registered")
}

fn unicode_is_space() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsSpace").expect("unicode.IsSpace should be registered")
}

fn unicode_is_control() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsControl").expect("unicode.IsControl should be registered")
}

fn unicode_is_number() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsNumber").expect("unicode.IsNumber should be registered")
}

#[test]
fn executes_unicode_letter_category_boundaries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 11,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 65 },
                Instruction::LoadInt { dst: 1, value: 688 },
                Instruction::LoadInt { dst: 2, value: 837 },
                Instruction::LoadInt {
                    dst: 3,
                    value: 8_544,
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7, 8],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false true\n");
}

#[test]
fn executes_unicode_upper_and_lower_category_boundaries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 13,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 923 },
                Instruction::LoadInt { dst: 1, value: 955 },
                Instruction::LoadInt {
                    dst: 2,
                    value: 8_544,
                },
                Instruction::LoadInt { dst: 3, value: 837 },
                Instruction::CallStdlib {
                    function: super::resolve_stdlib_function("unicode", "IsUpper")
                        .expect("unicode.IsUpper should be registered"),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: super::resolve_stdlib_function("unicode", "IsLower")
                        .expect("unicode.IsLower should be registered"),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: super::resolve_stdlib_function("unicode", "IsUpper")
                        .expect("unicode.IsUpper should be registered"),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: super::resolve_stdlib_function("unicode", "IsLower")
                        .expect("unicode.IsLower should be registered"),
                    args: vec![3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: super::resolve_stdlib_function("unicode", "IsNumber")
                        .expect("unicode.IsNumber should be registered"),
                    args: vec![2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![3],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7, 8, 9],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false true true\n");
}

#[test]
fn executes_unicode_space_and_control_boundaries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 16,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 160 },
                Instruction::LoadInt { dst: 1, value: 133 },
                Instruction::LoadInt {
                    dst: 2,
                    value: 5_760,
                },
                Instruction::LoadInt {
                    dst: 3,
                    value: 8_203,
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![0],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![3],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7, 8, 9, 10],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false true false false\n");
}

#[test]
fn executes_unicode_non_latin_white_space_boundaries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 14,
            code: vec![
                Instruction::LoadInt {
                    dst: 0,
                    value: 8_232,
                },
                Instruction::LoadInt {
                    dst: 1,
                    value: 8_233,
                },
                Instruction::LoadInt {
                    dst: 2,
                    value: 8_203,
                },
                Instruction::LoadInt {
                    dst: 3,
                    value: 8_288,
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false\n");
}

#[test]
fn executes_unicode_number_category_boundaries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 13,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 189 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 8_544,
                },
                Instruction::LoadInt {
                    dst: 2,
                    value: 12_295,
                },
                Instruction::LoadInt { dst: 3, value: 65 },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7, 8],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false false\n");
}
