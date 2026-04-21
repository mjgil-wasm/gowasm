use crate::{
    ChannelValue, Program, Value, ValueData, Vm, VmError, TYPE_TIME_TIMER, TYPE_TIME_TIMER_PTR,
};

const TIMER_CHANNEL_FIELD: &str = "__time_timer_channel";

pub(super) fn time_after(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let duration_nanos = super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "time.After",
        super::time_duration_impl::single_duration_arg(vm, program, args)?,
    )?;
    timer_channel_for_duration(vm, program, "time.After", duration_nanos)
}

pub(super) fn time_new_timer(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let duration_nanos = super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "time.NewTimer",
        super::time_duration_impl::single_duration_arg(vm, program, args)?,
    )?;
    let channel = timer_channel_for_duration(vm, program, "time.NewTimer", duration_nanos)?;
    Ok(timer_value(vm, channel))
}

pub(super) fn time_timer_stop(
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
    let Some(channel_id) = extract_timer_channel_id(vm, program, "time.(*Timer).Stop", &args[0])?
    else {
        return Ok(Value::bool(false));
    };
    Ok(Value::bool(vm.cancel_time_channel_send(channel_id)))
}

pub(super) fn time_timer_reset(
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
    let duration_nanos = super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "time.(*Timer).Reset",
        &args[1],
    )?;
    let Some(channel_id) = extract_timer_channel_id(vm, program, "time.(*Timer).Reset", &args[0])?
    else {
        return Ok(Value::bool(false));
    };

    let fired_at_unix_nanos = if duration_nanos <= 0 {
        Some(super::time_impl::current_time_unix_nanos(
            vm,
            program,
            "time.(*Timer).Reset",
        )?)
    } else {
        None
    };

    Ok(Value::bool(vm.reset_time_channel_send(
        program,
        channel_id,
        duration_nanos,
        fired_at_unix_nanos,
    )?))
}

fn timer_channel_for_duration(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    duration_nanos: i64,
) -> Result<Value, VmError> {
    if duration_nanos <= 0 {
        let unix_nanos = super::time_impl::current_time_unix_nanos(vm, program, builtin)?;
        let channel = vm.alloc_channel_value(1, super::time_impl::time_value(0));
        let ValueData::Channel(channel_value) = &channel.data else {
            unreachable!("alloc_channel_value should return a channel");
        };
        vm.send_to_channel_value(
            program,
            channel_value
                .id
                .expect("allocated timer channel should have an id"),
            super::time_impl::time_value(unix_nanos),
        )?;
        return Ok(channel);
    }

    let channel = vm.alloc_channel_value(1, super::time_impl::time_value(0));
    let ValueData::Channel(channel_value) = &channel.data else {
        unreachable!("alloc_channel_value should return a channel");
    };
    vm.schedule_time_channel_send(
        channel_value
            .id
            .expect("allocated timer channel should have an id"),
        duration_nanos,
    );
    Ok(channel)
}

fn timer_value(vm: &mut Vm, channel: Value) -> Value {
    let timer = Value::struct_value(
        TYPE_TIME_TIMER,
        vec![
            ("C".into(), channel.clone()),
            (TIMER_CHANNEL_FIELD.into(), channel),
        ],
    );
    vm.box_heap_value(timer, TYPE_TIME_TIMER_PTR)
}

fn extract_timer_channel_id(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<Option<u64>, VmError> {
    let current = vm.deref_pointer(program, receiver)?;
    if current.typ != TYPE_TIME_TIMER {
        return Err(invalid_time_argument(
            vm,
            program,
            builtin,
            "a valid time.Timer receiver",
        ));
    }
    let ValueData::Struct(fields) = &current.data else {
        return Err(invalid_time_argument(
            vm,
            program,
            builtin,
            "a valid time.Timer receiver",
        ));
    };

    Ok(field_channel_id(fields, TIMER_CHANNEL_FIELD).or_else(|| field_channel_id(fields, "C")))
}

fn field_channel_id(fields: &[(String, Value)], field: &str) -> Option<u64> {
    fields
        .iter()
        .find(|(name, _)| name == field)
        .and_then(|(_, value)| match &value.data {
            ValueData::Channel(ChannelValue { id, .. }) => *id,
            _ => None,
        })
}

fn invalid_time_argument(vm: &Vm, program: &Program, builtin: &str, expected: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| builtin.into()),
        builtin: builtin.into(),
        expected: expected.into(),
    }
}
