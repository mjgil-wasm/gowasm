use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn path_base() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Base").expect("path.Base should be registered")
}

fn path_clean() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Clean").expect("path.Clean should be registered")
}

fn path_dir() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Dir").expect("path.Dir should be registered")
}

fn path_ext() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Ext").expect("path.Ext should be registered")
}

fn path_is_abs() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "IsAbs").expect("path.IsAbs should be registered")
}

fn path_split() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Split").expect("path.Split should be registered")
}

fn path_join() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Join").expect("path.Join should be registered")
}

fn path_match() -> super::StdlibFunctionId {
    resolve_stdlib_function("path", "Match").expect("path.Match should be registered")
}

#[test]
fn executes_path_helpers() {
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
                    value: "/a/b/".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "a/../../b".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "archive.tar.gz".into(),
                },
                Instruction::CallStdlib {
                    function: path_base(),
                    args: vec![0],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: path_clean(),
                    args: vec![1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: path_dir(),
                    args: vec![0],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: path_ext(),
                    args: vec![2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: path_is_abs(),
                    args: vec![0],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: path_is_abs(),
                    args: vec![1],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3, 4, 5, 6, 7, 8],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b ../b /a/b .gz true false\n");
}

#[test]
fn executes_path_split_pairs() {
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
                    value: "a/b".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "../a".into(),
                },
                Instruction::CallStdlibMulti {
                    function: path_split(),
                    args: vec![0],
                    dsts: vec![2, 3],
                },
                Instruction::CallStdlibMulti {
                    function: path_split(),
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
    assert_eq!(vm.stdout(), "a/ b ../ a\n");
}

#[test]
fn executes_path_join() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 14,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "a".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "b".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "/a".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "../b".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "c".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "/a/b".into(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "..".into(),
                },
                Instruction::CallStdlib {
                    function: path_join(),
                    args: vec![],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: path_join(),
                    args: vec![0, 1, 2],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: path_join(),
                    args: vec![3, 4, 5],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: path_join(),
                    args: vec![6, 7, 5],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![8, 9, 10, 11],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), " a/b /b/c /a/c\n");
}

#[test]
fn executes_path_match() {
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
                    value: "a?c".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "abc".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "*".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "a/b".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "[".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "go".into(),
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![0, 1],
                    dsts: vec![6, 7],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![2, 3],
                    dsts: vec![8, 9],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![4, 5],
                    dsts: vec![10, 11],
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
    assert_eq!(
        vm.stdout(),
        "true <nil> false <nil> false syntax error in pattern\n"
    );
}

#[test]
fn executes_path_match_edge_cases() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 21,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "a\\*b".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "a*b".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "ab[^e-g]".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "abc".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "a?b".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "a☺b".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "[a-ζ]*".into(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "α".into(),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "[-]".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "-".into(),
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![0, 1],
                    dsts: vec![10, 11],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![2, 3],
                    dsts: vec![12, 13],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![4, 5],
                    dsts: vec![14, 15],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![6, 7],
                    dsts: vec![16, 17],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![8, 9],
                    dsts: vec![18, 19],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
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
        "true <nil> true <nil> true <nil> true <nil> false syntax error in pattern\n"
    );
}

#[test]
fn executes_path_match_star_scans() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 25,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "a*/b".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "abc/b".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "a*/b".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "a/c/b".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "a*b*c*d*e*/f".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "axbxcxdxe/f".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "a*b*c*d*e*/f".into(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "axbxcxdxe/xxx/f".into(),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "a*b?c*x".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "abxbbxdbxebxczzx".into(),
                },
                Instruction::LoadString {
                    dst: 10,
                    value: "a*b?c*x".into(),
                },
                Instruction::LoadString {
                    dst: 11,
                    value: "abxbbxdbxebxczzy".into(),
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![0, 1],
                    dsts: vec![12, 13],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![2, 3],
                    dsts: vec![14, 15],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![4, 5],
                    dsts: vec![16, 17],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![6, 7],
                    dsts: vec![18, 19],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![8, 9],
                    dsts: vec![20, 21],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![10, 11],
                    dsts: vec![22, 23],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
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
        "true <nil> false <nil> true <nil> false <nil> true <nil> false <nil>\n"
    );
}

#[test]
fn executes_path_match_bad_pattern_cases() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 25,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "[\\]a]".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "]".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "[x\\-]".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "-".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "[\\-x]".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "x".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "\\".into(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "a".into(),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "[a-b-c]".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "a".into(),
                },
                Instruction::LoadString {
                    dst: 10,
                    value: "[^bc".into(),
                },
                Instruction::LoadString {
                    dst: 11,
                    value: "a".into(),
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![0, 1],
                    dsts: vec![12, 13],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![2, 3],
                    dsts: vec![14, 15],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![4, 5],
                    dsts: vec![16, 17],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![6, 7],
                    dsts: vec![18, 19],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![8, 9],
                    dsts: vec![20, 21],
                },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![10, 11],
                    dsts: vec![22, 23],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
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
        "true <nil> true <nil> true <nil> false syntax error in pattern false syntax error in pattern false syntax error in pattern\n"
    );
}

#[test]
fn rejects_non_string_arguments_for_path_helpers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 7 },
                Instruction::CallStdlib {
                    function: path_clean(),
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
        .expect_err("path.Clean should reject ints");
    assert!(error
        .to_string()
        .contains("`path.Clean` expects a string argument"));
}

#[test]
fn rejects_non_string_arguments_for_path_join() {
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
                    value: "a".into(),
                },
                Instruction::LoadInt { dst: 1, value: 7 },
                Instruction::CallStdlib {
                    function: path_join(),
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
        .expect_err("path.Join should reject ints");
    assert!(error
        .to_string()
        .contains("`path.Join` expects string arguments"));
}

#[test]
fn rejects_non_string_arguments_for_path_match() {
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
                    value: "a*".into(),
                },
                Instruction::LoadInt { dst: 1, value: 7 },
                Instruction::CallStdlibMulti {
                    function: path_match(),
                    args: vec![0, 1],
                    dsts: vec![1, 2],
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("path.Match should reject ints");
    assert!(error
        .to_string()
        .contains("`path.Match` expects string arguments"));
}
