use super::{
    StdlibFunction, StdlibMethod, SYNC_MUTEX_LOCK, SYNC_MUTEX_UNLOCK, SYNC_ONCE_DO,
    SYNC_RW_MUTEX_LOCK, SYNC_RW_MUTEX_RLOCK, SYNC_RW_MUTEX_RUNLOCK, SYNC_RW_MUTEX_UNLOCK,
    SYNC_WAIT_GROUP_ADD, SYNC_WAIT_GROUP_DONE, SYNC_WAIT_GROUP_WAIT,
};
use crate::{
    FunctionValue, MutexState, OnceState, Program, RwMutexState, Value, ValueData, Vm, VmError,
    WaitGroupState, TYPE_SYNC_MUTEX, TYPE_SYNC_ONCE, TYPE_SYNC_RW_MUTEX, TYPE_SYNC_WAIT_GROUP,
};

const WAIT_GROUP_ID_FIELD: &str = "__sync_wait_group_id";
const ONCE_ID_FIELD: &str = "__sync_once_id";
const MUTEX_ID_FIELD: &str = "__sync_mutex_id";
const RW_MUTEX_ID_FIELD: &str = "__sync_rw_mutex_id";

pub(super) const SYNC_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "*sync.WaitGroup",
        method: "Add",
        function: SYNC_WAIT_GROUP_ADD,
    },
    StdlibMethod {
        receiver_type: "*sync.WaitGroup",
        method: "Done",
        function: SYNC_WAIT_GROUP_DONE,
    },
    StdlibMethod {
        receiver_type: "*sync.WaitGroup",
        method: "Wait",
        function: SYNC_WAIT_GROUP_WAIT,
    },
    StdlibMethod {
        receiver_type: "*sync.Once",
        method: "Do",
        function: SYNC_ONCE_DO,
    },
    StdlibMethod {
        receiver_type: "*sync.Mutex",
        method: "Lock",
        function: SYNC_MUTEX_LOCK,
    },
    StdlibMethod {
        receiver_type: "*sync.Mutex",
        method: "Unlock",
        function: SYNC_MUTEX_UNLOCK,
    },
    StdlibMethod {
        receiver_type: "*sync.RWMutex",
        method: "Lock",
        function: SYNC_RW_MUTEX_LOCK,
    },
    StdlibMethod {
        receiver_type: "*sync.RWMutex",
        method: "Unlock",
        function: SYNC_RW_MUTEX_UNLOCK,
    },
    StdlibMethod {
        receiver_type: "*sync.RWMutex",
        method: "RLock",
        function: SYNC_RW_MUTEX_RLOCK,
    },
    StdlibMethod {
        receiver_type: "*sync.RWMutex",
        method: "RUnlock",
        function: SYNC_RW_MUTEX_RUNLOCK,
    },
];

pub(super) const SYNC_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: SYNC_WAIT_GROUP_ADD,
        symbol: "Add",
        returns_value: false,
        handler: sync_wait_group_add,
    },
    StdlibFunction {
        id: SYNC_WAIT_GROUP_DONE,
        symbol: "Done",
        returns_value: false,
        handler: sync_wait_group_done,
    },
    StdlibFunction {
        id: SYNC_WAIT_GROUP_WAIT,
        symbol: "Wait",
        returns_value: false,
        handler: sync_wait_group_wait,
    },
    StdlibFunction {
        id: SYNC_ONCE_DO,
        symbol: "Do",
        returns_value: false,
        handler: sync_once_do,
    },
    StdlibFunction {
        id: SYNC_MUTEX_LOCK,
        symbol: "Lock",
        returns_value: false,
        handler: sync_mutex_lock,
    },
    StdlibFunction {
        id: SYNC_MUTEX_UNLOCK,
        symbol: "Unlock",
        returns_value: false,
        handler: sync_mutex_unlock,
    },
    StdlibFunction {
        id: SYNC_RW_MUTEX_LOCK,
        symbol: "Lock",
        returns_value: false,
        handler: sync_rw_mutex_lock,
    },
    StdlibFunction {
        id: SYNC_RW_MUTEX_UNLOCK,
        symbol: "Unlock",
        returns_value: false,
        handler: sync_rw_mutex_unlock,
    },
    StdlibFunction {
        id: SYNC_RW_MUTEX_RLOCK,
        symbol: "RLock",
        returns_value: false,
        handler: sync_rw_mutex_rlock,
    },
    StdlibFunction {
        id: SYNC_RW_MUTEX_RUNLOCK,
        symbol: "RUnlock",
        returns_value: false,
        handler: sync_rw_mutex_runlock,
    },
];

