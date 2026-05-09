use std::collections::VecDeque;

use super::{
    channels::{ChannelState, PendingSend},
    gc_metadata::{
        channel_state_descriptor, context_state_descriptor, deferred_call_descriptor,
        heap_cell_descriptor, value_gc_descriptor, visit_channel_state_gc_slots,
        visit_context_state_gc_slots, visit_deferred_call_gc_slots, visit_heap_cell_gc_slots,
        visit_value_gc_slots, GcObjectDescriptor, GcObjectKind, GcSlotDescriptor, GcSlotKind,
    },
    scheduler::GoroutineId,
    ContextState, DeferredCall, DeferredCallKind, TypeId, Value, ValueData,
};

fn slot(name: &'static str, kind: GcSlotKind) -> GcSlotDescriptor {
    GcSlotDescriptor { name, kind }
}

fn assert_descriptor(
    descriptor: &GcObjectDescriptor,
    kind: GcObjectKind,
    fixed_slots: &[GcSlotDescriptor],
    variable_slots: &[GcSlotDescriptor],
) {
    assert_eq!(descriptor.kind, kind);
    assert_eq!(descriptor.fixed_slots, fixed_slots);
    assert_eq!(descriptor.variable_slots, variable_slots);
}

fn collect_gc_slots(
    mut visit_slots: impl FnMut(&mut dyn FnMut(&'static str, &Value)),
) -> Vec<(&'static str, Value)> {
    let mut slots = Vec::new();
    visit_slots(&mut |name, value| slots.push((name, value.clone())));
    slots
}

#[test]
fn exposes_gc_descriptors_for_heap_managed_runtime_object_kinds() {
    assert_descriptor(
        heap_cell_descriptor(),
        GcObjectKind::HeapCell,
        &[slot("payload", GcSlotKind::Value)],
        &[],
    );
    assert_descriptor(
        value_gc_descriptor(&Value::function(7, vec![Value::int(1)])).unwrap(),
        GcObjectKind::ClosureEnv,
        &[],
        &[slot("captures", GcSlotKind::ValueVec)],
    );
    assert_descriptor(
        value_gc_descriptor(&Value::array(vec![Value::int(1)])).unwrap(),
        GcObjectKind::ArrayValue,
        &[],
        &[slot("elements", GcSlotKind::ValueVec)],
    );
    assert_descriptor(
        value_gc_descriptor(&Value {
            typ: TypeId(200),
            data: ValueData::Struct(vec![("x".into(), Value::int(1))]),
        })
        .unwrap(),
        GcObjectKind::StructValue,
        &[],
        &[slot("fields", GcSlotKind::ValueVec)],
    );
    assert_descriptor(
        value_gc_descriptor(&Value::slice(vec![Value::int(1)])).unwrap(),
        GcObjectKind::SliceValue,
        &[],
        &[slot("elements", GcSlotKind::ValueVec)],
    );
    assert_descriptor(
        value_gc_descriptor(&Value::map(vec![], Value::string(""))).unwrap(),
        GcObjectKind::MapValue,
        &[slot("zero_value", GcSlotKind::Value)],
        &[slot("entries", GcSlotKind::ValuePairVec)],
    );
    assert_descriptor(
        value_gc_descriptor(&Value::wrapped_error("outer".into(), Value::error("inner"))).unwrap(),
        GcObjectKind::ErrorValue,
        &[slot("wrapped", GcSlotKind::OptionalValue)],
        &[],
    );
    assert_descriptor(
        channel_state_descriptor(),
        GcObjectKind::ChannelState,
        &[slot("zero_value", GcSlotKind::Value)],
        &[
            slot("buffer", GcSlotKind::ValueVec),
            slot("pending_send_values", GcSlotKind::ValueVec),
        ],
    );
    assert_descriptor(
        deferred_call_descriptor(),
        GcObjectKind::DeferredCall,
        &[slot("closure_function", GcSlotKind::OptionalValue)],
        &[slot("args", GcSlotKind::ValueVec)],
    );
    assert_descriptor(
        context_state_descriptor(),
        GcObjectKind::ContextState,
        &[
            slot("parent_value", GcSlotKind::OptionalValue),
            slot("err", GcSlotKind::OptionalValue),
        ],
        &[slot("key_values", GcSlotKind::ValuePairVec)],
    );
    assert_eq!(value_gc_descriptor(&Value::int(1)), None);
}

#[test]
fn visits_gc_slots_for_heap_cells_and_value_backed_objects() {
    let closure = Value::function(9, vec![Value::int(1), Value::string("two")]);
    let array = Value::array(vec![Value::int(3), Value::int(4)]);
    let struct_value = Value {
        typ: TypeId(301),
        data: ValueData::Struct(vec![
            ("left".into(), Value::int(5)),
            ("right".into(), Value::string("six")),
        ]),
    };
    let slice = Value::slice(vec![Value::bool(true), Value::bool(false)]);
    let map = Value::map(
        vec![(Value::string("k"), Value::int(7))],
        Value::string("zero"),
    );
    let wrapped = Value::wrapped_error("outer".into(), Value::error("inner"));

    assert_eq!(
        collect_gc_slots(|visit| visit_heap_cell_gc_slots(&Value::int(8), visit)),
        vec![("payload", Value::int(8))]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&closure, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::ClosureEnv)
            );
        }),
        vec![
            ("captures", Value::int(1)),
            ("captures", Value::string("two")),
        ]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&array, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::ArrayValue)
            );
        }),
        vec![("elements", Value::int(3)), ("elements", Value::int(4))]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&struct_value, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::StructValue)
            );
        }),
        vec![("fields", Value::int(5)), ("fields", Value::string("six")),]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&slice, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::SliceValue)
            );
        }),
        vec![
            ("elements", Value::bool(true)),
            ("elements", Value::bool(false)),
        ]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&map, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::MapValue)
            );
        }),
        vec![
            ("zero_value", Value::string("zero")),
            ("entries", Value::string("k")),
            ("entries", Value::int(7)),
        ]
    );
    assert_eq!(
        collect_gc_slots(|visit| {
            assert_eq!(
                visit_value_gc_slots(&wrapped, visit).map(|descriptor| descriptor.kind),
                Some(GcObjectKind::ErrorValue)
            );
        }),
        vec![("wrapped", Value::error("inner"))]
    );
}

