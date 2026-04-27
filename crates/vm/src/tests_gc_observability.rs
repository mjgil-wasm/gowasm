use super::{Value, Vm, TYPE_POINTER};

#[test]
fn gc_stats_expose_threshold_live_cells_and_collection_totals() {
    let mut vm = Vm::new();
    let initial = vm.gc_stats();
    assert_eq!(initial.allocation_threshold, Some(256));
    assert_eq!(initial.allocations_since_gc, 0);
    assert_eq!(initial.heap_cells, 0);
    assert_eq!(initial.live_heap_cells, 0);
    assert_eq!(initial.free_heap_cells, 0);
    assert_eq!(initial.last_freed_cells, 0);
    assert_eq!(initial.total_collections, 0);
    assert_eq!(initial.total_freed_cells, 0);

    let left = vm.box_heap_value(Value::int(7), TYPE_POINTER);
    let right = vm.box_heap_value(Value::int(9), TYPE_POINTER);
    vm.globals = vec![left];

    let before_collect = vm.gc_stats();
    assert_eq!(before_collect.allocations_since_gc, 2);
    assert_eq!(before_collect.heap_cells, 2);
    assert_eq!(before_collect.live_heap_cells, 2);
    assert_eq!(before_collect.free_heap_cells, 0);
    assert_eq!(before_collect.total_collections, 0);

    assert_eq!(vm.collect_garbage(), 1);
    let after_collect = vm.gc_stats();
    assert_eq!(after_collect.allocations_since_gc, 0);
    assert_eq!(after_collect.heap_cells, 2);
    assert_eq!(after_collect.live_heap_cells, 1);
    assert_eq!(after_collect.free_heap_cells, 1);
    assert_eq!(after_collect.last_freed_cells, 1);
    assert_eq!(after_collect.total_collections, 1);
    assert_eq!(after_collect.total_freed_cells, 1);

    vm.globals.clear();
    assert_eq!(vm.collect_garbage(), 1);
    let final_stats = vm.gc_stats();
    assert_eq!(final_stats.live_heap_cells, 0);
    assert_eq!(final_stats.free_heap_cells, 2);
    assert_eq!(final_stats.last_freed_cells, 1);
    assert_eq!(final_stats.total_collections, 2);
    assert_eq!(final_stats.total_freed_cells, 2);

    let _reused = vm.box_heap_value(Value::int(11), TYPE_POINTER);
    let reused_stats = vm.gc_stats();
    assert_eq!(reused_stats.allocations_since_gc, 1);
    assert_eq!(reused_stats.heap_cells, 2);
    assert_eq!(reused_stats.live_heap_cells, 1);
    assert_eq!(reused_stats.free_heap_cells, 1);

    let _ = right;
}
