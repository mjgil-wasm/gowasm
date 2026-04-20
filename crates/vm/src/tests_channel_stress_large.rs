use super::{
    Function, GoroutineStatus, Instruction, Program, SelectCaseOp, SelectCaseOpKind, Value,
    ValueData, Vm,
};

fn broad_rotating_select_cases() -> Vec<SelectCaseOp> {
    vec![
        SelectCaseOp {
            chan: 10,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
        SelectCaseOp {
            chan: 11,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
        SelectCaseOp {
            chan: 12,
            kind: SelectCaseOpKind::Send { value: 30 },
        },
        SelectCaseOp {
            chan: 13,
            kind: SelectCaseOpKind::Send { value: 30 },
        },
        SelectCaseOp {
            chan: 14,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
        SelectCaseOp {
            chan: 15,
            kind: SelectCaseOpKind::Send { value: 31 },
        },
        SelectCaseOp {
            chan: 16,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
        SelectCaseOp {
            chan: 17,
            kind: SelectCaseOpKind::Send { value: 32 },
        },
        SelectCaseOp {
            chan: 18,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
        SelectCaseOp {
            chan: 19,
            kind: SelectCaseOpKind::Recv {
                value_dst: 25,
                ok_dst: Some(26),
            },
        },
    ]
}

fn run_large_ready_select_sequence(default_case: Option<usize>) {
    let program = queue_program(35);
    let cases = broad_rotating_select_cases();
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");

    let recv_a = vm.alloc_channel_value(1, Value::int(0));
    let recv_a_id = channel_id(&recv_a);
    let send_a = vm.alloc_channel_value(1, Value::int(0));
    let recv_b = vm.alloc_channel_value(1, Value::int(0));
    let recv_b_id = channel_id(&recv_b);
    let send_b = vm.alloc_channel_value(1, Value::int(0));
    let recv_c = vm.alloc_channel_value(1, Value::int(0));
    let recv_c_id = channel_id(&recv_c);
    let send_c = vm.alloc_channel_value(1, Value::int(0));
    let recv_d = vm.alloc_channel_value(1, Value::int(0));
    let recv_d_id = channel_id(&recv_d);
    let closed_recv = vm.alloc_channel_value(1, Value::int(0));
    let closed_recv_id = channel_id(&closed_recv);

    vm.channels[recv_a_id as usize]
        .buffer
        .push_back(Value::int(11));
    vm.channels[recv_b_id as usize]
        .buffer
        .push_back(Value::int(44));
    vm.channels[recv_c_id as usize]
        .buffer
        .push_back(Value::int(55));
    vm.channels[recv_d_id as usize]
        .buffer
        .push_back(Value::int(66));
    vm.channels[closed_recv_id as usize]
        .buffer
        .push_back(Value::int(77));
    vm.channels[closed_recv_id as usize].closed = true;

    vm.set_register_on_goroutine(&program, goroutine, 10, Value::nil_channel())
        .expect("nil receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 11, recv_a)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 12, Value::nil_channel())
        .expect("nil send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 13, send_a.clone())
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 14, recv_b)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 15, send_b.clone())
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 16, recv_c)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 17, send_c.clone())
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 18, recv_d)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 19, closed_recv)
        .expect("closed receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 30, Value::int(88))
        .expect("send value register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 31, Value::int(99))
        .expect("send value register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 32, Value::int(111))
        .expect("send value register should be writable");

    vm.current_goroutine = index;
    for (expected_choice, expected_value) in [
        (1, Some(11)),
        (3, None),
        (4, Some(44)),
        (5, None),
        (6, Some(55)),
        (7, None),
        (8, Some(66)),
        (9, Some(77)),
    ] {
        vm.execute_select(&program, 24, &cases, default_case)
            .expect("large mixed select should succeed");
        let frame = vm.goroutines[index]
            .frames
            .last()
            .expect("goroutine should retain its frame");
        assert_eq!(frame.registers[24], Value::int(expected_choice));
        if let Some(expected_value) = expected_value {
            assert_eq!(frame.registers[25], Value::int(expected_value));
            assert_eq!(frame.registers[26], Value::bool(true));
        }
    }

    vm.execute_chan_recv(&program, 24, 13)
        .expect("first sent value should be buffered");
    assert_eq!(
        vm.goroutines[index]
            .frames
            .last()
            .expect("goroutine should retain its frame")
            .registers[24],
        Value::int(88)
    );
    vm.execute_chan_recv(&program, 24, 15)
        .expect("second sent value should be buffered");
    assert_eq!(
        vm.goroutines[index]
            .frames
            .last()
            .expect("goroutine should retain its frame")
            .registers[24],
        Value::int(99)
    );
    vm.execute_chan_recv(&program, 24, 17)
        .expect("third sent value should be buffered");
    assert_eq!(
        vm.goroutines[index]
            .frames
            .last()
            .expect("goroutine should retain its frame")
            .registers[24],
        Value::int(111)
    );
}

fn queue_program(register_count: usize) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "worker".into(),
            param_count: 0,
            register_count,
            code: vec![Instruction::Return { src: None }],
        }],
    }
}

