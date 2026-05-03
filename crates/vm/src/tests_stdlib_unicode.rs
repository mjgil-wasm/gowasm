use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn unicode_is_digit() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsDigit").expect("unicode.IsDigit should be registered")
}

fn unicode_is_letter() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsLetter").expect("unicode.IsLetter should be registered")
}

fn unicode_is_space() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsSpace").expect("unicode.IsSpace should be registered")
}

fn unicode_is_upper() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsUpper").expect("unicode.IsUpper should be registered")
}

fn unicode_is_lower() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsLower").expect("unicode.IsLower should be registered")
}

fn unicode_is_number() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsNumber").expect("unicode.IsNumber should be registered")
}

fn unicode_is_print() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsPrint").expect("unicode.IsPrint should be registered")
}

fn unicode_is_graphic() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsGraphic").expect("unicode.IsGraphic should be registered")
}

fn unicode_is_punct() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsPunct").expect("unicode.IsPunct should be registered")
}

fn unicode_is_symbol() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsSymbol").expect("unicode.IsSymbol should be registered")
}

fn unicode_is_mark() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsMark").expect("unicode.IsMark should be registered")
}

fn unicode_is_control() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsControl").expect("unicode.IsControl should be registered")
}

fn unicode_is_title() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "IsTitle").expect("unicode.IsTitle should be registered")
}

fn unicode_to_upper() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "ToUpper").expect("unicode.ToUpper should be registered")
}

fn unicode_to_lower() -> super::StdlibFunctionId {
    resolve_stdlib_function("unicode", "ToLower").expect("unicode.ToLower should be registered")
}

#[test]
fn executes_unicode_predicates() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 15,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 55 },
                Instruction::LoadInt { dst: 1, value: 65 },
                Instruction::LoadInt { dst: 2, value: 955 },
                Instruction::LoadInt { dst: 3, value: 10 },
                Instruction::LoadInt { dst: 4, value: 71 },
                Instruction::LoadInt { dst: 5, value: 103 },
                Instruction::LoadInt { dst: 6, value: -1 },
                Instruction::LoadInt {
                    dst: 7,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: unicode_is_digit(),
                    args: vec![0],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_digit(),
                    args: vec![1],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![2],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![3],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_is_upper(),
                    args: vec![4],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_is_lower(),
                    args: vec![5],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![6],
                    dst: Some(14),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![7],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![8, 9, 10, 11, 12, 13, 14, 7],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true true true true false false\n");
}

#[test]
fn executes_unicode_case_mappings() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 12,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 97 },
                Instruction::LoadInt { dst: 1, value: 955 },
                Instruction::LoadInt { dst: 2, value: 65 },
                Instruction::LoadInt { dst: 3, value: 923 },
                Instruction::LoadInt { dst: 4, value: -1 },
                Instruction::LoadInt {
                    dst: 5,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![0],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![1],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_to_lower(),
                    args: vec![2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_to_lower(),
                    args: vec![3],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_to_upper(),
                    args: vec![4],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_to_lower(),
                    args: vec![5],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![6, 7, 8, 9, 10, 11],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65 923 97 955 -1 1114112\n");
}

#[test]
fn executes_unicode_number_and_print_predicates() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 12,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 189 },
                Instruction::LoadInt { dst: 1, value: 65 },
                Instruction::LoadInt { dst: 2, value: 955 },
                Instruction::LoadInt { dst: 3, value: 10 },
                Instruction::LoadInt { dst: 4, value: -1 },
                Instruction::LoadInt {
                    dst: 5,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![0],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![1],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![2],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![3],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![4],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![5],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![6, 7, 8, 9, 10, 11],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true false false false\n");
}

