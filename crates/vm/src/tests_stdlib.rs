use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn strings_contains() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Contains").expect("strings.Contains should be registered")
}

fn strings_has_prefix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "HasPrefix").expect("strings.HasPrefix should be registered")
}

fn strings_has_suffix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "HasSuffix").expect("strings.HasSuffix should be registered")
}

fn strings_trim_space() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "TrimSpace").expect("strings.TrimSpace should be registered")
}

fn strings_to_upper() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "ToUpper").expect("strings.ToUpper should be registered")
}

fn strings_to_lower() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "ToLower").expect("strings.ToLower should be registered")
}

fn strings_count() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Count").expect("strings.Count should be registered")
}

fn strings_repeat() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Repeat").expect("strings.Repeat should be registered")
}

fn strings_split() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Split").expect("strings.Split should be registered")
}

fn strings_join() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Join").expect("strings.Join should be registered")
}

fn strings_replace_all() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "ReplaceAll")
        .expect("strings.ReplaceAll should be registered")
}

fn strings_fields() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Fields").expect("strings.Fields should be registered")
}

fn strings_index() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "Index").expect("strings.Index should be registered")
}

fn strings_trim_prefix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "TrimPrefix")
        .expect("strings.TrimPrefix should be registered")
}

fn strings_trim_suffix() -> super::StdlibFunctionId {
    resolve_stdlib_function("strings", "TrimSuffix")
        .expect("strings.TrimSuffix should be registered")
}

fn strconv_itoa() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "Itoa").expect("strconv.Itoa should be registered")
}

fn strconv_format_bool() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "FormatBool")
        .expect("strconv.FormatBool should be registered")
}

fn strconv_quote() -> super::StdlibFunctionId {
    resolve_stdlib_function("strconv", "Quote").expect("strconv.Quote should be registered")
}

#[test]
fn executes_strings_search_helpers() {
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
                    value: "gowasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "was".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "asm".into(),
                },
                Instruction::CallStdlib {
                    function: strings_contains(),
                    args: vec![0, 1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: strings_has_prefix(),
                    args: vec![0, 2],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: strings_has_suffix(),
                    args: vec![0, 3],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\n");
}

#[test]
fn rejects_non_string_arguments_for_strings_helpers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 7 },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: strings_contains(),
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
        .expect_err("strings.Contains should reject ints");
    assert!(error
        .to_string()
        .contains("`strings.Contains` expects string arguments"));
}

#[test]
fn executes_strings_transform_helpers() {
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
                    value: "  go wasm  ".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "WASM".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim_space(),
                    args: vec![0],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_to_upper(),
                    args: vec![1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: strings_to_lower(),
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
    assert_eq!(vm.stdout(), "go wasm GO wasm\n");
}

#[test]
fn executes_strconv_format_helpers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 42 },
                Instruction::LoadBool {
                    dst: 1,
                    value: true,
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: strconv_itoa(),
                    args: vec![0],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strconv_format_bool(),
                    args: vec![1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: strconv_quote(),
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
    assert_eq!(vm.stdout(), "42 true \"go\"\n");
}

#[test]
fn executes_strings_count() {
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
                    value: "go go wasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: strings_count(),
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
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn executes_strings_repeat() {
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
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 1, value: 3 },
                Instruction::CallStdlib {
                    function: strings_repeat(),
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
    assert_eq!(vm.stdout(), "gogogo\n");
}

#[test]
fn executes_strings_split() {
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
                    value: "go,wasm,go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: ",".into(),
                },
                Instruction::CallStdlib {
                    function: strings_split(),
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
    assert_eq!(vm.stdout(), "[go wasm go]\n");
}

#[test]
fn executes_strings_join() {
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
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "wasm".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "go".into(),
                },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 3,
                    items: vec![0, 1, 2],
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "-".into(),
                },
                Instruction::CallStdlib {
                    function: strings_join(),
                    args: vec![3, 4],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![5],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go-wasm-go\n");
}

#[test]
fn executes_strings_replace_all() {
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
                    function: strings_replace_all(),
                    args: vec![0, 1, 2],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "zig wasm zig\n");
}

#[test]
fn executes_strings_fields() {
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
                    value: "  go\twasm  zig  ".into(),
                },
                Instruction::CallStdlib {
                    function: strings_fields(),
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
    assert_eq!(vm.stdout(), "[go wasm zig]\n");
}

#[test]
fn executes_strings_index() {
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
                    value: "go wasm".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "wasm".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zig".into(),
                },
                Instruction::CallStdlib {
                    function: strings_index(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_index(),
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
    assert_eq!(vm.stdout(), "3 -1\n");
}

#[test]
fn executes_strings_trim_prefix() {
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
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zig".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim_prefix(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_trim_prefix(),
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
    assert_eq!(vm.stdout(), "wasm gowasm\n");
}

#[test]
fn executes_strings_trim_suffix() {
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
                    value: "wasm".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "zig".into(),
                },
                Instruction::CallStdlib {
                    function: strings_trim_suffix(),
                    args: vec![0, 1],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: strings_trim_suffix(),
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
    assert_eq!(vm.stdout(), "go gowasm\n");
}