fn channel_id(value: &Value) -> u64 {
    let ValueData::Channel(channel) = &value.data else {
        panic!("expected channel value");
    };
    channel.id.expect("channel should be live")
}

#[test]
fn blocking_select_rotates_across_large_mixed_case_sets() {
    run_large_ready_select_sequence(None);
}

#[test]
fn default_select_rotates_across_large_mixed_case_sets() {
    run_large_ready_select_sequence(Some(10));
}

#[test]
fn large_blocked_select_resumes_on_closed_case_across_nil_and_live_cases() {
    let program = queue_program(35);
    let cases = broad_rotating_select_cases();
    let mut vm = Vm::new();
    let goroutine = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("goroutine should spawn");
    let index = vm
        .goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist");

    let empty_recv_a = vm.alloc_channel_value(0, Value::int(0));
    let send_a = vm.alloc_channel_value(0, Value::int(0));
    let empty_recv_b = vm.alloc_channel_value(0, Value::int(0));
    let send_b = vm.alloc_channel_value(0, Value::int(0));
    let empty_recv_c = vm.alloc_channel_value(0, Value::int(0));
    let send_c = vm.alloc_channel_value(0, Value::int(0));
    let empty_recv_d = vm.alloc_channel_value(0, Value::int(0));
    let closed_recv = vm.alloc_channel_value(0, Value::int(0));
    let closed_recv_id = channel_id(&closed_recv);

    vm.set_register_on_goroutine(&program, goroutine, 10, Value::nil_channel())
        .expect("nil receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 11, empty_recv_a)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 12, Value::nil_channel())
        .expect("nil send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 13, send_a)
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 14, empty_recv_b)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 15, send_b)
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 16, empty_recv_c)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 17, send_c)
        .expect("send register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 18, empty_recv_d)
        .expect("receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 19, closed_recv)
        .expect("closed receive register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 30, Value::int(88))
        .expect("send value register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 31, Value::int(99))
        .expect("send value register should be writable");
    vm.set_register_on_goroutine(&program, goroutine, 32, Value::int(111))
        .expect("send value register should be writable");

    vm.current_goroutine = index;
    vm.execute_select(&program, 24, &cases, None)
        .expect("large mixed select should block");
    assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
    assert!(vm.goroutines[index].active_select.is_some());

    vm.close_channel_by_id(&program, closed_recv_id)
        .expect("close should wake the blocked select");

    let frame = vm.goroutines[index]
        .frames
        .last()
        .expect("goroutine should retain its frame");
    assert_eq!(vm.goroutines[index].status, GoroutineStatus::Runnable);
    assert_eq!(vm.goroutines[index].active_select, None);
    assert_eq!(frame.registers[24], Value::int(9));
    assert_eq!(frame.registers[25], Value::int(0));
    assert_eq!(frame.registers[26], Value::bool(false));
}
