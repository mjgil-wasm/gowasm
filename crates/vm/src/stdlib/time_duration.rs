use crate::{Program, Value, ValueData, Vm, VmError, TYPE_TIME_DURATION};

const MICROSECOND: i64 = 1_000;
const MILLISECOND: i64 = 1_000_000;
const SECOND: i64 = 1_000_000_000;
const MINUTE: i64 = 60 * SECOND;
const HOUR: i64 = 60 * MINUTE;

pub(super) fn time_duration_nanoseconds(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::int(extract_duration_nanos(
        vm,
        program,
        "time.Duration.Nanoseconds",
        single_duration_arg(vm, program, args)?,
    )?))
}

pub(super) fn time_duration_microseconds(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::int(
        extract_duration_nanos(
            vm,
            program,
            "time.Duration.Microseconds",
            single_duration_arg(vm, program, args)?,
        )? / MICROSECOND,
    ))
}

pub(super) fn time_duration_milliseconds(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::int(
        extract_duration_nanos(
            vm,
            program,
            "time.Duration.Milliseconds",
            single_duration_arg(vm, program, args)?,
        )? / MILLISECOND,
    ))
}

pub(super) fn time_duration_seconds(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::float(
        extract_duration_nanos(
            vm,
            program,
            "time.Duration.Seconds",
            single_duration_arg(vm, program, args)?,
        )? as f64
            / SECOND as f64,
    ))
}

pub(super) fn time_duration_minutes(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::float(
        extract_duration_nanos(
            vm,
            program,
            "time.Duration.Minutes",
            single_duration_arg(vm, program, args)?,
        )? as f64
            / MINUTE as f64,
    ))
}

pub(super) fn time_duration_hours(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::float(
        extract_duration_nanos(
            vm,
            program,
            "time.Duration.Hours",
            single_duration_arg(vm, program, args)?,
        )? as f64
            / HOUR as f64,
    ))
}

pub(super) fn duration_value(duration_nanos: i64) -> Value {
    Value {
        typ: TYPE_TIME_DURATION,
        data: ValueData::Int(duration_nanos),
    }
}

pub(super) fn single_duration_arg<'a>(
    vm: &Vm,
    program: &Program,
    args: &'a [Value],
) -> Result<&'a Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(&args[0])
}

pub(super) fn extract_duration_nanos(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<i64, VmError> {
    if value.typ != TYPE_TIME_DURATION {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a time.Duration receiver".into(),
        });
    }
    let ValueData::Int(duration_nanos) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a time.Duration receiver".into(),
        });
    };
    Ok(*duration_nanos)
}
