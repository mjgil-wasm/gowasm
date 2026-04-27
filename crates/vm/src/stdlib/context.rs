use super::{
    StdlibConstantValue, StdlibFunction, StdlibMethod, StdlibValue, StdlibValueInit,
    CONTEXT_BACKGROUND, CONTEXT_DEADLINE, CONTEXT_DONE, CONTEXT_ERR, CONTEXT_INTERNAL_CANCEL,
    CONTEXT_TODO, CONTEXT_VALUE, CONTEXT_WITH_CANCEL, CONTEXT_WITH_DEADLINE, CONTEXT_WITH_TIMEOUT,
    CONTEXT_WITH_VALUE,
};
use crate::{
    ChannelValue, ContextErrorKind, ContextState, Program, Value, ValueData, Vm, VmError,
    TYPE_CONTEXT, TYPE_EMPTY_STRUCT,
};

const CONTEXT_ID_FIELD: &str = "__context_id";
const CONTEXT_RECEIVER_TYPE: &str = "context.__impl";
const CONTEXT_CANCEL_HELPER_NAME: &str = "__gowasm_context_cancel";

#[derive(Debug, Clone)]
struct ContextParent {
    id: Option<u64>,
    value: Value,
    done_channel_id: Option<u64>,
    deadline_unix_nanos: Option<i64>,
    err: Option<Value>,
}

pub(super) const CONTEXT_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "Canceled",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("context canceled")),
    },
    StdlibValue {
        symbol: "DeadlineExceeded",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("context deadline exceeded")),
    },
];

pub(super) const CONTEXT_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: CONTEXT_BACKGROUND,
        symbol: "Background",
        returns_value: true,
        handler: context_background,
    },
    StdlibFunction {
        id: CONTEXT_TODO,
        symbol: "TODO",
        returns_value: true,
        handler: context_todo,
    },
    StdlibFunction {
        id: CONTEXT_WITH_CANCEL,
        symbol: "WithCancel",
        returns_value: false,
        handler: unsupported_context_multi_result,
    },
    StdlibFunction {
        id: CONTEXT_WITH_DEADLINE,
        symbol: "WithDeadline",
        returns_value: false,
        handler: unsupported_context_multi_result,
    },
    StdlibFunction {
        id: CONTEXT_WITH_TIMEOUT,
        symbol: "WithTimeout",
        returns_value: false,
        handler: unsupported_context_multi_result,
    },
    StdlibFunction {
        id: CONTEXT_WITH_VALUE,
        symbol: "WithValue",
        returns_value: true,
        handler: context_with_value,
    },
    StdlibFunction {
        id: CONTEXT_INTERNAL_CANCEL,
        symbol: "__cancel",
        returns_value: false,
        handler: context_internal_cancel,
    },
];

pub(super) const CONTEXT_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: CONTEXT_RECEIVER_TYPE,
        method: "Deadline",
        function: CONTEXT_DEADLINE,
    },
    StdlibMethod {
        receiver_type: CONTEXT_RECEIVER_TYPE,
        method: "Done",
        function: CONTEXT_DONE,
    },
    StdlibMethod {
        receiver_type: CONTEXT_RECEIVER_TYPE,
        method: "Err",
        function: CONTEXT_ERR,
    },
    StdlibMethod {
        receiver_type: CONTEXT_RECEIVER_TYPE,
        method: "Value",
        function: CONTEXT_VALUE,
    },
];

pub(super) const CONTEXT_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: CONTEXT_DEADLINE,
        symbol: "Deadline",
        returns_value: false,
        handler: unsupported_context_multi_result,
    },
    StdlibFunction {
        id: CONTEXT_DONE,
        symbol: "Done",
        returns_value: true,
        handler: context_done,
    },
    StdlibFunction {
        id: CONTEXT_ERR,
        symbol: "Err",
        returns_value: true,
        handler: context_err,
    },
    StdlibFunction {
        id: CONTEXT_VALUE,
        symbol: "Value",
        returns_value: true,
        handler: context_value_method,
    },
];

pub(super) fn context_background(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    no_arg_context(vm, program, "context.Background", args)
}

