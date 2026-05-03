use super::super::{
    unsupported_multi_result_stdlib, StdlibConstant, StdlibConstantValue, StdlibFunction,
    StdlibMethod, TIME_AFTER, TIME_DURATION_HOURS, TIME_DURATION_MICROSECONDS,
    TIME_DURATION_MILLISECONDS, TIME_DURATION_MINUTES, TIME_DURATION_NANOSECONDS,
    TIME_DURATION_SECONDS, TIME_NEW_TIMER, TIME_NOW, TIME_PARSE, TIME_SINCE, TIME_SLEEP,
    TIME_TIMER_RESET, TIME_TIMER_STOP, TIME_TIME_ADD, TIME_TIME_AFTER, TIME_TIME_BEFORE,
    TIME_TIME_COMPARE, TIME_TIME_EQUAL, TIME_TIME_FORMAT, TIME_TIME_IS_ZERO, TIME_TIME_SUB,
    TIME_TIME_UNIX, TIME_TIME_UNIX_MICRO, TIME_TIME_UNIX_MILLI, TIME_TIME_UNIX_NANO, TIME_UNIX,
    TIME_UNIX_MICRO, TIME_UNIX_MILLI, TIME_UNTIL,
};
use crate::{Program, Value, ValueData, Vm, VmError, TYPE_TIME};

const TIME_UNIX_NANOS_FIELD: &str = "__time_unix_nanos";
const NANOSECOND: i64 = 1;
const MICROSECOND: i64 = 1_000;
const MILLISECOND: i64 = 1_000_000;
const SECOND: i64 = 1_000_000_000;
const MINUTE: i64 = 60 * SECOND;
const HOUR: i64 = 60 * MINUTE;

pub(crate) const TIME_CONSTANTS: &[StdlibConstant] = &[
    StdlibConstant {
        symbol: "Nanosecond",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(NANOSECOND),
    },
    StdlibConstant {
        symbol: "Microsecond",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(MICROSECOND),
    },
    StdlibConstant {
        symbol: "Millisecond",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(MILLISECOND),
    },
    StdlibConstant {
        symbol: "Second",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(SECOND),
    },
    StdlibConstant {
        symbol: "Minute",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(MINUTE),
    },
    StdlibConstant {
        symbol: "Hour",
        typ: "time.Duration",
        value: StdlibConstantValue::Int(HOUR),
    },
    StdlibConstant {
        symbol: "DateTime",
        typ: "string",
        value: StdlibConstantValue::String(super::super::time_format_impl::DATE_TIME_LAYOUT),
    },
    StdlibConstant {
        symbol: "ANSIC",
        typ: "string",
        value: StdlibConstantValue::String(super::parse::ANSIC_LAYOUT),
    },
    StdlibConstant {
        symbol: "RFC850",
        typ: "string",
        value: StdlibConstantValue::String(super::parse::RFC850_LAYOUT),
    },
    StdlibConstant {
        symbol: "RFC1123",
        typ: "string",
        value: StdlibConstantValue::String(super::parse::RFC1123_LAYOUT),
    },
    StdlibConstant {
        symbol: "RFC1123Z",
        typ: "string",
        value: StdlibConstantValue::String(super::parse::RFC1123Z_LAYOUT),
    },
    StdlibConstant {
        symbol: "RFC3339",
        typ: "string",
        value: StdlibConstantValue::String(super::parse::RFC3339_LAYOUT),
    },
];

pub(crate) const TIME_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: TIME_NOW,
        symbol: "Now",
        returns_value: true,
        handler: time_now,
    },
    StdlibFunction {
        id: TIME_UNIX,
        symbol: "Unix",
        returns_value: true,
        handler: time_unix,
    },
    StdlibFunction {
        id: TIME_UNIX_MILLI,
        symbol: "UnixMilli",
        returns_value: true,
        handler: time_unix_milli,
    },
    StdlibFunction {
        id: TIME_UNIX_MICRO,
        symbol: "UnixMicro",
        returns_value: true,
        handler: time_unix_micro,
    },
    StdlibFunction {
        id: TIME_SINCE,
        symbol: "Since",
        returns_value: true,
        handler: time_since,
    },
    StdlibFunction {
        id: TIME_UNTIL,
        symbol: "Until",
        returns_value: true,
        handler: time_until,
    },
    StdlibFunction {
        id: TIME_PARSE,
        symbol: "Parse",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: TIME_SLEEP,
        symbol: "Sleep",
        returns_value: false,
        handler: time_sleep,
    },
    StdlibFunction {
        id: TIME_AFTER,
        symbol: "After",
        returns_value: true,
        handler: super::super::time_timer_impl::time_after,
    },
    StdlibFunction {
        id: TIME_NEW_TIMER,
        symbol: "NewTimer",
        returns_value: true,
        handler: super::super::time_timer_impl::time_new_timer,
    },
];

