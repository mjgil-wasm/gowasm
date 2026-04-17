use super::{GoroutineStatus, Instruction, Program, Value, ValueData, Vm};

fn worker_program(register_count: usize) -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![super::Function {
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

fn goroutine_index(vm: &Vm, goroutine: super::scheduler::GoroutineId) -> usize {
    vm.goroutines
        .iter()
        .position(|candidate| candidate.id == goroutine)
        .expect("goroutine should exist")
}

fn register_int(vm: &Vm, goroutine_index: usize, register: usize) -> i64 {
    let value = &vm.goroutines[goroutine_index]
        .frames
        .last()
        .expect("goroutine should have a frame")
        .registers[register];
    let ValueData::Int(value) = value.data else {
        panic!("expected int register");
    };
    value
}

fn buffered_values(vm: &Vm, channel_id: u64) -> Vec<i64> {
    vm.channels[channel_id as usize]
        .buffer
        .iter()
        .map(|value| {
            let ValueData::Int(value) = value.data else {
                panic!("expected buffered int value");
            };
            value
        })
        .collect()
}

#[test]
fn buffered_receive_wakes_blocked_senders_in_fifo_order_under_contention() {
    let program = worker_program(2);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(2, Value::int(0));
    let channel_id = channel_id(&channel);
    vm.channels[channel_id as usize]
        .buffer
        .extend([Value::int(10), Value::int(20)]);

    let mut senders = Vec::new();
    for value in [30, 40, 50] {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("sender goroutine should spawn");
        let index = goroutine_index(&vm, goroutine);
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.set_register_on_goroutine(&program, goroutine, 1, Value::int(value))
            .expect("value register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_send(&program, 0, 1)
            .expect("full buffered send should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        senders.push(index);
    }

    let receiver = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("receiver goroutine should spawn");
    let receiver_index = goroutine_index(&vm, receiver);
    vm.set_register_on_goroutine(&program, receiver, 0, channel)
        .expect("channel register should be writable");

    for (step, expected) in [(0, 10), (1, 20), (2, 30)] {
        vm.current_goroutine = receiver_index;
        vm.execute_chan_recv(&program, 1, 0)
            .expect("buffered receive should succeed");
        assert_eq!(register_int(&vm, receiver_index, 1), expected);
        assert_eq!(
            vm.goroutines[senders[step]].status,
            GoroutineStatus::Runnable
        );
    }

    assert_eq!(buffered_values(&vm, channel_id), vec![40, 50]);
    assert!(vm.channels[channel_id as usize].pending_sends.is_empty());
    assert_eq!(vm.goroutines[senders[1]].status, GoroutineStatus::Runnable);
    assert_eq!(vm.goroutines[senders[2]].status, GoroutineStatus::Runnable);
}

#[test]
fn buffered_send_wakes_blocked_receivers_in_fifo_order_under_contention() {
    let program = worker_program(2);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(2, Value::int(0));
    let channel_id = channel_id(&channel);

    let mut receivers = Vec::new();
    for _ in 0..3 {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("receiver goroutine should spawn");
        let index = goroutine_index(&vm, goroutine);
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_recv(&program, 1, 0)
            .expect("empty buffered receive should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        receivers.push(index);
    }

    let sender = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("sender goroutine should spawn");
    let sender_index = goroutine_index(&vm, sender);
    vm.set_register_on_goroutine(&program, sender, 0, channel)
        .expect("channel register should be writable");

    for (step, value) in [(0, 70), (1, 80), (2, 90)] {
        vm.set_register_on_goroutine(&program, sender, 1, Value::int(value))
            .expect("value register should be writable");
        vm.current_goroutine = sender_index;
        vm.execute_chan_send(&program, 0, 1)
            .expect("buffered send should wake the oldest receiver");
        assert_eq!(
            vm.goroutines[receivers[step]].status,
            GoroutineStatus::Runnable
        );
        assert_eq!(register_int(&vm, receivers[step], 1), value);
    }

    assert!(vm.channels[channel_id as usize]
        .pending_receivers
        .is_empty());
    assert!(vm.channels[channel_id as usize].buffer.is_empty());
}

#[test]
fn unbuffered_receive_wakes_blocked_senders_in_fifo_order_under_contention() {
    let program = worker_program(2);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(0, Value::int(0));
    let channel_id = channel_id(&channel);

    let mut senders = Vec::new();
    for value in [10, 20, 30] {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("sender goroutine should spawn");
        let index = goroutine_index(&vm, goroutine);
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.set_register_on_goroutine(&program, goroutine, 1, Value::int(value))
            .expect("value register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_send(&program, 0, 1)
            .expect("unbuffered send should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        senders.push(index);
    }

    let receiver = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("receiver goroutine should spawn");
    let receiver_index = goroutine_index(&vm, receiver);
    vm.set_register_on_goroutine(&program, receiver, 0, channel)
        .expect("channel register should be writable");

    for (step, expected) in [(0, 10), (1, 20), (2, 30)] {
        vm.current_goroutine = receiver_index;
        vm.execute_chan_recv(&program, 1, 0)
            .expect("unbuffered receive should wake the oldest sender");
        assert_eq!(register_int(&vm, receiver_index, 1), expected);
        assert_eq!(
            vm.goroutines[senders[step]].status,
            GoroutineStatus::Runnable
        );
    }

    assert!(vm.channels[channel_id as usize].pending_sends.is_empty());
    assert!(vm.channels[channel_id as usize].buffer.is_empty());
}

#[test]
fn unbuffered_send_wakes_blocked_receivers_in_fifo_order_under_contention() {
    let program = worker_program(2);
    let mut vm = Vm::new();
    let channel = vm.alloc_channel_value(0, Value::int(0));
    let channel_id = channel_id(&channel);

    let mut receivers = Vec::new();
    for _ in 0..3 {
        let goroutine = vm
            .spawn_goroutine(&program, program.entry_function, Vec::new())
            .expect("receiver goroutine should spawn");
        let index = goroutine_index(&vm, goroutine);
        vm.set_register_on_goroutine(&program, goroutine, 0, channel.clone())
            .expect("channel register should be writable");
        vm.current_goroutine = index;
        vm.execute_chan_recv(&program, 1, 0)
            .expect("unbuffered receive should block");
        assert_eq!(vm.goroutines[index].status, GoroutineStatus::Blocked);
        receivers.push(index);
    }

    let sender = vm
        .spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("sender goroutine should spawn");
    let sender_index = goroutine_index(&vm, sender);
    vm.set_register_on_goroutine(&program, sender, 0, channel)
        .expect("channel register should be writable");

    for (step, value) in [(0, 70), (1, 80), (2, 90)] {
        vm.set_register_on_goroutine(&program, sender, 1, Value::int(value))
            .expect("value register should be writable");
        vm.current_goroutine = sender_index;
        vm.execute_chan_send(&program, 0, 1)
            .expect("unbuffered send should wake the oldest receiver");
        assert_eq!(
            vm.goroutines[receivers[step]].status,
            GoroutineStatus::Runnable
        );
        assert_eq!(register_int(&vm, receivers[step], 1), value);
    }

    assert!(vm.channels[channel_id as usize]
        .pending_receivers
        .is_empty());
    assert!(vm.channels[channel_id as usize].buffer.is_empty());
}
