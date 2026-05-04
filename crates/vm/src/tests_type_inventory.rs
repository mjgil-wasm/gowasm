use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    register_program_type_inventory, value_runtime_type, ConcreteType, Function, Program,
    ProgramTypeInventory, RuntimeTypeField, RuntimeTypeInfo, RuntimeTypeKind, SliceValue, TypeId,
    Value, ValueData, Vm, TYPE_ANY, TYPE_INT, TYPE_STRING,
};

fn empty_program() -> Program {
    Program {
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 0,
            code: Vec::new(),
        }],
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    }
}

fn base_inventory() -> ProgramTypeInventory {
    let mut inventory = ProgramTypeInventory::default();
    inventory.register(RuntimeTypeInfo::scalar(
        "int",
        RuntimeTypeKind::Int,
        Some(TYPE_INT),
    ));
    inventory.register(RuntimeTypeInfo::scalar(
        "interface{}",
        RuntimeTypeKind::Interface,
        Some(TYPE_ANY),
    ));
    inventory.register(RuntimeTypeInfo::scalar(
        "string",
        RuntimeTypeKind::String,
        Some(TYPE_STRING),
    ));
    inventory
}

#[test]
fn value_runtime_type_prefers_named_type_ids_over_wrapper_metadata() {
    let program = empty_program();
    let mut inventory = base_inventory();
    inventory.register(RuntimeTypeInfo {
        display_name: "Labels".into(),
        package_path: Some("main".into()),
        kind: RuntimeTypeKind::Slice,
        type_id: Some(TypeId(200)),
        fields: Vec::new(),
        elem: Some(Box::new(ConcreteType::TypeId(TYPE_INT))),
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: Some(Box::new(ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_INT)),
        })),
        channel_direction: None,
    });
    register_program_type_inventory(&program, inventory);

    let value = Value {
        typ: TypeId(200),
        data: ValueData::Slice(SliceValue {
            values: Rc::new(RefCell::new(vec![Value::int(1)])),
            start: 0,
            len: 1,
            cap: 1,
            is_nil: false,
            concrete_type: Some(ConcreteType::Slice {
                element: Box::new(ConcreteType::TypeId(TYPE_INT)),
            }),
        }),
    };

    let info = value_runtime_type(&program, &Vm::new(), &value)
        .expect("named slice alias should resolve through its type id");
    assert_eq!(info.display_name, "Labels");
    assert_eq!(info.kind, RuntimeTypeKind::Slice);
}

#[test]
fn value_runtime_type_handles_typed_nil_interfaces_and_typed_slices() {
    let program = empty_program();
    register_program_type_inventory(&program, base_inventory());

    let typed_nil = Value {
        typ: TYPE_ANY,
        data: ValueData::Nil,
    };
    let nil_info = value_runtime_type(&program, &Vm::new(), &typed_nil)
        .expect("typed nil interface should keep interface metadata");
    assert_eq!(nil_info.display_name, "interface{}");
    assert_eq!(nil_info.kind, RuntimeTypeKind::Interface);

    let slice = Value::slice_typed(
        vec![Value::int(1), Value::int(2)],
        ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_INT)),
        },
    );
    let slice_info =
        value_runtime_type(&program, &Vm::new(), &slice).expect("typed slice should resolve");
    assert_eq!(slice_info.display_name, "[]int");
    assert_eq!(slice_info.kind, RuntimeTypeKind::Slice);
}