pub(super) fn context_todo(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    no_arg_context(vm, program, "context.TODO", args)
}

pub(super) fn context_with_cancel(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    derive_context(vm, program, "context.WithCancel", args, None, None)
}

pub(super) fn context_with_deadline(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let deadline_unix_nanos =
        super::time_impl::extract_time_unix_nanos(vm, program, "context.WithDeadline", &args[1])?;
    derive_context(
        vm,
        program,
        "context.WithDeadline",
        &args[..1],
        Some(deadline_unix_nanos),
        None,
    )
}

pub(super) fn context_with_timeout(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let timeout_nanos = super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "context.WithTimeout",
        &args[1],
    )?;
    let now_unix_nanos =
        super::time_impl::current_time_unix_nanos(vm, program, "context.WithTimeout")?;
    let deadline_unix_nanos = now_unix_nanos
        .checked_add(timeout_nanos)
        .ok_or_else(|| invalid_context_argument(vm, program, "context.WithTimeout"))?;
    derive_context(
        vm,
        program,
        "context.WithTimeout",
        &args[..1],
        Some(deadline_unix_nanos),
        Some(now_unix_nanos),
    )
}

pub(super) fn context_with_value(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let parent = context_parent(vm, program, "context.WithValue", &args[0])?;
    validate_context_value_key(vm, program, &args[1])?;
    let (context_id, context) = alloc_context(
        vm,
        ContextState {
            parent_id: parent.id,
            parent_value: parent.id.is_none().then_some(parent.value.clone()),
            children: Vec::new(),
            done_channel_id: parent.done_channel_id,
            deadline_unix_nanos: parent.deadline_unix_nanos,
            err: parent.err.clone(),
            values: vec![(args[1].clone(), args[2].clone())],
        },
    );
    if let Some(parent_id) = parent.id {
        if let Some(parent_state) = vm.context_values.get_mut(&parent_id) {
            parent_state.children.push(context_id);
        }
    } else if let Some(done_channel_id) = parent.done_channel_id {
        vm.watch_context_done_channel(done_channel_id, context_id, parent.value);
    }
    Ok(context)
}

pub(super) fn context_deadline(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let state = context_state(vm, program, "context.Context.Deadline", &args[0])?;
    let deadline = state.deadline_unix_nanos.unwrap_or(0);
    Ok(vec![
        super::time_impl::time_value(deadline),
        Value::bool(state.deadline_unix_nanos.is_some()),
    ])
}

pub(super) fn context_done(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let state = context_state(vm, program, "context.Context.Done", &args[0])?;
    Ok(match state.done_channel_id {
        Some(channel_id) => Value::channel(channel_id),
        None => Value::nil_channel(),
    })
}

pub(super) fn context_err(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let state = context_state(vm, program, "context.Context.Err", &args[0])?;
    Ok(match state.err {
        Some(err) => err,
        None => Value::nil(),
    })
}

pub(super) fn context_value_method(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let mut current_id = Some(context_id(vm, program, "context.Context.Value", &args[0])?);
    while let Some(id) = current_id {
        let Some(state) = vm.context_values.get(&id) else {
            break;
        };
        if let Some((_, value)) = state.values.iter().rev().find(|(key, _)| key == &args[1]) {
            return Ok(value.clone());
        }
        let parent_id = state.parent_id;
        let parent_value = state.parent_value.clone();
        if let Some(parent_id) = parent_id {
            current_id = Some(parent_id);
            continue;
        }
        if let Some(parent_value) = parent_value {
            return context_parent_value(vm, program, &parent_value, &args[1]);
        }
        current_id = None;
    }
    Ok(Value::nil())
}

pub(super) fn context_internal_cancel(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let context_id = context_id(vm, program, "context.__cancel", &args[0])?;
    let _ = vm.cancel_context_with_reason(program, context_id, ContextErrorKind::Canceled)?;
    Ok(Value::nil())
}