pub(crate) const TIME_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Unix",
        function: TIME_TIME_UNIX,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "UnixMilli",
        function: TIME_TIME_UNIX_MILLI,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "UnixMicro",
        function: TIME_TIME_UNIX_MICRO,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "UnixNano",
        function: TIME_TIME_UNIX_NANO,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Before",
        function: TIME_TIME_BEFORE,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "After",
        function: TIME_TIME_AFTER,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Equal",
        function: TIME_TIME_EQUAL,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "IsZero",
        function: TIME_TIME_IS_ZERO,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Compare",
        function: TIME_TIME_COMPARE,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Add",
        function: TIME_TIME_ADD,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Sub",
        function: TIME_TIME_SUB,
    },
    StdlibMethod {
        receiver_type: "time.Time",
        method: "Format",
        function: TIME_TIME_FORMAT,
    },
    StdlibMethod {
        receiver_type: "*time.Timer",
        method: "Stop",
        function: TIME_TIMER_STOP,
    },
    StdlibMethod {
        receiver_type: "*time.Timer",
        method: "Reset",
        function: TIME_TIMER_RESET,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Nanoseconds",
        function: TIME_DURATION_NANOSECONDS,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Microseconds",
        function: TIME_DURATION_MICROSECONDS,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Milliseconds",
        function: TIME_DURATION_MILLISECONDS,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Seconds",
        function: TIME_DURATION_SECONDS,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Minutes",
        function: TIME_DURATION_MINUTES,
    },
    StdlibMethod {
        receiver_type: "time.Duration",
        method: "Hours",
        function: TIME_DURATION_HOURS,
    },
];

pub(crate) const TIME_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: TIME_TIME_UNIX,
        symbol: "Unix",
        returns_value: true,
        handler: time_time_unix,
    },
    StdlibFunction {
        id: TIME_TIME_UNIX_MILLI,
        symbol: "UnixMilli",
        returns_value: true,
        handler: time_time_unix_milli,
    },
    StdlibFunction {
        id: TIME_TIME_UNIX_MICRO,
        symbol: "UnixMicro",
        returns_value: true,
        handler: time_time_unix_micro,
    },
    StdlibFunction {
        id: TIME_TIME_UNIX_NANO,
        symbol: "UnixNano",
        returns_value: true,
        handler: time_time_unix_nano,
    },
    StdlibFunction {
        id: TIME_TIME_BEFORE,
        symbol: "Before",
        returns_value: true,
        handler: time_time_before,
    },
    StdlibFunction {
        id: TIME_TIME_AFTER,
        symbol: "After",
        returns_value: true,
        handler: time_time_after,
    },
    StdlibFunction {
        id: TIME_TIME_EQUAL,
        symbol: "Equal",
        returns_value: true,
        handler: time_time_equal,
    },
    StdlibFunction {
        id: TIME_TIME_IS_ZERO,
        symbol: "IsZero",
        returns_value: true,
        handler: time_time_is_zero,
    },
    StdlibFunction {
        id: TIME_TIME_COMPARE,
        symbol: "Compare",
        returns_value: true,
        handler: time_time_compare,
    },
    StdlibFunction {
        id: TIME_TIME_ADD,
        symbol: "Add",
        returns_value: true,
        handler: time_time_add,
    },
    StdlibFunction {
        id: TIME_TIME_SUB,
        symbol: "Sub",
        returns_value: true,
        handler: time_time_sub,
    },
    StdlibFunction {
        id: TIME_TIME_FORMAT,
        symbol: "Format",
        returns_value: true,
        handler: super::super::time_format_impl::time_time_format,
    },
    StdlibFunction {
        id: TIME_TIMER_STOP,
        symbol: "Stop",
        returns_value: true,
        handler: super::super::time_timer_impl::time_timer_stop,
    },
    StdlibFunction {
        id: TIME_TIMER_RESET,
        symbol: "Reset",
        returns_value: true,
        handler: super::super::time_timer_impl::time_timer_reset,
    },
    StdlibFunction {
        id: TIME_DURATION_NANOSECONDS,
        symbol: "Nanoseconds",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_nanoseconds,
    },
    StdlibFunction {
        id: TIME_DURATION_MICROSECONDS,
        symbol: "Microseconds",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_microseconds,
    },
    StdlibFunction {
        id: TIME_DURATION_MILLISECONDS,
        symbol: "Milliseconds",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_milliseconds,
    },
    StdlibFunction {
        id: TIME_DURATION_SECONDS,
        symbol: "Seconds",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_seconds,
    },
    StdlibFunction {
        id: TIME_DURATION_MINUTES,
        symbol: "Minutes",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_minutes,
    },
    StdlibFunction {
        id: TIME_DURATION_HOURS,
        symbol: "Hours",
        returns_value: true,
        handler: super::super::time_duration_impl::time_duration_hours,
    },
];

