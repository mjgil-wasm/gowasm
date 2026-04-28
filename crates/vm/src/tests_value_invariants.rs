use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::{
    ConcreteType, ContextState, Function, FunctionValue, MapValue, PointerTarget, PointerValue,
    Program, TypeId, Value, ValueData, Vm, TYPE_BASE64_ENCODING, TYPE_CONTEXT, TYPE_FUNCTION,
    TYPE_INT, TYPE_POINTER, TYPE_REGEXP, TYPE_SLICE, TYPE_STRING, TYPE_SYNC_WAIT_GROUP, TYPE_TIME,
    TYPE_TIME_TIMER,
};

fn test_program() -> Program {
    Program {
        entry_function: 0,
        global_count: 1,
        methods: Vec::new(),
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: Vec::new(),
        }],
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "non-string panic payload".into()
}

#[test]
fn value_invariants_accept_supported_runtime_shapes() {
    let program = test_program();
    let mut vm = Vm::new();

    let channel = vm.alloc_channel_value(1, Value::int(0));
    let timer = Value::struct_value(
        TYPE_TIME_TIMER,
        vec![
            ("C".into(), channel.clone()),
            ("__time_timer_channel".into(), channel.clone()),
        ],
    );
    let pointer = vm.box_heap_value(Value::int(7), TYPE_POINTER);

    vm.context_values.insert(11, ContextState::default());
    vm.wait_groups.insert(21, Default::default());
    vm.compiled_regexps
        .insert(31, regex::Regex::new("ok").expect("regex should compile"));

    let values = vec![
        Value::int(1),
        Value::struct_value(
            TYPE_TIME,
            vec![("__time_unix_nanos".into(), Value::int(123))],
        ),
        Value::struct_value(TYPE_CONTEXT, vec![("__context_id".into(), Value::int(11))]),
        Value::struct_value(
            TYPE_SYNC_WAIT_GROUP,
            vec![("__sync_wait_group_id".into(), Value::int(21))],
        ),
        Value {
            typ: TYPE_REGEXP,
            data: ValueData::Struct(vec![("__regexp_id".into(), Value::int(31))]),
        },
        Value {
            typ: TYPE_BASE64_ENCODING,
            data: ValueData::Struct(vec![("__encodingKind".into(), Value::int(0))]),
        },
        Value::slice_typed(
            vec![Value::int(1), Value::int(2)],
            ConcreteType::Slice {
                element: Box::new(ConcreteType::TypeId(TYPE_INT)),
            },
        ),
        Value {
            typ: TypeId(900),
            data: ValueData::Map(MapValue::with_entries(
                vec![(Value::int(1), Value::string("one"))],
                Value::string(""),
                Some(ConcreteType::Map {
                    key: Box::new(ConcreteType::TypeId(TYPE_INT)),
                    value: Box::new(ConcreteType::TypeId(TYPE_STRING)),
                }),
            )),
        },
        Value {
            typ: TYPE_FUNCTION,
            data: ValueData::Function(FunctionValue {
                function: 0,
                captures: vec![Value::bool(true)],
                concrete_type: Some(ConcreteType::Function {
                    params: Vec::new(),
                    results: Vec::new(),
                }),
            }),
        },
        Value {
            typ: TYPE_POINTER,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::HeapCell { cell: 0 },
                concrete_type: Some(ConcreteType::Pointer {
                    element: Box::new(ConcreteType::TypeId(TYPE_INT)),
                }),
            }),
        },
        timer,
        pointer,
    ];

    for value in values {
        vm.assert_value_invariants(&program, &value);
    }
}

#[test]
fn value_invariants_reject_invalid_slice_capacity() {
    let program = test_program();
    let vm = Vm::new();
    let value = Value {
        typ: TYPE_SLICE,
        data: ValueData::Slice(crate::SliceValue {
            values: Rc::new(RefCell::new(vec![Value::int(1)])),
            start: 0,
            len: 1,
            cap: 0,
            is_nil: false,
            concrete_type: None,
        }),
    };

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_value_invariants(&program, &value)
    }))
    .expect_err("invalid slice capacity should panic");
    assert!(panic_message(panic).contains("slice cap 0 is smaller than len 1"));
}

#[test]
fn value_invariants_reject_map_index_corruption() {
    let program = test_program();
    let vm = Vm::new();
    let mut wrong_index = HashMap::new();
    wrong_index.insert(999, vec![0]);
    let map = MapValue::with_entries(
        vec![(Value::int(1), Value::string("one"))],
        Value::string(""),
        None,
    );
    *map.index
        .as_ref()
        .expect("map index should exist")
        .borrow_mut() = wrong_index;
    let value = Value {
        typ: TypeId(910),
        data: ValueData::Map(map),
    };

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_value_invariants(&program, &value)
    }))
    .expect_err("mismatched map index should panic");
    assert!(panic_message(panic).contains("map index does not match its entries"));
}

#[test]
fn value_invariants_reject_dangling_heap_pointers() {
    let program = test_program();
    let mut vm = Vm::new();
    let pointer = vm.box_heap_value(Value::int(7), TYPE_POINTER);
    vm.heap_cells[0] = None;

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_value_invariants(&program, &pointer)
    }))
    .expect_err("dangling heap pointer should panic");
    assert!(panic_message(panic).contains("heap cell 0 is not live"));
}

#[test]
fn value_invariants_reject_missing_host_registry_entries() {
    let program = test_program();
    let vm = Vm::new();
    let value = Value::struct_value(TYPE_CONTEXT, vec![("__context_id".into(), Value::int(404))]);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        vm.assert_value_invariants(&program, &value)
    }))
    .expect_err("missing context registry entry should panic");
    assert!(panic_message(panic).contains("missing registry id 404"));
}
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn value_invariants_accept_zero_value_timer() {
    let program = test_program();
    let vm = Vm::new();
    let timer = Value::struct_value(TYPE_TIME_TIMER, Vec::new());
    vm.assert_value_invariants(&program, &timer);
}
