use std::collections::{HashSet, VecDeque};

use super::{channels::ChannelState, TypeId, Value, Vm, TYPE_POINTER};

const TYPE_NODE: TypeId = TypeId(900);

fn heap_cell(pointer: &Value) -> usize {
    let super::ValueData::Pointer(pointer) = &pointer.data else {
        panic!("expected pointer value");
    };
    let super::PointerTarget::HeapCell { cell } = pointer.target else {
        panic!("expected heap-cell pointer");
    };
    cell
}

fn live_heap_cells(vm: &Vm) -> usize {
    vm.heap_cells.iter().filter(|slot| slot.is_some()).count()
}

fn node_value(id: i64, next: Value) -> Value {
    Value::struct_value(
        TYPE_NODE,
        vec![("id".into(), Value::int(id)), ("next".into(), next)],
    )
}

#[test]
fn collects_and_reuses_mixed_runtime_graphs_across_repeated_cycles() {
    let mut vm = Vm::default();

    for cycle in 0..6 {
        let left = vm.box_heap_value(
            node_value(cycle as i64, Value::nil_pointer(TYPE_POINTER)),
            TYPE_POINTER,
        );
        let right = vm.box_heap_value(node_value(-(cycle as i64) - 1, left.clone()), TYPE_POINTER);
        let leaf = vm.box_heap_value(Value::int(100 + cycle as i64), TYPE_POINTER);

        let left_cell = heap_cell(&left);
        let right_cell = heap_cell(&right);
        vm.heap_cells[left_cell] = Some(node_value(cycle as i64, right.clone()));
        vm.heap_cells[right_cell] = Some(node_value(-(cycle as i64) - 1, left.clone()));

        let map_payload = Value::map(
            vec![
                (Value::string("left"), left.clone()),
                (Value::string("right"), right.clone()),
                (Value::string("leaf"), leaf.clone()),
            ],
            Value::nil_pointer(TYPE_POINTER),
        );
        let closure = Value::function(7, vec![leaf.clone(), map_payload.clone()]);

        vm.callback_result = Some(vec![Value::wrapped_error(
            format!("cycle-{cycle}"),
            closure,
        )]);
        vm.globals = vec![Value::channel(0)];
        vm.channels = vec![ChannelState {
            capacity: 1,
            closed: false,
            buffer: VecDeque::from([map_payload]),
            pending_sends: VecDeque::new(),
            pending_receivers: VecDeque::new(),
            zero_value: Value::nil(),
        }];

        assert_eq!(vm.collect_garbage(), 0);
        assert_eq!(live_heap_cells(&vm), 3);

        vm.callback_result = None;
        assert_eq!(vm.collect_garbage(), 0);

        vm.globals.clear();
        vm.channels.clear();
        assert_eq!(vm.collect_garbage(), 3);
        assert_eq!(live_heap_cells(&vm), 0);

        let freed_cells = vm.free_heap_cells.iter().copied().collect::<HashSet<_>>();
        assert_eq!(freed_cells.len(), 3);

        let reused = [
            vm.box_heap_value(Value::int(1000 + cycle as i64), TYPE_POINTER),
            vm.box_heap_value(Value::int(2000 + cycle as i64), TYPE_POINTER),
            vm.box_heap_value(Value::int(3000 + cycle as i64), TYPE_POINTER),
        ];
        for pointer in &reused {
            assert!(freed_cells.contains(&heap_cell(pointer)));
        }
        assert_eq!(vm.collect_garbage(), 3);
        assert_eq!(live_heap_cells(&vm), 0);
    }
}