fn time_now(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(time_value(current_time_unix_nanos(
        vm, program, "time.Now",
    )?))
}

fn time_unix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let sec = int_arg(vm, program, "time.Unix", &args[0])?;
    let nsec = int_arg(vm, program, "time.Unix", &args[1])?;
    Ok(time_value(checked_unix_nanos(
        vm,
        program,
        "time.Unix",
        sec,
        nsec,
    )?))
}

fn time_unix_milli(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let millis = int_arg(vm, program, "time.UnixMilli", &args[0])?;
    let unix_nanos = i128::from(millis) * 1_000_000i128;
    let function = vm.current_function_name(program)?;
    let unix_nanos = i64::try_from(unix_nanos).map_err(|_| VmError::UnhandledPanic {
        function: function.clone(),
        value: "time.UnixMilli overflow".into(),
    })?;
    Ok(time_value(unix_nanos))
}

fn time_unix_micro(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let micros = int_arg(vm, program, "time.UnixMicro", &args[0])?;
    let unix_nanos = i128::from(micros) * 1_000i128;
    let function = vm.current_function_name(program)?;
    let unix_nanos = i64::try_from(unix_nanos).map_err(|_| VmError::UnhandledPanic {
        function: function.clone(),
        value: "time.UnixMicro overflow".into(),
    })?;
    Ok(time_value(unix_nanos))
}

fn time_since(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let now = current_time_unix_nanos(vm, program, "time.Since")?;
    let then = extract_time_unix_nanos(vm, program, "time.Since", &args[0])?;
    Ok(super::super::time_duration_impl::duration_value(
        checked_sub(vm, program, "time.Since", now, then)?,
    ))
}

fn time_until(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let now = current_time_unix_nanos(vm, program, "time.Until")?;
    let then = extract_time_unix_nanos(vm, program, "time.Until", &args[0])?;
    Ok(super::super::time_duration_impl::duration_value(
        checked_sub(vm, program, "time.Until", then, now)?,
    ))
}

fn time_sleep(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let duration_nanos = super::super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "time.Sleep",
        super::super::time_duration_impl::single_duration_arg(vm, program, args)?,
    )?;
    if duration_nanos <= 0 {
        return Ok(Value::nil());
    }
    if !vm.capability_requests_enabled() {
        return Err(VmError::UnhandledPanic {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "time.Sleep".into()),
            value: "time.Sleep requires a host-backed timer capability".into(),
        });
    }
    vm.sleep_current_goroutine(duration_nanos);
    Ok(Value::nil())
}

fn time_time_unix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::int(
        extract_time_unix_nanos(vm, program, "time.Time.Unix", &args[0])?.div_euclid(1_000_000_000),
    ))
}

fn time_time_unix_milli(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::int(
        extract_time_unix_nanos(vm, program, "time.Time.UnixMilli", &args[0])?
            .div_euclid(1_000_000),
    ))
}

fn time_time_unix_micro(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::int(
        extract_time_unix_nanos(vm, program, "time.Time.UnixMicro", &args[0])?.div_euclid(1_000),
    ))
}

