use std::collections::VecDeque;

use super::{
    channels::ChannelState, ContextDeadlineTimer, ContextState, PointerTarget, Value, ValueData,
    Vm, TYPE_CONTEXT, TYPE_POINTER,
};

fn heap_cell(pointer: &Value) -> usize {
    let ValueData::Pointer(pointer) = &pointer.data else {
        panic!("expected pointer value");
    };
    let PointerTarget::HeapCell { cell } = pointer.target else {
        panic!("expected heap-cell pointer");
    };
    cell
}

#[test]
fn collects_unreachable_heap_cells_and_reuses_freed_slots() {
    let mut vm = Vm::default();

    let rooted = vm.box_heap_value(Value::int(7), TYPE_POINTER);
    let abandoned = vm.box_heap_value(Value::int(9), TYPE_POINTER);
    vm.globals = vec![rooted.clone()];

    assert_eq!(vm.collect_garbage(), 1);
    assert_eq!(
        vm.heap_cells.iter().filter(|slot| slot.is_some()).count(),
        1
    );
    assert!(vm.heap_cells[heap_cell(&abandoned)].is_none());

    let reused = vm.box_heap_value(Value::int(11), TYPE_POINTER);
    assert_eq!(heap_cell(&reused), heap_cell(&abandoned));
    assert_eq!(vm.heap_cells[heap_cell(&rooted)], Some(Value::int(7)));
    assert_eq!(vm.heap_cells[heap_cell(&reused)], Some(Value::int(11)));
}

#[test]
fn mark_traversal_preserves_heap_cells_through_pointer_context_and_channel_edges() {
    let mut vm = Vm::default();

    let nested = vm.box_heap_value(Value::int(99), TYPE_POINTER);
    let root_chain = vm.box_heap_value(nested.clone(), TYPE_POINTER);
    let unreachable = vm.box_heap_value(Value::int(123), TYPE_POINTER);
    let channel_payload = vm.box_heap_value(Value::int(33), TYPE_POINTER);
    let context_payload = vm.box_heap_value(Value::int(44), TYPE_POINTER);

    vm.globals = vec![root_chain.clone()];
    vm.channels = vec![ChannelState {
        capacity: 1,
        closed: false,
        buffer: VecDeque::from([channel_payload.clone()]),
        pending_sends: VecDeque::new(),
        pending_receivers: VecDeque::new(),
        zero_value: Value::int(0),
    }];
    vm.context_values.insert(
        1,
        ContextState {
            parent_id: None,
            parent_value: Some(context_payload.clone()),
            children: Vec::new(),
            done_channel_id: Some(0),
            deadline_unix_nanos: Some(5),
            err: None,
            values: Vec::new(),
        },
    );
    vm.context_deadline_timers = vec![ContextDeadlineTimer {
        context_id: 1,
        remaining_nanos: 5,
    }];

    assert_eq!(vm.collect_garbage(), 1);

    assert_eq!(vm.heap_cells[heap_cell(&root_chain)], Some(nested.clone()));
    assert_eq!(vm.heap_cells[heap_cell(&nested)], Some(Value::int(99)));
    assert_eq!(
        vm.heap_cells[heap_cell(&channel_payload)],
        Some(Value::int(33))
    );
    assert_eq!(
        vm.heap_cells[heap_cell(&context_payload)],
        Some(Value::int(44))
    );
    assert!(vm.heap_cells[heap_cell(&unreachable)].is_none());

    let context_root =
        Value::struct_value(TYPE_CONTEXT, vec![("__context_id".into(), Value::int(1))]);
    vm.globals.push(context_root);
    assert_eq!(vm.collect_garbage(), 0);
}