#[test]
fn value_runtime_type_handles_named_nil_interfaces_nil_aliases_and_generic_instances() {
    let program = empty_program();
    let mut inventory = base_inventory();
    inventory.register(RuntimeTypeInfo {
        display_name: "Labels".into(),
        package_path: Some("main".into()),
        kind: RuntimeTypeKind::Slice,
        type_id: Some(TypeId(200)),
        fields: Vec::new(),
        elem: Some(Box::new(ConcreteType::TypeId(TYPE_STRING))),
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: Some(Box::new(ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        })),
        channel_direction: None,
    });
    inventory.register(RuntimeTypeInfo {
        display_name: "Notifier".into(),
        package_path: Some("main".into()),
        kind: RuntimeTypeKind::Interface,
        type_id: Some(TypeId(300)),
        fields: Vec::new(),
        elem: None,
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: None,
        channel_direction: None,
    });
    inventory.register(RuntimeTypeInfo {
        display_name: "Box[Labels]".into(),
        package_path: Some("main".into()),
        kind: RuntimeTypeKind::Struct,
        type_id: Some(TypeId(301)),
        fields: vec![
            RuntimeTypeField {
                name: "Value".into(),
                typ: ConcreteType::TypeId(TypeId(200)),
                embedded: false,
                tag: None,
            },
            RuntimeTypeField {
                name: "Any".into(),
                typ: ConcreteType::TypeId(TypeId(300)),
                embedded: false,
                tag: None,
            },
            RuntimeTypeField {
                name: "Next".into(),
                typ: ConcreteType::Pointer {
                    element: Box::new(ConcreteType::TypeId(TypeId(301))),
                },
                embedded: false,
                tag: None,
            },
        ],
        elem: None,
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: None,
        channel_direction: None,
    });
    register_program_type_inventory(&program, inventory);

    let notifier_nil = Value {
        typ: TypeId(300),
        data: ValueData::Nil,
    };
    let notifier_info = value_runtime_type(&program, &Vm::new(), &notifier_nil)
        .expect("named nil interfaces should keep interface metadata");
    assert_eq!(notifier_info.display_name, "Notifier");
    assert_eq!(notifier_info.kind, RuntimeTypeKind::Interface);

    let labels_nil = Value {
        typ: TypeId(200),
        data: ValueData::Slice(SliceValue {
            values: Rc::new(RefCell::new(Vec::new())),
            start: 0,
            len: 0,
            cap: 0,
            is_nil: true,
            concrete_type: Some(ConcreteType::Slice {
                element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
            }),
        }),
    };
    let labels_info = value_runtime_type(&program, &Vm::new(), &labels_nil)
        .expect("named nil aliases should prefer their type id");
    assert_eq!(labels_info.display_name, "Labels");
    assert_eq!(labels_info.kind, RuntimeTypeKind::Slice);

    let boxed_value = Value {
        typ: TypeId(301),
        data: ValueData::Struct(vec![
            ("Value".into(), labels_nil),
            ("Any".into(), notifier_nil),
            ("Next".into(), Value::nil()),
        ]),
    };
    let boxed_info = value_runtime_type(&program, &Vm::new(), &boxed_value)
        .expect("generic instances should keep their registered metadata");
    assert_eq!(boxed_info.display_name, "Box[Labels]");
    assert_eq!(boxed_info.kind, RuntimeTypeKind::Struct);
    assert_eq!(boxed_info.fields.len(), 3);
    assert_eq!(boxed_info.fields[0].typ, ConcreteType::TypeId(TypeId(200)));
    assert_eq!(boxed_info.fields[1].typ, ConcreteType::TypeId(TypeId(300)));
    assert_eq!(
        boxed_info.fields[2].typ,
        ConcreteType::Pointer {
            element: Box::new(ConcreteType::TypeId(TypeId(301))),
        }
    );
}

#[test]
fn value_runtime_type_prefers_dynamic_concrete_type_for_non_nil_interface_wrappers() {
    let program = empty_program();
    let mut inventory = base_inventory();
    inventory.register(RuntimeTypeInfo {
        display_name: "[]int".into(),
        package_path: None,
        kind: RuntimeTypeKind::Slice,
        type_id: Some(TypeId(200)),
        fields: Vec::new(),
        elem: Some(Box::new(ConcreteType::TypeId(TYPE_INT))),
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: None,
        channel_direction: None,
    });
    register_program_type_inventory(&program, inventory);

    let wrapped_slice = Value {
        typ: TYPE_ANY,
        data: ValueData::Slice(SliceValue {
            values: Rc::new(RefCell::new(vec![Value::int(1), Value::int(2)])),
            start: 0,
            len: 2,
            cap: 2,
            is_nil: false,
            concrete_type: Some(ConcreteType::TypeId(TypeId(200))),
        }),
    };

    let info = value_runtime_type(&program, &Vm::new(), &wrapped_slice)
        .expect("non-nil interface wrappers should resolve their dynamic concrete type");
    assert_eq!(info.display_name, "[]int");
    assert_eq!(info.kind, RuntimeTypeKind::Slice);
}

#[test]
fn value_runtime_type_prefers_dynamic_scalar_type_for_non_nil_interface_wrappers() {
    let program = empty_program();
    let mut inventory = base_inventory();
    inventory.register(RuntimeTypeInfo::scalar(
        "float64",
        RuntimeTypeKind::Float64,
        Some(crate::TYPE_FLOAT64),
    ));
    register_program_type_inventory(&program, inventory);

    let wrapped_float = Value {
        typ: TYPE_ANY,
        data: ValueData::Float(crate::Float64(1.5)),
    };

    let info = value_runtime_type(&program, &Vm::new(), &wrapped_float)
        .expect("non-nil interface scalar wrappers should resolve their dynamic concrete type");
    assert_eq!(info.display_name, "float64");
    assert_eq!(info.kind, RuntimeTypeKind::Float64);
}
