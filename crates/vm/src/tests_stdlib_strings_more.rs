use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_clone() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Clone").expect("strings.Clone should be registered")
}

fn strings_contains_rune() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "ContainsRune")
        .expect("strings.ContainsRune should be registered")
}

fn strings_index_rune() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "IndexRune").expect("strings.IndexRune should be registered")
}

fn strings_compare() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Compare").expect("strings.Compare should be registered")
}

fn strings_replace() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Replace").expect("strings.Replace should be registered")
}

fn strings_index_byte() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "IndexByte").expect("strings.IndexByte should be registered")
}

fn strings_last_index_byte() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "LastIndexByte")
        .expect("strings.LastIndexByte should be registered")
}

fn strconv_quote_to_ascii() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "QuoteToASCII")
        .expect("strconv.QuoteToASCII should be registered")
}

fn strconv_quote_rune_to_ascii() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "QuoteRuneToASCII")
        .expect("strconv.QuoteRuneToASCII should be registered")
}

fn strconv_quote_rune() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "QuoteRune").expect("strconv.QuoteRune should be registered")
}

#[test]
fn executes_strings_clone() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "gowasm".into(),
                },
                Instruction::CallStdlib {
                    function: strings_clone(),
                    args: vec![0],
                    dst: Some(1),
                },
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
    assert_eq!(vm.stdout(), "gowasm\n");
}

#[test]
fn executes_strconv_quote_rune() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 955 },
                Instruction::LoadInt { dst: 1, value: 10 },
                Instruction::CallStdlib {
                    function: strconv_quote_rune(),
                    args: vec![0],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: strconv_quote_rune(),
                    args: vec![1],
                    dst: Some(3),
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
    assert_eq!(vm.stdout(), "'λ' '\\n'\n");
}

#[test]
fn executes_strings_contains_rune() {
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
                    value: "goλ".into(),
                },
                Instruction::LoadInt { dst: 1, value: 955 },
                Instruction::LoadString {
                    dst: 2,
                    value: "gowasm".into(),
                },
                Instruction::CallStdlib {
                    function: strings_contains_rune(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_contains_rune(),
                    args: vec![2, 1],
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
fn executes_strings_index_rune() {
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
                    value: "goλ".into(),
                },
                Instruction::LoadInt { dst: 1, value: 955 },
                Instruction::LoadString {
                    dst: 2,
                    value: "gowasm".into(),
                },
                Instruction::CallStdlib {
                    function: strings_index_rune(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_index_rune(),
                    args: vec![2, 1],
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
    assert_eq!(vm.stdout(), "2 -1\n");
}

#[test]
fn executes_strings_compare() {
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
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "hi".into(),
                },
                Instruction::CallStdlib {
                    function: strings_compare(),
                    args: vec![0, 0],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: strings_compare(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_compare(),
                    args: vec![1, 0],
                    dst: Some(4),
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
    assert_eq!(vm.stdout(), "0 -1 1\n");
}

#[test]
fn executes_strings_replace() {
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
                    value: "go go go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zig".into(),
                },
                Instruction::LoadInt { dst: 3, value: 2 },
                Instruction::LoadString {
                    dst: 4,
                    value: "abc".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "-".into(),
                },
                Instruction::CallStdlib {
                    function: strings_replace(),
                    args: vec![0, 1, 2, 3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: strings_replace(),
                    args: vec![4, 5, 6, 3],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 8],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "zig zig go -a-bc\n");
}

#[test]
fn executes_strings_index_byte() {
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
                Instruction::LoadInt { dst: 1, value: 119 },
                Instruction::LoadInt { dst: 2, value: 122 },
                Instruction::CallStdlib {
                    function: strings_index_byte(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_index_byte(),
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
    assert_eq!(vm.stdout(), "2 -1\n");
}

#[test]
fn executes_strings_last_index_byte() {
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
                    value: "gowasmwow".into(),
                },
                Instruction::LoadInt { dst: 1, value: 119 },
                Instruction::LoadInt { dst: 2, value: 122 },
                Instruction::CallStdlib {
                    function: strings_last_index_byte(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_last_index_byte(),
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
fn executes_strconv_quote_to_ascii() {
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
                    value: "go\nλ".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go\"".into(),
                },
                Instruction::CallStdlib {
                    function: strconv_quote_to_ascii(),
                    args: vec![0],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: strconv_quote_to_ascii(),
                    args: vec![1],
                    dst: Some(3),
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
    assert_eq!(vm.stdout(), "\"go\\n\\u03bb\" \"go\\\"\"\n");
}

#[test]
fn executes_strconv_quote_rune_to_ascii() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 955 },
                Instruction::LoadInt {
                    dst: 1,
                    value: 1_114_112,
                },
                Instruction::CallStdlib {
                    function: strconv_quote_rune_to_ascii(),
                    args: vec![0],
                    dst: Some(2),
                },
                Instruction::CallStdlib {
                    function: strconv_quote_rune_to_ascii(),
                    args: vec![1],
                    dst: Some(3),
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
    assert_eq!(vm.stdout(), "'\\u03bb' '\\ufffd'\n");
}