fn time_time_unix_nano(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::int(extract_time_unix_nanos(
        vm,
        program,
        "time.Time.UnixNano",
        &args[0],
    )?))
}

fn time_time_before(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (left, right) = compare_time_args(vm, program, "time.Time.Before", args)?;
    Ok(Value::bool(left < right))
}

fn time_time_after(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (left, right) = compare_time_args(vm, program, "time.Time.After", args)?;
    Ok(Value::bool(left > right))
}

fn time_time_equal(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (left, right) = compare_time_args(vm, program, "time.Time.Equal", args)?;
    Ok(Value::bool(left == right))
}

fn time_time_is_zero(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::bool(
        extract_time_unix_nanos(vm, program, "time.Time.IsZero", &args[0])? == 0,
    ))
}

fn time_time_compare(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (left, right) = compare_time_args(vm, program, "time.Time.Compare", args)?;
    Ok(Value::int(match left.cmp(&right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

fn time_time_add(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let unix_nanos = extract_time_unix_nanos(vm, program, "time.Time.Add", &args[0])?;
    let duration_nanos = super::super::time_duration_impl::extract_duration_nanos(
        vm,
        program,
        "time.Time.Add",
        &args[1],
    )?;
    Ok(time_value(checked_add(
        vm,
        program,
        "time.Time.Add",
        unix_nanos,
        duration_nanos,
    )?))
}

fn time_time_sub(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (left, right) = compare_time_args(vm, program, "time.Time.Sub", args)?;
    Ok(super::super::time_duration_impl::duration_value(
        checked_sub(vm, program, "time.Time.Sub", left, right)?,
    ))
}

pub(crate) fn time_value(unix_nanos: i64) -> Value {
    Value {
        typ: TYPE_TIME,
        data: ValueData::Struct(vec![(TIME_UNIX_NANOS_FIELD.into(), Value::int(unix_nanos))]),
    }
}

fn checked_unix_nanos(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    sec: i64,
    nsec: i64,
) -> Result<i64, VmError> {
    let unix_nanos = i128::from(sec) * 1_000_000_000i128 + i128::from(nsec);
    i64::try_from(unix_nanos).map_err(|_| VmError::UnhandledPanic {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| builtin.into()),
        value: format!("{builtin} overflow"),
    })
}

fn checked_add(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    left: i64,
    right: i64,
) -> Result<i64, VmError> {
    left.checked_add(right)
        .ok_or_else(|| VmError::UnhandledPanic {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| builtin.into()),
            value: format!("{builtin} overflow"),
        })
}

fn checked_sub(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    left: i64,
    right: i64,
) -> Result<i64, VmError> {
    left.checked_sub(right)
        .ok_or_else(|| VmError::UnhandledPanic {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| builtin.into()),
            value: format!("{builtin} overflow"),
        })
}

fn int_arg(vm: &Vm, program: &Program, builtin: &str, value: &Value) -> Result<i64, VmError> {
    let ValueData::Int(number) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "int arguments".into(),
        });
    };
    Ok(*number)
}

fn compare_time_args(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(i64, i64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    Ok((
        extract_time_unix_nanos(vm, program, builtin, &args[0])?,
        extract_time_unix_nanos(vm, program, builtin, &args[1])?,
    ))
}

pub(crate) fn current_time_unix_nanos(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
) -> Result<i64, VmError> {
    if let Some(unix_nanos) = vm.take_current_time_unix_nanos() {
        return Ok(unix_nanos);
    }
    if vm.capability_requests_enabled() {
        return Err(VmError::CapabilityRequest {
            kind: crate::CapabilityRequest::ClockNow,
        });
    }
    Err(VmError::UnhandledPanic {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| builtin.into()),
        value: format!("{builtin} requires a host-provided wall clock"),
    })
}

pub(crate) fn extract_time_unix_nanos(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<i64, VmError> {
    if value.typ != TYPE_TIME {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a time.Time receiver".into(),
        });
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a time.Time receiver".into(),
        });
    };
    Ok(fields
        .iter()
        .find(|(name, _)| name == TIME_UNIX_NANOS_FIELD)
        .and_then(|(_, value)| match &value.data {
            ValueData::Int(unix_nanos) => Some(*unix_nanos),
            _ => None,
        })
        .unwrap_or(0))
}
