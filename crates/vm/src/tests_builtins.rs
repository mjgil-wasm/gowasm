use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn builtin_len() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "len").expect("len should be registered")
}

fn builtin_append() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "append").expect("append should be registered")
}

fn builtin_range_keys() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "__range_keys").expect("range keys should be registered")
}

fn builtin_cap() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "cap").expect("cap should be registered")
}

fn builtin_delete() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "delete").expect("delete should be registered")
}

fn builtin_make_slice() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "__make_slice").expect("make slice should be registered")
}

fn builtin_range_value() -> super::StdlibFunctionId {
    resolve_stdlib_function("builtin", "__range_value").expect("range value should be registered")
}

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn computes_len_for_strings_and_collections() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 11,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::LoadInt { dst: 2, value: 2 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 3,
                    items: vec![1, 2],
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 5, value: 1 },
                Instruction::LoadInt { dst: 6, value: 0 },
                Instruction::MakeMap {
                    concrete_type: None,
                    dst: 7,
                    entries: vec![(4, 5)],
                    zero: 6,
                },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![0],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![3],
                    dst: Some(9),
                },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![7],
                    dst: Some(10),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![8, 9, 10],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 2 1\n");
}

#[test]
fn len_rejects_non_collection_values() {
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
                    function: builtin_len(),
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
        .expect_err("len should reject ints");
    assert!(error
        .to_string()
        .contains("`len` expects string, array, slice, map, or channel"));
}

#[test]
fn appends_values_onto_slices() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::LoadInt { dst: 3, value: 3 },
                Instruction::LoadInt { dst: 4, value: 4 },
                Instruction::CallStdlib {
                    function: builtin_append(),
                    args: vec![2, 3, 4],
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
    assert_eq!(vm.stdout(), "[1 2 3 4]\n");
}

#[test]
fn append_rejects_non_slice_targets() {
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
                Instruction::LoadInt { dst: 1, value: 8 },
                Instruction::CallStdlib {
                    function: builtin_append(),
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
        .expect_err("append should reject ints");
    assert!(error
        .to_string()
        .contains("`append` target must be a slice"));
}

#[test]
fn computes_range_keys_for_slices_and_maps() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 4 },
                Instruction::LoadInt { dst: 1, value: 5 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 4, value: 1 },
                Instruction::LoadInt { dst: 5, value: 0 },
                Instruction::MakeMap {
                    concrete_type: None,
                    dst: 6,
                    entries: vec![(3, 4)],
                    zero: 5,
                },
                Instruction::CallStdlib {
                    function: builtin_range_keys(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: builtin_range_keys(),
                    args: vec![6],
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
    assert_eq!(vm.stdout(), "[0 1] [go]\n");
}

#[test]
fn range_keys_rejects_invalid_targets() {
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
                    function: builtin_range_keys(),
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
        .expect_err("range keys should reject ints");
    assert!(error
        .to_string()
        .contains("`range` target must be an array, slice, map, or string"));
}

#[test]
fn computes_cap_for_arrays_and_slices() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeArray {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::LoadInt { dst: 3, value: 4 },
                Instruction::LoadInt { dst: 4, value: 5 },
                Instruction::LoadInt { dst: 5, value: 6 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 6,
                    items: vec![3, 4, 5],
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
                    args: vec![2],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
                    args: vec![6],
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
    assert_eq!(vm.stdout(), "2 3\n");
}

#[test]
fn cap_rejects_invalid_targets() {
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
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
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
        .expect_err("cap should reject strings");
    assert!(error
        .to_string()
        .contains("`cap` expects an array, slice, or channel"));
}

#[test]
fn len_and_cap_work_on_channels() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 9,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 3 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: Some(0),
                    zero: 2,
                },
                Instruction::LoadInt { dst: 3, value: 42 },
                Instruction::ChanSend { chan: 1, value: 3 },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![1],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::LoadNilChannel {
                    concrete_type: None,
                    dst: 6,
                },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![6],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
                    args: vec![6],
                    dst: Some(8),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 7, 8],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 3 0 0\n");
}