#[test]
fn visits_gc_slots_for_channel_deferred_and_context_runtime_state() {
    let mut buffer = VecDeque::new();
    buffer.push_back(Value::int(1));
    buffer.push_back(Value::int(2));
    let channel = ChannelState {
        capacity: 2,
        closed: false,
        buffer,
        pending_sends: VecDeque::from([PendingSend {
            goroutine: GoroutineId(4),
            value: Value::string("queued"),
            select: None,
        }]),
        pending_receivers: VecDeque::new(),
        zero_value: Value::int(0),
    };
    let deferred = DeferredCall {
        kind: DeferredCallKind::Closure {
            function: Value::function(17, vec![Value::int(9)]),
        },
        args: vec![Value::string("arg"), Value::bool(true)],
    };
    let context = ContextState {
        parent_id: Some(1),
        parent_value: Some(Value::string("parent")),
        children: vec![2, 3],
        done_channel_id: Some(7),
        deadline_unix_nanos: Some(99),
        err: Some(Value::error("canceled")),
        values: vec![(Value::string("key"), Value::int(11))],
    };

    assert_eq!(
        collect_gc_slots(|visit| visit_channel_state_gc_slots(&channel, visit)),
        vec![
            ("zero_value", Value::int(0)),
            ("buffer", Value::int(1)),
            ("buffer", Value::int(2)),
            ("pending_send_values", Value::string("queued")),
        ]
    );
    assert_eq!(
        collect_gc_slots(|visit| visit_deferred_call_gc_slots(&deferred, visit)),
        vec![
            ("closure_function", Value::function(17, vec![Value::int(9)])),
            ("args", Value::string("arg")),
            ("args", Value::bool(true)),
        ]
    );
    assert_eq!(
        collect_gc_slots(|visit| visit_context_state_gc_slots(&context, visit)),
        vec![
            ("parent_value", Value::string("parent")),
            ("err", Value::error("canceled")),
            ("key_values", Value::string("key")),
            ("key_values", Value::int(11)),
        ]
    );
}
