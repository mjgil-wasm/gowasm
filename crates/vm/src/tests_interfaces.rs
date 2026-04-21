use super::{
    Function, Instruction, InterfaceMethodCheck, MethodBinding, Program, TypeCheck, TypeId, Vm,
};

#[test]
fn loads_nil_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadNil { dst: 0 },
                Instruction::Move { dst: 1, src: 0 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn asserts_matching_runtime_types() {
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
                Instruction::AssertType {
                    dst: 1,
                    src: 0,
                    target: TypeCheck::Int,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn rejects_mismatched_runtime_types() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadBool {
                    dst: 0,
                    value: true,
                },
                Instruction::AssertType {
                    dst: 1,
                    src: 0,
                    target: TypeCheck::Struct {
                        type_id: TypeId(123),
                        name: "Point".into(),
                    },
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("type assertion should fail");
    assert!(error
        .to_string()
        .contains("type assertion to `Point` failed"));
}

#[test]
fn computes_type_match_bools() {
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
                Instruction::TypeMatches {
                    dst: 1,
                    src: 0,
                    target: TypeCheck::Int,
                },
                Instruction::TypeMatches {
                    dst: 2,
                    src: 0,
                    target: TypeCheck::Bool,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn rejects_nil_interfaces_for_interface_targets() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadNil { dst: 0 },
                Instruction::Retag {
                    dst: 1,
                    src: 0,
                    typ: super::TYPE_ANY,
                },
                Instruction::AssertType {
                    dst: 2,
                    src: 1,
                    target: TypeCheck::Interface {
                        name: "Any".into(),
                        methods: vec![],
                    },
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil interface assertion should fail");
    assert!(error.to_string().contains("type assertion to `Any` failed"));
}

#[test]
fn narrows_typed_nil_pointer_assertions_to_exact_pointer_types() {
    let pointer_type = TypeId(201);
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
                    typ: pointer_type,
                    concrete_type: Some(super::ConcreteType::TypeId(pointer_type)),
                },
                Instruction::Retag {
                    dst: 1,
                    src: 0,
                    typ: super::TYPE_ANY,
                },
                Instruction::AssertType {
                    dst: 2,
                    src: 1,
                    target: TypeCheck::Exact {
                        type_id: pointer_type,
                        name: "*Point".into(),
                    },
                },
                Instruction::IsNil { dst: 3, src: 2 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("typed nil pointer assertion should succeed");
}

#[test]
fn matches_interface_targets_from_method_sets() {
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: TypeId(123),
            target_receiver_type: TypeId(123),
            name: "area".into(),
            function: 1,
            param_types: Vec::new(),
            result_types: Vec::new(),
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::MakeStruct {
                        dst: 1,
                        typ: TypeId(123),
                        fields: vec![],
                    },
                    Instruction::TypeMatches {
                        dst: 2,
                        src: 1,
                        target: TypeCheck::Interface {
                            name: "Shape".into(),
                            methods: vec![InterfaceMethodCheck {
                                name: "area".into(),
                                param_types: Vec::new(),
                                result_types: Vec::new(),
                            }],
                        },
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "Point.area".into(),
                param_count: 1,
                register_count: 1,
                code: vec![Instruction::Return { src: None }],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn rejects_interface_targets_with_mismatched_method_signatures() {
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: TypeId(123),
            target_receiver_type: TypeId(123),
            name: "read".into(),
            function: 1,
            param_types: vec!["string".into()],
            result_types: vec!["int".into()],
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::MakeStruct {
                        dst: 0,
                        typ: TypeId(123),
                        fields: vec![],
                    },
                    Instruction::AssertType {
                        dst: 1,
                        src: 0,
                        target: TypeCheck::Interface {
                            name: "Reader".into(),
                            methods: vec![InterfaceMethodCheck {
                                name: "read".into(),
                                param_types: vec!["int".into()],
                                result_types: vec!["int".into()],
                            }],
                        },
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "Point.read".into(),
                param_count: 2,
                register_count: 2,
                code: vec![Instruction::Return { src: None }],
            },
        ],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    assert!(error
        .to_string()
        .contains("type assertion to `Reader` failed"));
}

#[test]
fn dispatches_methods_for_named_pointer_receiver_types() {
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: TypeId(201),
            target_receiver_type: TypeId(100),
            name: "area".into(),
            function: 1,
            param_types: Vec::new(),
            result_types: vec!["int".into()],
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::LoadNilPointer {
                        dst: 0,
                        typ: TypeId(201),
                        concrete_type: None,
                    },
                    Instruction::CallMethod {
                        receiver: 0,
                        method: "area".into(),
                        args: vec![],
                        dst: Some(2),
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "*Point.area".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 7 },
                    Instruction::Return { src: Some(1) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn dispatches_value_receiver_methods_for_pointer_interface_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![MethodBinding {
            receiver_type: TypeId(201),
            target_receiver_type: TypeId(100),
            name: "area".into(),
            function: 1,
            param_types: Vec::new(),
            result_types: vec!["int".into()],
            promoted_fields: Vec::new(),
        }],
        global_count: 0,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 8 },
                    Instruction::MakeStruct {
                        dst: 1,
                        typ: TypeId(100),
                        fields: vec![("x".into(), 0)],
                    },
                    Instruction::BoxHeap {
                        dst: 2,
                        src: 1,
                        typ: TypeId(201),
                    },
                    Instruction::CallMethod {
                        receiver: 2,
                        method: "area".into(),
                        args: vec![],
                        dst: Some(3),
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "Point.area".into(),
                param_count: 1,
                register_count: 2,
                code: vec![
                    Instruction::GetField {
                        dst: 1,
                        target: 0,
                        field: "x".into(),
                    },
                    Instruction::Return { src: Some(1) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn computes_nil_matches_for_interface_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadNil { dst: 0 },
                Instruction::LoadString {
                    dst: 1,
                    value: "bad".into(),
                },
                Instruction::IsNil { dst: 2, src: 0 },
                Instruction::IsNil { dst: 3, src: 1 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
}

#[test]
fn loads_error_messages_from_error_values() {
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
                    value: "bad".into(),
                },
                Instruction::LoadErrorMessage { dst: 2, src: 0 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    assert!(error
        .to_string()
        .contains("builtin error target must be an error value"));
}