fn sync_wait_group_add(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let delta = int_arg(vm, program, "sync.(*WaitGroup).Add", &args[1])?;
    let wait_group_id = ensure_wait_group_id(vm, program, &args[0])?;
    let (negative_counter, waiters) = {
        let state = vm.wait_groups.entry(wait_group_id).or_default();
        state.count += delta;
        if state.count < 0 {
            (true, Vec::new())
        } else if state.count == 0 {
            (false, std::mem::take(&mut state.waiters))
        } else {
            (false, Vec::new())
        }
    };
    if negative_counter {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "sync: negative WaitGroup counter".into(),
        });
    }
    wake_waiters(vm, waiters);
    Ok(Value::nil())
}

fn sync_wait_group_done(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    sync_wait_group_add(vm, program, &[args[0].clone(), Value::int(-1)])
}

fn sync_wait_group_wait(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let wait_group_id = ensure_wait_group_id(vm, program, &args[0])?;
    let current = vm.current_goroutine_id();
    let should_block = {
        let state = vm.wait_groups.entry(wait_group_id).or_default();
        if state.count == 0 {
            false
        } else {
            push_unique_waiter(&mut state.waiters, current);
            true
        }
    };
    if should_block {
        vm.block_current_goroutine();
    }
    Ok(Value::nil())
}

fn sync_once_do(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let once_id = ensure_once_id(vm, program, &args[0])?;
    let callback = function_arg(vm, program, "sync.(*Once).Do", &args[1])?;
    let current = vm.current_goroutine_id();
    enum OnceAction {
        Return,
        Block,
        Call(FunctionValue),
    }
    let action = {
        let state = vm.once_values.entry(once_id).or_default();
        if state.done {
            OnceAction::Return
        } else if state.running {
            push_unique_waiter(&mut state.waiters, current);
            OnceAction::Block
        } else {
            state.running = true;
            OnceAction::Call(callback)
        }
    };

    match action {
        OnceAction::Return => Ok(Value::nil()),
        OnceAction::Block => {
            vm.block_current_goroutine();
            Ok(Value::nil())
        }
        OnceAction::Call(callback) => {
            let result = vm.invoke_callback_no_result_or_panic(
                program,
                callback.function,
                callback.captures.clone(),
            );
            let waiters = {
                let state = vm.once_values.entry(once_id).or_default();
                state.running = false;
                state.done = true;
                std::mem::take(&mut state.waiters)
            };
            wake_waiters(vm, waiters);
            match result? {
                Ok(()) => Ok(Value::nil()),
                Err(value) => {
                    vm.propagate_panic_value(program, value)?;
                    Ok(Value::nil())
                }
            }
        }
    }
}

fn sync_mutex_lock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let mutex_id = ensure_mutex_id(vm, program, &args[0])?;
    let current = vm.current_goroutine_id();
    let should_block = {
        let state = vm.mutex_values.entry(mutex_id).or_default();
        if state.locked {
            push_unique_waiter(&mut state.waiters, current);
            true
        } else {
            state.locked = true;
            false
        }
    };
    if should_block {
        vm.block_current_goroutine();
    }
    Ok(Value::nil())
}

fn sync_mutex_unlock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let mutex_id = ensure_mutex_id(vm, program, &args[0])?;
    enum UnlockAction {
        Panic,
        Release,
        Wake(crate::GoroutineId),
    }
    let action = {
        let state = vm.mutex_values.entry(mutex_id).or_default();
        if !state.locked {
            UnlockAction::Panic
        } else if state.waiters.is_empty() {
            state.locked = false;
            UnlockAction::Release
        } else {
            UnlockAction::Wake(state.waiters.remove(0))
        }
    };
    match action {
        UnlockAction::Panic => Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "sync: unlock of unlocked mutex".into(),
        }),
        UnlockAction::Release => Ok(Value::nil()),
        UnlockAction::Wake(goroutine) => {
            vm.wake_goroutine(goroutine);
            Ok(Value::nil())
        }
    }
}

