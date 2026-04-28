use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_cut_prefix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "CutPrefix").expect("strings.CutPrefix should be registered")
}

fn strings_cut_suffix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "CutSuffix").expect("strings.CutSuffix should be registered")
}

fn strings_cut() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Cut").expect("strings.Cut should be registered")
}

fn strconv_unquote_char() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "UnquoteChar")
        .expect("strconv.UnquoteChar should be registered")
}

#[test]
fn executes_multi_result_function_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::CallFunctionMulti {
                        function: 1,
                        args: vec![],
                        dsts: vec![0, 1],
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0, 1],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "pair".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 7 },
                    Instruction::LoadString {
                        dst: 1,
                        value: "go".into(),
                    },
                    Instruction::ReturnMulti { srcs: vec![0, 1] },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go\n");
}

#[test]
fn executes_multi_result_stdlib_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strings_cut_prefix(),
                    args: vec![0, 1],
                    dsts: vec![2, 3],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "wasm true\n");
}

#[test]
fn executes_cut_suffix_multi_result_stdlib_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "wasm".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strings_cut_suffix(),
                    args: vec![0, 1],
                    dsts: vec![2, 3],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go true\n");
}

#[test]
fn executes_cut_triple_result_stdlib_calls() {
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
                    value: "go:wasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ":".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strings_cut(),
                    args: vec![0, 1],
                    dsts: vec![2, 3, 4],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go wasm true\n");
}

#[test]
fn executes_quad_result_function_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 5,
                code: vec![
                    Instruction::CallFunctionMulti {
                        function: 1,
                        args: vec![],
                        dsts: vec![0, 1, 2, 3],
                    },
                    Instruction::CallStdlib {
                        function: fmt_println(),
                        args: vec![0, 1, 2, 3],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "quad".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 7 },
                    Instruction::LoadString {
                        dst: 1,
                        value: "go".into(),
                    },
                    Instruction::LoadBool {
                        dst: 2,
                        value: true,
                    },
                    Instruction::LoadInt { dst: 3, value: 9 },
                    Instruction::ReturnMulti {
                        srcs: vec![0, 1, 2, 3],
                    },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go true 9\n");
}

#[test]
fn executes_quad_result_stdlib_calls() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "\\nrest".into(),
                },
                Instruction::LoadInt { dst: 1, value: 34 },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote_char(),
                    args: vec![0, 1],
                    dsts: vec![2, 3, 4, 5],
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
    assert_eq!(vm.stdout(), "10 false rest <nil>\n");
}

#[test]
fn executes_unquote_char_hex_and_unicode_escapes() {
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
                    value: "\\x41rest".into(),
                },
                Instruction::LoadInt { dst: 1, value: 34 },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote_char(),
                    args: vec![0, 1],
                    dsts: vec![2, 3, 4, 5],
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "\\u03bbx".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote_char(),
                    args: vec![6, 1],
                    dsts: vec![7, 8, 9, 10],
                },
                Instruction::LoadString {
                    dst: 11,
                    value: "\\U0001f642!".into(),
                },
                Instruction::CallStdlibMulti {
                    function: strconv_unquote_char(),
                    args: vec![11, 1],
                    dsts: vec![12, 13, 14, 15],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![2, 3, 4, 7, 8, 9, 12, 13, 14],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65 false rest 955 true x 128578 true !\n");
}
