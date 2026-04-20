use super::{
    resolve_stdlib_function, CompareOp, Function, Instruction, Program, TypeId, Vm, TYPE_POINTER,
};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

#[test]
fn compares_nil_pointers_against_nil() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadNilPointer {
                    dst: 0,
                    typ: TYPE_POINTER,
                    concrete_type: None,
                },
                Instruction::LoadNil { dst: 1 },
                Instruction::Compare {
                    dst: 2,
                    op: CompareOp::Equal,
                    left: 0,
                    right: 1,
                },
                Instruction::Compare {
                    dst: 3,
                    op: CompareOp::NotEqual,
                    left: 0,
                    right: 1,
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
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn takes_local_addresses_and_dereferences_them() {
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
                Instruction::AddressLocal {
                    dst: 1,
                    src: 0,
                    typ: TYPE_POINTER,
                },
                Instruction::Deref { dst: 2, src: 1 },
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
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn writes_through_local_pointers() {
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
                Instruction::AddressLocal {
                    dst: 1,
                    src: 0,
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 2, value: 9 },
                Instruction::StoreIndirect { target: 1, src: 2 },
                Instruction::Deref { dst: 3, src: 1 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![0, 3],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 9\n");
}

#[test]
fn reads_and_writes_through_local_field_pointers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 5,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::MakeStruct {
                    dst: 1,
                    typ: super::TypeId(100),
                    fields: vec![("x".into(), 0)],
                },
                Instruction::AddressLocalField {
                    dst: 2,
                    src: 1,
                    field: "x".into(),
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 3, value: 9 },
                Instruction::StoreIndirect { target: 2, src: 3 },
                Instruction::Deref { dst: 4, src: 2 },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn reads_and_writes_through_projected_field_pointers() {
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
                Instruction::MakeStruct {
                    dst: 1,
                    typ: TypeId(100),
                    fields: vec![("x".into(), 0)],
                },
                Instruction::AddressLocal {
                    dst: 2,
                    src: 1,
                    typ: TypeId(101),
                },
                Instruction::ProjectFieldPointer {
                    dst: 3,
                    src: 2,
                    field: "x".into(),
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 4, value: 9 },
                Instruction::StoreIndirect { target: 3, src: 4 },
                Instruction::Deref { dst: 5, src: 3 },
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
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn reads_and_writes_through_local_index_pointers() {
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
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::AddressLocalIndex {
                    dst: 4,
                    src: 2,
                    index: 3,
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 5, value: 9 },
                Instruction::StoreIndirect { target: 4, src: 5 },
                Instruction::Deref { dst: 0, src: 4 },
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
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn reads_and_writes_through_projected_index_pointers() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 7,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 2,
                    items: vec![0, 1],
                },
                Instruction::AddressLocal {
                    dst: 3,
                    src: 2,
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 4, value: 1 },
                Instruction::ProjectIndexPointer {
                    dst: 5,
                    src: 3,
                    index: 4,
                    typ: TYPE_POINTER,
                },
                Instruction::LoadInt { dst: 6, value: 9 },
                Instruction::StoreIndirect { target: 5, src: 6 },
                Instruction::Deref { dst: 0, src: 5 },
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
    assert_eq!(vm.stdout(), "9\n");
}