fn sync_rw_mutex_lock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let rw_mutex_id = ensure_rw_mutex_id(vm, program, &args[0])?;
    let current = vm.current_goroutine_id();
    let should_block = {
        let state = vm.rw_mutex_values.entry(rw_mutex_id).or_default();
        if state.writer_active || state.reader_count > 0 {
            push_unique_waiter(&mut state.writer_waiters, current);
            true
        } else {
            state.writer_active = true;
            false
        }
    };
    if should_block {
        vm.block_current_goroutine();
    }
    Ok(Value::nil())
}

fn sync_rw_mutex_unlock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let rw_mutex_id = ensure_rw_mutex_id(vm, program, &args[0])?;
    enum UnlockAction {
        Panic,
        Release,
        WakeWriter(crate::GoroutineId),
        WakeReaders(Vec<crate::GoroutineId>),
    }
    let action = {
        let state = vm.rw_mutex_values.entry(rw_mutex_id).or_default();
        if !state.writer_active {
            UnlockAction::Panic
        } else if !state.writer_waiters.is_empty() {
            UnlockAction::WakeWriter(state.writer_waiters.remove(0))
        } else if state.reader_waiters.is_empty() {
            state.writer_active = false;
            UnlockAction::Release
        } else {
            let waiters = std::mem::take(&mut state.reader_waiters);
            state.writer_active = false;
            state.reader_count = waiters.len();
            UnlockAction::WakeReaders(waiters)
        }
    };
    match action {
        UnlockAction::Panic => Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "sync: Unlock of unlocked RWMutex".into(),
        }),
        UnlockAction::Release => Ok(Value::nil()),
        UnlockAction::WakeWriter(goroutine) => {
            vm.wake_goroutine(goroutine);
            Ok(Value::nil())
        }
        UnlockAction::WakeReaders(waiters) => {
            wake_waiters(vm, waiters);
            Ok(Value::nil())
        }
    }
}

fn sync_rw_mutex_rlock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let rw_mutex_id = ensure_rw_mutex_id(vm, program, &args[0])?;
    let current = vm.current_goroutine_id();
    let should_block = {
        let state = vm.rw_mutex_values.entry(rw_mutex_id).or_default();
        if state.writer_active || !state.writer_waiters.is_empty() {
            push_unique_waiter(&mut state.reader_waiters, current);
            true
        } else {
            state.reader_count += 1;
            false
        }
    };
    if should_block {
        vm.block_current_goroutine();
    }
    Ok(Value::nil())
}

fn sync_rw_mutex_runlock(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let rw_mutex_id = ensure_rw_mutex_id(vm, program, &args[0])?;
    enum RUnlockAction {
        Panic,
        Release,
        WakeWriter(crate::GoroutineId),
    }
    let action = {
        let state = vm.rw_mutex_values.entry(rw_mutex_id).or_default();
        if state.reader_count == 0 {
            RUnlockAction::Panic
        } else {
            state.reader_count -= 1;
            if state.reader_count == 0 && !state.writer_waiters.is_empty() {
                state.writer_active = true;
                RUnlockAction::WakeWriter(state.writer_waiters.remove(0))
            } else {
                RUnlockAction::Release
            }
        }
    };
    match action {
        RUnlockAction::Panic => Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "sync: RUnlock of unlocked RWMutex".into(),
        }),
        RUnlockAction::Release => Ok(Value::nil()),
        RUnlockAction::WakeWriter(goroutine) => {
            vm.wake_goroutine(goroutine);
            Ok(Value::nil())
        }
    }
}

fn ensure_wait_group_id(vm: &mut Vm, program: &Program, receiver: &Value) -> Result<u64, VmError> {
    ensure_sync_object_id(
        vm,
        program,
        receiver,
        TYPE_SYNC_WAIT_GROUP,
        WAIT_GROUP_ID_FIELD,
        "sync.(*WaitGroup)",
        |vm, id| {
            vm.wait_groups.insert(id, WaitGroupState::default());
        },
    )
}