pub(crate) fn context_value_error(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<Value>, VmError> {
    context_parent_err(vm, program, builtin, value)
}

pub(crate) fn context_value_deadline_unix_nanos(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<i64>, VmError> {
    context_parent_deadline(vm, program, builtin, value)
}

fn derive_context(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
    requested_deadline_unix_nanos: Option<i64>,
    current_time_unix_nanos: Option<i64>,
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let parent = context_parent(vm, program, builtin, &args[0])?;
    let effective_deadline = match (parent.deadline_unix_nanos, requested_deadline_unix_nanos) {
        (Some(parent_deadline), Some(requested_deadline)) => {
            Some(parent_deadline.min(requested_deadline))
        }
        (Some(parent_deadline), None) => Some(parent_deadline),
        (None, Some(requested_deadline)) => Some(requested_deadline),
        (None, None) => None,
    };
    let done_channel = vm.alloc_channel_value(0, empty_struct_value());
    let ValueData::Channel(ChannelValue {
        id: Some(done_channel_id),
        ..
    }) = done_channel.data
    else {
        unreachable!("context done channel should allocate a concrete channel");
    };
    let (context_id, context) = alloc_context(
        vm,
        ContextState {
            parent_id: parent.id,
            parent_value: parent.id.is_none().then_some(parent.value.clone()),
            children: Vec::new(),
            done_channel_id: Some(done_channel_id),
            deadline_unix_nanos: effective_deadline,
            err: None,
            values: Vec::new(),
        },
    );
    if let Some(parent_id) = parent.id {
        if let Some(parent_state) = vm.context_values.get_mut(&parent_id) {
            parent_state.children.push(context_id);
        }
    } else if let Some(parent_done_channel_id) = parent.done_channel_id {
        vm.watch_context_done_channel(parent_done_channel_id, context_id, parent.value.clone());
    }
    let cancel = cancel_func_value(vm, program, context.clone())?;

    if let Some(error) = parent.err {
        let _ = vm.cancel_context_with_error(program, context_id, error)?;
        return Ok(vec![context, cancel]);
    }

    if let Some(deadline_unix_nanos) = effective_deadline {
        let inherits_parent_deadline = parent.deadline_unix_nanos == Some(deadline_unix_nanos);
        if !inherits_parent_deadline {
            let now_unix_nanos = match current_time_unix_nanos {
                Some(unix_nanos) => unix_nanos,
                None => super::time_impl::current_time_unix_nanos(vm, program, builtin)?,
            };
            if deadline_unix_nanos <= now_unix_nanos {
                let _ = vm.cancel_context_with_reason(
                    program,
                    context_id,
                    ContextErrorKind::DeadlineExceeded,
                )?;
            } else {
                vm.schedule_context_deadline(context_id, deadline_unix_nanos - now_unix_nanos);
            }
        }
    }

    Ok(vec![context, cancel])
}

fn no_arg_context(
    vm: &mut Vm,
    program: &Program,
    _builtin: &str,
    args: &[Value],
) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(alloc_context(vm, ContextState::default()).1)
}

fn alloc_context(vm: &mut Vm, state: ContextState) -> (u64, Value) {
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.context_values.insert(id, state);
    (
        id,
        Value::struct_value(
            TYPE_CONTEXT,
            vec![(CONTEXT_ID_FIELD.into(), Value::int(id as i64))],
        ),
    )
}

fn cancel_func_value(vm: &Vm, program: &Program, context: Value) -> Result<Value, VmError> {
    let function = program
        .functions
        .iter()
        .rposition(|function| function.name == CONTEXT_CANCEL_HELPER_NAME)
        .ok_or_else(|| invalid_context_argument(vm, program, "context.CancelFunc"))?;
    Ok(Value::function(function, vec![context]))
}

fn context_state(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<ContextState, VmError> {
    let id = context_id(vm, program, builtin, value)?;
    vm.context_values
        .get(&id)
        .cloned()
        .ok_or_else(|| invalid_context_argument(vm, program, builtin))
}

fn context_parent(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<ContextParent, VmError> {
    if matches!(&value.data, ValueData::Nil) {
        return Err(context_panic(
            vm,
            program,
            "cannot create context from nil parent",
        ));
    }
    if let Some(id) = try_context_id(value) {
        let state = vm
            .context_values
            .get(&id)
            .cloned()
            .ok_or_else(|| invalid_context_argument(vm, program, builtin))?;
        return Ok(ContextParent {
            id: Some(id),
            value: value.clone(),
            done_channel_id: state.done_channel_id,
            deadline_unix_nanos: state.deadline_unix_nanos,
            err: state.err,
        });
    }
    Ok(ContextParent {
        id: None,
        value: value.clone(),
        done_channel_id: context_parent_done_channel(vm, program, builtin, value)?,
        deadline_unix_nanos: context_parent_deadline(vm, program, builtin, value)?,
        err: context_parent_err(vm, program, builtin, value)?,
    })
}

fn context_id(vm: &Vm, program: &Program, builtin: &str, value: &Value) -> Result<u64, VmError> {
    try_context_id(value).ok_or_else(|| invalid_context_argument(vm, program, builtin))
}

fn try_context_id(value: &Value) -> Option<u64> {
    if value.typ != TYPE_CONTEXT {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    fields
        .iter()
        .find(|(name, _)| name == CONTEXT_ID_FIELD)
        .and_then(|(_, value)| match value.data {
            ValueData::Int(id) if id > 0 => Some(id as u64),
            _ => None,
        })
}

fn empty_struct_value() -> Value {
    Value::struct_value(TYPE_EMPTY_STRUCT, Vec::new())
}

fn validate_context_value_key(vm: &Vm, program: &Program, key: &Value) -> Result<(), VmError> {
    if matches!(&key.data, ValueData::Nil) {
        return Err(context_panic(vm, program, "nil key"));
    }
    if !is_comparable_context_key(key) {
        return Err(context_panic(vm, program, "key is not comparable"));
    }
    Ok(())
}

fn is_comparable_context_key(value: &Value) -> bool {
    match &value.data {
        ValueData::Nil => false,
        ValueData::Int(_)
        | ValueData::Float(_)
        | ValueData::String(_)
        | ValueData::Bool(_)
        | ValueData::Pointer(_)
        | ValueData::Channel(_) => true,
        ValueData::Error(error) => match error.wrapped.as_deref() {
            Some(wrapped) => is_comparable_context_key(wrapped),
            None => true,
        },
        ValueData::Array(array) => array
            .values_snapshot()
            .iter()
            .all(is_comparable_context_key),
        ValueData::Struct(fields) => fields
            .iter()
            .all(|(_, value)| is_comparable_context_key(value)),
        ValueData::Slice(_) | ValueData::Map(_) | ValueData::Function(_) => false,
    }
}

fn invalid_context_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a value implementing context.Context".into(),
    }
}

fn context_panic(vm: &Vm, program: &Program, value: &str) -> VmError {
    VmError::UnhandledPanic {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        value: value.into(),
    }
}

fn unsupported_context_multi_result(
    _vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::nil())
}

fn context_parent_done_channel(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<u64>, VmError> {
    let done = vm.invoke_method(program, value.clone(), "Done", Vec::new())?;
    match &done.data {
        ValueData::Channel(ChannelValue { id, .. }) => Ok(*id),
        _ => Err(invalid_context_argument(vm, program, builtin)),
    }
}

fn context_parent_deadline(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<i64>, VmError> {
    let results = vm.invoke_method_results(program, value.clone(), "Deadline", Vec::new())?;
    if results.len() != 2 {
        return Err(invalid_context_argument(vm, program, builtin));
    }
    let ok = match &results[1].data {
        ValueData::Bool(ok) => *ok,
        _ => return Err(invalid_context_argument(vm, program, builtin)),
    };
    if !ok {
        return Ok(None);
    }
    Ok(Some(super::time_impl::extract_time_unix_nanos(
        vm,
        program,
        builtin,
        &results[0],
    )?))
}

fn context_parent_err(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<Value>, VmError> {
    let err = vm.invoke_method(program, value.clone(), "Err", Vec::new())?;
    match &err.data {
        ValueData::Nil => Ok(None),
        ValueData::Error(_) => Ok(Some(err)),
        _ => Err(invalid_context_argument(vm, program, builtin)),
    }
}

fn context_parent_value(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    key: &Value,
) -> Result<Value, VmError> {
    vm.invoke_method(program, value.clone(), "Value", vec![key.clone()])
}