#[test]
fn executes_unicode_graphic_and_print_predicates() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 14,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 32 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 12_288,
                },
                Instruction::LoadInt { dst: 2, value: 769 },
                Instruction::LoadInt {
                    dst: 3,
                    value: 8205,
                },
                Instruction::LoadInt { dst: 4, value: -1 },
                Instruction::CallStdlib {
                    function: unicode_is_graphic(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![0],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_graphic(),
                    args: vec![1],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_graphic(),
                    args: vec![2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![2],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_is_graphic(),
                    args: vec![3],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![4],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![3],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![5, 6, 7, 8, 9, 10, 11, 12, 13],
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
        "true true true false true true false false false\n"
    );
}

#[test]
fn executes_unicode_punctuation_predicate() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 33 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 8212,
                },
                Instruction::LoadInt { dst: 2, value: 95 },
                Instruction::LoadInt { dst: 3, value: 36 },
                Instruction::LoadInt { dst: 4, value: 65 },
                Instruction::CallStdlib {
                    function: unicode_is_punct(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_punct(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_punct(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_punct(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_punct(),
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
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn executes_unicode_symbol_predicate() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 36 },
                Instruction::LoadInt { dst: 1, value: 43 },
                Instruction::LoadInt { dst: 2, value: 169 },
                Instruction::LoadInt { dst: 3, value: 65 },
                Instruction::LoadInt { dst: 4, value: 95 },
                Instruction::CallStdlib {
                    function: unicode_is_symbol(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_symbol(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_symbol(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_symbol(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_symbol(),
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
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn executes_unicode_mark_predicate() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 769 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 2307,
                },
                Instruction::LoadInt {
                    dst: 2,
                    value: 8413,
                },
                Instruction::LoadInt { dst: 3, value: 65 },
                Instruction::LoadInt { dst: 4, value: 36 },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_mark(),
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
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn executes_unicode_control_predicate() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 10 },
                Instruction::LoadInt { dst: 1, value: 127 },
                Instruction::LoadInt { dst: 2, value: 32 },
                Instruction::LoadInt { dst: 3, value: 955 },
                Instruction::LoadInt { dst: 4, value: -1 },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
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
    assert_eq!(vm.stdout(), "true true false false false\n");
}

#[test]
fn executes_unicode_title_predicate() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 453 },
                Instruction::LoadInt { dst: 1, value: 452 },
                Instruction::LoadInt { dst: 2, value: 454 },
                Instruction::LoadInt { dst: 3, value: 65 },
                Instruction::LoadInt { dst: 4, value: 97 },
                Instruction::CallStdlib {
                    function: unicode_is_title(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: unicode_is_title(),
                    args: vec![1],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_title(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_title(),
                    args: vec![3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_title(),
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
    assert_eq!(vm.stdout(), "true false false false false\n");
}

#[test]
fn executes_unicode_non_ascii_predicates() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 16,
            code: vec![
                Instruction::LoadInt {
                    dst: 0,
                    value: 1637,
                },
                Instruction::LoadInt {
                    dst: 1,
                    value: 8544,
                },
                Instruction::LoadInt {
                    dst: 2,
                    value: 12_288,
                },
                Instruction::LoadInt {
                    dst: 3,
                    value: 1046,
                },
                Instruction::LoadInt {
                    dst: 4,
                    value: 1078,
                },
                Instruction::LoadInt { dst: 5, value: 133 },
                Instruction::CallStdlib {
                    function: unicode_is_digit(),
                    args: vec![0],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: unicode_is_digit(),
                    args: vec![1],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: unicode_is_number(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: unicode_is_space(),
                    args: vec![2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: unicode_is_upper(),
                    args: vec![3],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: unicode_is_lower(),
                    args: vec![4],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: unicode_is_control(),
                    args: vec![5],
                    dst: Some(12),
                },
                Instruction::CallStdlib {
                    function: unicode_is_graphic(),
                    args: vec![5],
                    dst: Some(13),
                },
                Instruction::CallStdlib {
                    function: unicode_is_letter(),
                    args: vec![3],
                    dst: Some(14),
                },
                Instruction::CallStdlib {
                    function: unicode_is_print(),
                    args: vec![2],
                    dst: Some(15),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
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
        "true false true true true true true false true false\n"
    );
}

#[test]
fn rejects_non_int_arguments_for_unicode_helpers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "7".into(),
                },
                Instruction::CallStdlib {
                    function: unicode_is_digit(),
                    args: vec![0],
                    dst: Some(1),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("unicode.IsDigit should reject strings");
    assert!(error
        .to_string()
        .contains("`unicode.IsDigit` expects an int argument"));
}