fn ensure_once_id(vm: &mut Vm, program: &Program, receiver: &Value) -> Result<u64, VmError> {
    ensure_sync_object_id(
        vm,
        program,
        receiver,
        TYPE_SYNC_ONCE,
        ONCE_ID_FIELD,
        "sync.(*Once)",
        |vm, id| {
            vm.once_values.insert(id, OnceState::default());
        },
    )
}

fn ensure_mutex_id(vm: &mut Vm, program: &Program, receiver: &Value) -> Result<u64, VmError> {
    ensure_sync_object_id(
        vm,
        program,
        receiver,
        TYPE_SYNC_MUTEX,
        MUTEX_ID_FIELD,
        "sync.(*Mutex)",
        |vm, id| {
            vm.mutex_values.insert(id, MutexState::default());
        },
    )
}

fn ensure_rw_mutex_id(vm: &mut Vm, program: &Program, receiver: &Value) -> Result<u64, VmError> {
    ensure_sync_object_id(
        vm,
        program,
        receiver,
        TYPE_SYNC_RW_MUTEX,
        RW_MUTEX_ID_FIELD,
        "sync.(*RWMutex)",
        |vm, id| {
            vm.rw_mutex_values.insert(id, RwMutexState::default());
        },
    )
}

fn ensure_sync_object_id<F>(
    vm: &mut Vm,
    program: &Program,
    receiver: &Value,
    expected_type: crate::TypeId,
    id_field: &str,
    builtin: &str,
    initialize: F,
) -> Result<u64, VmError>
where
    F: FnOnce(&mut Vm, u64),
{
    let current = vm.deref_pointer(program, receiver)?;
    if current.typ != expected_type {
        return Err(invalid_sync_argument(
            vm,
            program,
            builtin,
            "a valid sync receiver",
        ));
    }
    let ValueData::Struct(mut fields) = current.data else {
        return Err(invalid_sync_argument(
            vm,
            program,
            builtin,
            "a valid sync receiver",
        ));
    };
    if let Some(id) = hidden_int_field(&fields, id_field) {
        return Ok(id);
    }

    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    initialize(vm, id);
    set_hidden_field(&mut fields, id_field, Value::int(id as i64));
    vm.store_indirect(
        program,
        receiver,
        Value {
            typ: expected_type,
            data: ValueData::Struct(fields),
        },
    )?;
    Ok(id)
}

fn hidden_int_field(fields: &[(String, Value)], field: &str) -> Option<u64> {
    fields.iter().find_map(|(name, value)| {
        if name != field {
            return None;
        }
        match &value.data {
            ValueData::Int(id) if *id > 0 => Some(*id as u64),
            _ => None,
        }
    })
}

fn set_hidden_field(fields: &mut Vec<(String, Value)>, field: &str, value: Value) {
    if let Some((_, slot)) = fields.iter_mut().find(|(name, _)| name == field) {
        *slot = value;
        return;
    }
    fields.push((field.into(), value));
}

fn push_unique_waiter(waiters: &mut Vec<crate::GoroutineId>, goroutine: crate::GoroutineId) {
    if !waiters.contains(&goroutine) {
        waiters.push(goroutine);
    }
}

fn wake_waiters(vm: &mut Vm, waiters: Vec<crate::GoroutineId>) {
    for goroutine in waiters {
        vm.wake_goroutine(goroutine);
    }
}

fn int_arg(vm: &Vm, program: &Program, builtin: &str, value: &Value) -> Result<i64, VmError> {
    match &value.data {
        ValueData::Int(delta) => Ok(*delta),
        _ => Err(invalid_sync_argument(
            vm,
            program,
            builtin,
            "an int argument",
        )),
    }
}

fn function_arg(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<FunctionValue, VmError> {
    match &value.data {
        ValueData::Function(function) => Ok(function.clone()),
        _ => Err(invalid_sync_argument(
            vm,
            program,
            builtin,
            "a func() argument",
        )),
    }
}

fn invalid_sync_argument(vm: &Vm, program: &Program, builtin: &str, expected: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: expected.into(),
    }
}