#[test]
fn copies_values_into_slices() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::LoadInt { dst: 2, value: 3 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 3,
                    items: vec![0, 1, 2],
                },
                Instruction::LoadInt { dst: 4, value: 7 },
                Instruction::LoadInt { dst: 5, value: 8 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 6,
                    items: vec![4, 5],
                },
                Instruction::Copy {
                    target: 3,
                    src: 6,
                    count_dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7, 3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 [7 8 3]\n");
}

#[test]
fn copy_rejects_non_slice_targets() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 7 },
                Instruction::LoadInt { dst: 1, value: 1 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![1],
                },
                Instruction::Copy {
                    target: 0,
                    src: 2,
                    count_dst: Some(3),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("copy should reject non-slice targets");
    assert!(error.to_string().contains("`copy` target must be a slice"));
}

#[test]
fn copy_rejects_non_slice_sources() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0],
                },
                Instruction::Copy {
                    target: 2,
                    src: 1,
                    count_dst: Some(3),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("copy should reject non-slice sources");
    assert!(error.to_string().contains("`copy` source must be a slice"));
}

#[test]
fn computes_range_keys_and_values_for_strings() {
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
                    value: "hé".into(),
                },
                Instruction::CallStdlib {
                    function: builtin_range_keys(),
                    args: vec![0],
                    dst: Some(1),
                },
                Instruction::LoadInt { dst: 2, value: 1 },
                Instruction::Index {
                    dst: 3,
                    target: 1,
                    index: 2,
                },
                Instruction::CallStdlib {
                    function: builtin_range_value(),
                    args: vec![0, 3],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![1, 3, 4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[0 1] 1 233\n");
}

#[test]
fn builtin_delete_removes_existing_map_entries() {
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
                    function: builtin_delete(),
                    args: vec![3, 0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: builtin_len(),
                    args: vec![4],
                    dst: Some(0),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![0],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn builtin_delete_is_a_no_op_on_nil_maps() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
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
                Instruction::LoadString {
                    dst: 2,
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: builtin_delete(),
                    args: vec![1, 2],
                    dst: Some(3),
                },
                Instruction::LoadNil { dst: 4 },
                Instruction::Compare {
                    dst: 0,
                    op: super::CompareOp::Equal,
                    left: 3,
                    right: 4,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![0],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn builtin_make_slice_builds_zero_filled_slices() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 3 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::CallStdlib {
                    function: builtin_make_slice(),
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
    assert_eq!(vm.stdout(), "[0 0 0]\n");
}

#[test]
fn builtin_make_slice_rejects_negative_lengths() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: -1 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::CallStdlib {
                    function: builtin_make_slice(),
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
        .expect_err("negative make length should fail");
    assert!(error.to_string().contains("must not be negative"));
}

#[test]
fn builtin_make_slice_accepts_explicit_capacity() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 6,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::LoadInt { dst: 1, value: 5 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::CallStdlib {
                    function: builtin_make_slice(),
                    args: vec![0, 1, 2],
                    dst: Some(3),
                },
                Instruction::CallStdlib {
                    function: builtin_cap(),
                    args: vec![3],
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
    assert_eq!(vm.stdout(), "[0 0] 5\n");
}

#[test]
fn builtin_make_slice_rejects_capacities_smaller_than_length() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 3 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::LoadInt { dst: 2, value: 0 },
                Instruction::CallStdlib {
                    function: builtin_make_slice(),
                    args: vec![0, 1, 2],
                    dst: Some(3),
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("smaller capacities should fail");
    assert!(error.to_string().contains("must be >= length 3"));
}

#[test]
fn nil_slices_compare_equal_to_nil() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadNilSlice {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadNil { dst: 1 },
                Instruction::Compare {
                    dst: 2,
                    op: super::CompareOp::Equal,
                    left: 0,
                    right: 1,
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
    assert_eq!(vm.stdout(), "true\n");
}
