use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_last_index() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "LastIndex").expect("strings.LastIndex should be registered")
}

fn strings_trim_left() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "TrimLeft").expect("strings.TrimLeft should be registered")
}

fn strings_trim_right() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "TrimRight").expect("strings.TrimRight should be registered")
}

fn strings_trim() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Trim").expect("strings.Trim should be registered")
}

fn strings_contains_any() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "ContainsAny")
        .expect("strings.ContainsAny should be registered")
}

fn strings_index_any() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "IndexAny").expect("strings.IndexAny should be registered")
}

fn strings_last_index_any() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "LastIndexAny")
        .expect("strings.LastIndexAny should be registered")
}

fn strconv_can_backquote() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "CanBackquote")
        .expect("strconv.CanBackquote should be registered")
}

fn strconv_format_int() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "FormatInt").expect("strconv.FormatInt should be registered")
}

fn strconv_atoi() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "Atoi").expect("strconv.Atoi should be registered")
}

fn strconv_parse_bool() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "ParseBool").expect("strconv.ParseBool should be registered")
}

fn strconv_parse_int() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "ParseInt").expect("strconv.ParseInt should be registered")
}

fn strconv_unquote() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "Unquote").expect("strconv.Unquote should be registered")
}

fn errors_new() -> super::StdlibFunctionId {
    resolve_stdlib_function("errors", "New").expect("errors.New should be registered")
}

#[test]
fn executes_strings_last_index() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go wasm go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zig".into(),
                },
                Instruction::CallStdlib {
                    function: strings_last_index(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_last_index(),
                    args: vec![0, 2],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8 -1\n");
}

#[test]
fn executes_errors_new() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "bad".into(),
                },
                Instruction::CallStdlib {
                    function: errors_new(),
                    args: vec![0],
                    dst: Some(1),
                },
                Instruction::LoadErrorMessage { dst: 2, src: 1 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1, 2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "bad bad\n");
}

#[test]
fn executes_strconv_parse_int() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 13,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "0xff".into(),
                },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::LoadInt { dst: 2, value: 16 },
                Instruction::CallStdlibMulti {
                    function: strconv_parse_int(),
                    args: vec![0, 1, 2],
                    dsts: vec![3, 4],
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "128".into(),
                },
                Instruction::LoadInt { dst: 6, value: 10 },
                Instruction::LoadInt { dst: 7, value: 8 },
                Instruction::CallStdlibMulti {
                    function: strconv_parse_int(),
                    args: vec![5, 6, 7],
                    dsts: vec![8, 9],
                },
                Instruction::LoadErrorMessage { dst: 10, src: 9 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4, 8, 10],
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
        "255 <nil> 0 strconv.ParseInt: parsing \"128\": value out of range\n"
    );
}

#[test]
fn executes_strconv_unquote() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 10,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "\"go\"".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![0],
                    dsts: vec![1, 2],
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "go".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![3],
                    dsts: vec![4, 5],
                },
                Instruction::LoadErrorMessage { dst: 6, src: 5 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1, 2, 4, 6],
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
        "go <nil>  strconv.Unquote: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn executes_strconv_unquote_hex_and_unicode_escapes() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 16,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "\"\\x41\"".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![0],
                    dsts: vec![1, 2],
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "\"\\u03bb\"".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![3],
                    dsts: vec![4, 5],
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "\"\\U0001f642\"".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![6],
                    dsts: vec![7, 8],
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "\"\\u00zz\"".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote(),
                    args: vec![9],
                    dsts: vec![10, 11],
                },
                Instruction::LoadErrorMessage { dst: 12, src: 11 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1, 2, 4, 5, 7, 8, 10, 12],
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
        "A <nil> λ <nil> 🙂 <nil>  strconv.Unquote: parsing \"\\\"\\\\u00zz\\\"\": invalid syntax\n"
    );
}

#[test]
fn executes_strings_trim_left() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "..go..".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ".".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim_left(),
                    args: vec![0, 1],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go..\n");
}

#[test]
fn executes_strings_trim_right() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "..go..".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ".".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim_right(),
                    args: vec![0, 1],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "..go\n");
}

#[test]
fn executes_strings_trim() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "..go..".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ".".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim(),
                    args: vec![0, 1],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go\n");
}

#[test]
fn executes_strconv_atoi() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "42".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_atoi(),
                    args: vec![0],
                    dsts: vec![2, 3],
                },
                Instruction::CallStdlibMulti {
                    function: strconv_atoi(),
                    args: vec![1],
                    dsts: vec![4, 5],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3, 4, 5],
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
        "42 <nil> 0 strconv.Atoi: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn executes_strconv_parse_bool() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "TRUE".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_parse_bool(),
                    args: vec![0],
                    dsts: vec![2, 3],
                },
                Instruction::CallStdlibMulti {
                    function: strconv_parse_bool(),
                    args: vec![1],
                    dsts: vec![4, 5],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3, 4, 5],
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
        "true <nil> false strconv.ParseBool: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn executes_strings_contains_any() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "mx".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zx".into(),
                },
                Instruction::CallStdlib {
                    function: strings_contains_any(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_contains_any(),
                    args: vec![0, 2],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn executes_strings_index_any() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "mx".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zx".into(),
                },
                Instruction::CallStdlib {
                    function: strings_index_any(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_index_any(),
                    args: vec![0, 2],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 -1\n");
}

#[test]
fn executes_strings_last_index_any() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "ma".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zx".into(),
                },
                Instruction::CallStdlib {
                    function: strings_last_index_any(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_last_index_any(),
                    args: vec![0, 2],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 -1\n");
}

#[test]
fn executes_strconv_can_backquote() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go\twasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go\nwasm".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "`go`".into(),
                },
                Instruction::CallStdlib {
                    function: strconv_can_backquote(),
                    args: vec![0],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strconv_can_backquote(),
                    args: vec![1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: strconv_can_backquote(),
                    args: vec![2],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4, 5],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false false\n");
}

#[test]
fn executes_strconv_format_int() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 255 },
                Instruction::LoadInt { dst: 1, value: 16 },
                Instruction::LoadInt { dst: 2, value: -5 },
                Instruction::LoadInt { dst: 3, value: 2 },
                Instruction::CallStdlib {
                    function: strconv_format_int(),
                    args: vec![0, 1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: strconv_format_int(),
                    args: vec![2, 3],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ff -101\n");
}

#[test]
fn rejects_strconv_format_int_with_illegal_base() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 10 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::CallStdlib {
                    function: strconv_format_int(),
                    args: vec![0, 1],
                    dst: Some(2),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("strconv.FormatInt should reject illegal bases");
    assert!(error
        .to_string()
        .contains("`strconv.FormatInt` base 1 must be between 2 and 36"));
}
