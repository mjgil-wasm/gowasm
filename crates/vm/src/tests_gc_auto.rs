use super::{
    resolve_stdlib_function, CapabilityRequest, Function, Instruction, Program, RunOutcome, Value,
    ValueData, Vm, TYPE_POINTER, TYPE_TIME_DURATION,
};

fn heap_cell(pointer: &Value) -> usize {
    let ValueData::Pointer(pointer) = &pointer.data else {
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

#[test]
fn vm_new_enables_default_automatic_gc_policy() {
    let vm = Vm::new();
    assert_eq!(vm.gc_allocation_threshold, Some(256));
}

#[test]
fn automatic_gc_reclaims_long_running_allocation_churn() {
    let mut code = Vec::new();
    for value in 0..32 {
        code.push(Instruction::LoadInt { dst: 0, value });
        code.push(Instruction::BoxHeap {
            dst: 1,
            src: 0,
            typ: TYPE_POINTER,
        });
        code.push(Instruction::StoreGlobal { global: 0, src: 1 });
    }
    code.push(Instruction::Return { src: None });

    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code,
        }],
    };

    let mut vm = Vm::new();
    vm.set_gc_allocation_threshold(1);
    vm.run_program(&program).expect("program should run");

    assert!(live_heap_cells(&vm) <= 2);
    assert!(vm.heap_cells.len() <= 3);
    let pointer = vm.globals[0].clone();
    assert_eq!(vm.heap_cells[heap_cell(&pointer)], Some(Value::int(31)));
}

#[test]
fn automatic_gc_preserves_values_transferred_across_goroutines_and_channels() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 2,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 0 },
                    Instruction::LoadInt { dst: 1, value: 1 },
                    Instruction::MakeChannel {
                        dst: 2,
                        concrete_type: None,
                        cap: Some(1),
                        zero: 0,
                    },
                    Instruction::StoreGlobal { global: 0, src: 2 },
                    Instruction::GoCall {
                        function: 1,
                        args: vec![2],
                    },
                    Instruction::ChanRecv { dst: 3, chan: 2 },
                    Instruction::Deref { dst: 1, src: 3 },
                    Instruction::StoreGlobal { global: 1, src: 1 },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "worker".into(),
                param_count: 1,
                register_count: 4,
                code: vec![
                    Instruction::LoadInt { dst: 1, value: 41 },
                    Instruction::BoxHeap {
                        dst: 2,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::ChanSend { chan: 0, value: 2 },
                    Instruction::LoadInt { dst: 1, value: 99 },
                    Instruction::BoxHeap {
                        dst: 3,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::LoadInt { dst: 1, value: 100 },
                    Instruction::BoxHeap {
                        dst: 3,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::LoadInt { dst: 1, value: 101 },
                    Instruction::BoxHeap {
                        dst: 3,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::Return { src: None },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.set_gc_allocation_threshold(1);
    vm.run_program(&program).expect("program should run");

    assert_eq!(vm.globals[1], Value::int(41));
    assert!(vm.heap_cells.len() <= 3);
}

#[test]
fn automatic_gc_preserves_live_closure_captures_across_repeated_collections() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 4,
                code: vec![
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![],
                        dst: Some(0),
                    },
                    Instruction::LoadInt { dst: 1, value: 100 },
                    Instruction::BoxHeap {
                        dst: 2,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::LoadInt { dst: 1, value: 200 },
                    Instruction::BoxHeap {
                        dst: 2,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::LoadInt { dst: 1, value: 300 },
                    Instruction::BoxHeap {
                        dst: 2,
                        src: 1,
                        typ: TYPE_POINTER,
                    },
                    Instruction::CallClosure {
                        callee: 0,
                        args: vec![],
                        dst: Some(3),
                    },
                    Instruction::StoreGlobal { global: 0, src: 3 },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "make".into(),
                param_count: 0,
                register_count: 3,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 41 },
                    Instruction::BoxHeap {
                        dst: 1,
                        src: 0,
                        typ: TYPE_POINTER,
                    },
                    Instruction::MakeClosure {
                        concrete_type: None,
                        dst: 2,
                        function: 2,
                        captures: vec![1],
                    },
                    Instruction::Return { src: Some(2) },
                ],
            },
            Function {
                name: "next".into(),
                param_count: 1,
                register_count: 4,
                code: vec![
                    Instruction::Deref { dst: 1, src: 0 },
                    Instruction::LoadInt { dst: 2, value: 1 },
                    Instruction::Add {
                        dst: 1,
                        left: 1,
                        right: 2,
                    },
                    Instruction::StoreIndirect { target: 0, src: 1 },
                    Instruction::Return { src: Some(1) },
                ],
            },
        ],
    };

    let mut vm = Vm::new();
    vm.set_gc_allocation_threshold(1);
    vm.run_program(&program).expect("program should run");

    assert_eq!(vm.globals[0], Value::int(42));
    assert!(vm.heap_cells.len() <= 3);
}

#[test]
fn automatic_gc_reclaims_heap_churn_across_cooperative_yield_pauses() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 2,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::BoxHeap {
                    dst: 1,
                    src: 0,
                    typ: TYPE_POINTER,
                },
                Instruction::StoreGlobal { global: 0, src: 1 },
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::BoxHeap {
                    dst: 1,
                    src: 0,
                    typ: TYPE_POINTER,
                },
                Instruction::StoreGlobal { global: 0, src: 1 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_gc_allocation_threshold(1);
    vm.set_instruction_yield_interval(2);

    let mut yields = 0;
    let mut outcome = vm.start_program(&program).expect("program should start");
    loop {
        match outcome {
            RunOutcome::Completed => break,
            RunOutcome::CapabilityRequest(CapabilityRequest::Yield) => {
                yields += 1;
                outcome = vm.resume_program(&program).expect("program should resume");
            }
            other => panic!("unexpected run outcome: {other:?}"),
        }
    }

    assert!(yields > 0);
    assert_eq!(
        vm.heap_cells[heap_cell(&vm.globals[0])],
        Some(Value::int(2))
    );
    assert!(vm.gc_stats().total_collections > 0);
    assert_eq!(vm.collect_garbage(), 1);
    assert_eq!(live_heap_cells(&vm), 1);
}

#[test]
fn automatic_gc_preserves_live_heap_roots_across_paused_host_waits() {
    let sleep = resolve_stdlib_function("time", "Sleep").expect("time.Sleep should exist");
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 41 },
                Instruction::BoxHeap {
                    dst: 1,
                    src: 0,
                    typ: TYPE_POINTER,
                },
                Instruction::StoreGlobal { global: 0, src: 1 },
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::Retag {
                    dst: 2,
                    src: 0,
                    typ: TYPE_TIME_DURATION,
                },
                Instruction::CallStdlib {
                    function: sleep,
                    args: vec![2],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_gc_allocation_threshold(1);

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 1);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(
        vm.heap_cells[heap_cell(&vm.globals[0])],
        Some(Value::int(41))
    );
    assert!(vm.gc_stats().total_collections > 0);

    vm.advance_timers(&program, 1, None)
        .expect("timers should advance");
    assert_eq!(
        vm.resume_program(&program).expect("program should resume"),
        RunOutcome::Completed
    );

    assert_eq!(
        vm.heap_cells[heap_cell(&vm.globals[0])],
        Some(Value::int(41))
    );
}
