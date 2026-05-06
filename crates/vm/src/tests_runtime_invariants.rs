use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::{
    register_program_type_inventory, ConcreteType, Function, Instruction, Program,
    ProgramTypeInventory, RuntimeInvariantMode, RuntimeTypeInfo, RuntimeTypeKind, TypeId, Value,
    ValueData, Vm, TYPE_FUNCTION, TYPE_INT, TYPE_STRING,
};

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "non-string panic payload".into()
}

fn single_instruction_program() -> Program {
    Program {
        entry_function: 0,
        global_count: 1,
        methods: Vec::new(),
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![Instruction::LoadInt { dst: 0, value: 1 }],
        }],
    }
}

#[test]
fn runtime_invariants_reject_mismatched_builtin_type_ids() {
    let program = single_instruction_program();
    let mut vm = Vm::new();
    vm.globals = vec![Value {
        typ: TYPE_INT,
        data: ValueData::String("bad".into()),
    }];

    let panic = catch_unwind(AssertUnwindSafe(|| vm.assert_runtime_invariants(&program)))
        .expect_err("mismatched builtin type id should panic");
    assert!(panic_message(panic).contains("builtin type id 1"));
}

#[test]
fn runtime_invariants_reject_frame_register_layout_corruption() {
    let program = single_instruction_program();
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");
    vm.goroutines[index].frames[0].registers.clear();

    let panic = catch_unwind(AssertUnwindSafe(|| vm.assert_runtime_invariants(&program)))
        .expect_err("broken frame register layout should panic");
    assert!(panic_message(panic).contains("register count 0 disagrees"));
}

#[test]
fn runtime_invariants_reject_gc_free_list_corruption() {
    let program = single_instruction_program();
    let mut vm = Vm::new();
    let _pointer = vm.box_heap_value(Value::int(7), TypeId(200));
    vm.free_heap_cells.push(0);

    let panic = catch_unwind(AssertUnwindSafe(|| vm.assert_runtime_invariants(&program)))
        .expect_err("live heap cell in free list should panic");
    assert!(panic_message(panic).contains("free heap cell index 0 still contains a live value"));
}

#[test]
fn runtime_invariant_mode_panics_after_instruction_progress() {
    let program = single_instruction_program();
    let mut vm = Vm::new();
    vm.set_runtime_invariant_mode(RuntimeInvariantMode::AfterEachInstruction);
    vm.globals = vec![Value {
        typ: TYPE_INT,
        data: ValueData::String("bad".into()),
    }];
    vm.spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = vm.resume_program(&program);
    }))
    .expect_err("runtime invariant mode should panic after the instruction finishes");
    assert!(panic_message(panic).contains("globals[0] uses builtin type id 1"));
}

#[test]
fn runtime_invariants_allow_nil_values_for_function_kinds() {
    let program = single_instruction_program();
    let mut inventory = ProgramTypeInventory::default();
    inventory.register(RuntimeTypeInfo {
        display_name: "Handler".into(),
        package_path: Some("main".into()),
        kind: RuntimeTypeKind::Function,
        type_id: Some(TypeId(117)),
        fields: Vec::new(),
        elem: None,
        key: None,
        len: None,
        params: vec![ConcreteType::TypeId(TYPE_STRING)],
        results: vec![ConcreteType::TypeId(TYPE_INT)],
        underlying: Some(Box::new(ConcreteType::Function {
            params: vec![ConcreteType::TypeId(TYPE_STRING)],
            results: vec![ConcreteType::TypeId(TYPE_INT)],
        })),
        channel_direction: None,
    });
    register_program_type_inventory(&program, inventory);

    let mut vm = Vm::new();
    vm.globals = vec![
        Value {
            typ: TYPE_FUNCTION,
            data: ValueData::Nil,
        },
        Value {
            typ: TypeId(117),
            data: ValueData::Nil,
        },
    ];
    vm.assert_runtime_invariants(&program);
}

#[test]
fn runtime_invariants_allow_well_typed_runtime_state() {
    let program = single_instruction_program();
    let mut vm = Vm::new();
    vm.globals = vec![Value {
        typ: TYPE_STRING,
        data: ValueData::String("ok".into()),
    }];
    vm.spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    vm.assert_runtime_invariants(&program);
}
