use super::{resolve_stdlib_function, CompareOp, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn formats_map_values_with_fmt_println() {
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
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeMap {
                    concrete_type: None,
                    dst: 3,
                    entries: vec![(0, 1)],
                    zero: 2,
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
    assert_eq!(vm.stdout(), "map[go:1]\n");
}

#[test]
fn reads_zero_values_from_nil_maps() {
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
                    value: "missing".into(),
                },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeNilMap {
                    dst: 2,
                    concrete_type: None,
                    zero: 1,
                },
                Instruction::Index {
                    dst: 3,
                    target: 2,
                    index: 0,
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
    assert_eq!(vm.stdout(), "map[] 0\n");
}

#[test]
fn updates_existing_map_entries_and_appends_new_ones() {
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
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeMap {
                    concrete_type: None,
                    dst: 3,
                    entries: vec![(0, 1)],
                    zero: 2,
                },
                Instruction::LoadInt { dst: 4, value: 4 },
                Instruction::SetIndex {
                    target: 3,
                    index: 0,
                    src: 4,
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "wasm".into(),
                },
                Instruction::LoadInt { dst: 6, value: 2 },
                Instruction::SetIndex {
                    target: 3,
                    index: 5,
                    src: 6,
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
    assert_eq!(vm.stdout(), "map[go:4 wasm:2]\n");
}

#[test]
fn assigning_into_nil_maps_fails() {
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
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeNilMap {
                    dst: 2,
                    concrete_type: None,
                    zero: 1,
                },
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::SetIndex {
                    target: 2,
                    index: 0,
                    src: 3,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil map writes should fail");
    assert!(error.to_string().contains("cannot assign into a nil map"));
}

#[test]
fn nil_maps_compare_equal_to_nil() {
    let program = Program {
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeNilMap {
                    dst: 1,
                    concrete_type: None,
                    zero: 0,
                },
                Instruction::LoadNil { dst: 2 },
                Instruction::Compare {
                    dst: 3,
                    op: CompareOp::Equal,
                    left: 1,
                    right: 2,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
        methods: vec![],
        global_count: 0,
        entry_function: 0,
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn map_contains_reports_present_and_missing_keys() {
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
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::MakeMap {
                    concrete_type: None,
                    dst: 3,
                    entries: vec![(0, 1)],
                    zero: 2,
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "go".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "wasm".into(),
                },
                Instruction::MapContains {
                    dst: 6,
                    target: 3,
                    index: 4,
                },
                Instruction::MapContains {
                    dst: 7,
                    target: 3,
                    index: 5,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![6, 7],
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
fn map_contains_rejects_non_map_targets() {
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
                Instruction::MapContains {
                    dst: 2,
                    target: 0,
                    index: 1,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("map contains should reject ints");
    let text = error.to_string();
    assert!(text.contains("comma-ok lookup target must be a map in function `main`"));
    assert!(text.contains("got int value `7`"));
}
