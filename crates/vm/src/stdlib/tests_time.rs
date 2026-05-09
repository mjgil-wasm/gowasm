use super::{
    resolve_stdlib_constant, resolve_stdlib_function, resolve_stdlib_method,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value,
};

#[test]
fn resolves_time_package_functions_and_methods_from_the_registry() {
    let date_time =
        resolve_stdlib_constant("time", "DateTime").expect("time.DateTime should exist");
    assert_eq!(date_time.typ, "string");
    assert_eq!(
        date_time.value,
        super::StdlibConstantValue::String("2006-01-02 15:04:05")
    );

    let ansic = resolve_stdlib_constant("time", "ANSIC").expect("time.ANSIC should exist");
    assert_eq!(ansic.typ, "string");
    assert_eq!(
        ansic.value,
        super::StdlibConstantValue::String("Mon Jan _2 15:04:05 2006")
    );

    let rfc850 = resolve_stdlib_constant("time", "RFC850").expect("time.RFC850 should exist");
    assert_eq!(rfc850.typ, "string");
    assert_eq!(
        rfc850.value,
        super::StdlibConstantValue::String("Monday, 02-Jan-06 15:04:05 MST")
    );

    let rfc1123 = resolve_stdlib_constant("time", "RFC1123").expect("time.RFC1123 should exist");
    assert_eq!(rfc1123.typ, "string");
    assert_eq!(
        rfc1123.value,
        super::StdlibConstantValue::String("Mon, 02 Jan 2006 15:04:05 MST")
    );

    let rfc1123z = resolve_stdlib_constant("time", "RFC1123Z").expect("time.RFC1123Z should exist");
    assert_eq!(rfc1123z.typ, "string");
    assert_eq!(
        rfc1123z.value,
        super::StdlibConstantValue::String("Mon, 02 Jan 2006 15:04:05 -0700")
    );

    let rfc3339 = resolve_stdlib_constant("time", "RFC3339").expect("time.RFC3339 should exist");
    assert_eq!(rfc3339.typ, "string");
    assert_eq!(
        rfc3339.value,
        super::StdlibConstantValue::String("2006-01-02T15:04:05Z07:00")
    );

    let now = resolve_stdlib_function("time", "Now").expect("time.Now should exist");
    assert!(stdlib_function_returns_value(now));
    assert_eq!(stdlib_function_result_count(now), 1);
    assert_eq!(stdlib_function_param_types(now), Some(&[][..]));
    assert_eq!(stdlib_function_result_types(now), Some(&["time.Time"][..]));

    let unix = resolve_stdlib_function("time", "Unix").expect("time.Unix should exist");
    assert!(stdlib_function_returns_value(unix));
    assert_eq!(stdlib_function_result_count(unix), 1);
    assert_eq!(stdlib_function_param_types(unix), Some(&["int", "int"][..]));
    assert_eq!(stdlib_function_result_types(unix), Some(&["time.Time"][..]));

    let unix_milli =
        resolve_stdlib_function("time", "UnixMilli").expect("time.UnixMilli should exist");
    assert!(stdlib_function_returns_value(unix_milli));
    assert_eq!(stdlib_function_result_count(unix_milli), 1);
    assert_eq!(stdlib_function_param_types(unix_milli), Some(&["int"][..]));
    assert_eq!(
        stdlib_function_result_types(unix_milli),
        Some(&["time.Time"][..])
    );

    let unix_micro =
        resolve_stdlib_function("time", "UnixMicro").expect("time.UnixMicro should exist");
    assert!(stdlib_function_returns_value(unix_micro));
    assert_eq!(stdlib_function_result_count(unix_micro), 1);
    assert_eq!(stdlib_function_param_types(unix_micro), Some(&["int"][..]));
    assert_eq!(
        stdlib_function_result_types(unix_micro),
        Some(&["time.Time"][..])
    );

    let since = resolve_stdlib_function("time", "Since").expect("time.Since should exist");
    assert!(stdlib_function_returns_value(since));
    assert_eq!(stdlib_function_result_count(since), 1);
    assert_eq!(stdlib_function_param_types(since), Some(&["time.Time"][..]));
    assert_eq!(
        stdlib_function_result_types(since),
        Some(&["time.Duration"][..])
    );

    let until = resolve_stdlib_function("time", "Until").expect("time.Until should exist");
    assert!(stdlib_function_returns_value(until));
    assert_eq!(stdlib_function_result_count(until), 1);
    assert_eq!(stdlib_function_param_types(until), Some(&["time.Time"][..]));
    assert_eq!(
        stdlib_function_result_types(until),
        Some(&["time.Duration"][..])
    );

    let parse = resolve_stdlib_function("time", "Parse").expect("time.Parse should exist");
    assert!(!stdlib_function_returns_value(parse));
    assert_eq!(stdlib_function_result_count(parse), 2);
    assert_eq!(
        stdlib_function_param_types(parse),
        Some(&["string", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse),
        Some(&["time.Time", "error"][..])
    );

    let sleep = resolve_stdlib_function("time", "Sleep").expect("time.Sleep should exist");
    assert!(!stdlib_function_returns_value(sleep));
    assert_eq!(stdlib_function_result_count(sleep), 0);
    assert_eq!(
        stdlib_function_param_types(sleep),
        Some(&["time.Duration"][..])
    );
    assert_eq!(stdlib_function_result_types(sleep), Some(&[][..]));

    let after = resolve_stdlib_function("time", "After").expect("time.After should exist");
    assert!(stdlib_function_returns_value(after));
    assert_eq!(stdlib_function_result_count(after), 1);
    assert_eq!(
        stdlib_function_param_types(after),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(after),
        Some(&["<-chan time.Time"][..])
    );

    let new_timer =
        resolve_stdlib_function("time", "NewTimer").expect("time.NewTimer should exist");
    assert!(stdlib_function_returns_value(new_timer));
    assert_eq!(stdlib_function_result_count(new_timer), 1);
    assert_eq!(
        stdlib_function_param_types(new_timer),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(new_timer),
        Some(&["*time.Timer"][..])
    );

    let method_unix =
        resolve_stdlib_method("time.Time", "Unix").expect("time.Time.Unix should exist");
    assert!(stdlib_function_returns_value(method_unix));
    assert_eq!(stdlib_function_result_count(method_unix), 1);
    assert_eq!(
        stdlib_function_param_types(method_unix),
        Some(&["time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(method_unix),
        Some(&["int64"][..])
    );

    let method_unix_milli =
        resolve_stdlib_method("time.Time", "UnixMilli").expect("time.Time.UnixMilli should exist");
    assert!(stdlib_function_returns_value(method_unix_milli));
    assert_eq!(stdlib_function_result_count(method_unix_milli), 1);
    assert_eq!(
        stdlib_function_param_types(method_unix_milli),
        Some(&["time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(method_unix_milli),
        Some(&["int64"][..])
    );

    let method_unix_micro =
        resolve_stdlib_method("time.Time", "UnixMicro").expect("time.Time.UnixMicro should exist");
    assert!(stdlib_function_returns_value(method_unix_micro));
    assert_eq!(stdlib_function_result_count(method_unix_micro), 1);
    assert_eq!(
        stdlib_function_param_types(method_unix_micro),
        Some(&["time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(method_unix_micro),
        Some(&["int64"][..])
    );

    let method_unix_nano =
        resolve_stdlib_method("time.Time", "UnixNano").expect("time.Time.UnixNano should exist");
    assert!(stdlib_function_returns_value(method_unix_nano));
    assert_eq!(stdlib_function_result_count(method_unix_nano), 1);
    assert_eq!(
        stdlib_function_param_types(method_unix_nano),
        Some(&["time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(method_unix_nano),
        Some(&["int64"][..])
    );

    let before =
        resolve_stdlib_method("time.Time", "Before").expect("time.Time.Before should exist");
    assert!(stdlib_function_returns_value(before));
    assert_eq!(stdlib_function_result_count(before), 1);
    assert_eq!(
        stdlib_function_param_types(before),
        Some(&["time.Time", "time.Time"][..])
    );
    assert_eq!(stdlib_function_result_types(before), Some(&["bool"][..]));

    let after_method =
        resolve_stdlib_method("time.Time", "After").expect("time.Time.After should exist");
    assert!(stdlib_function_returns_value(after_method));
    assert_eq!(stdlib_function_result_count(after_method), 1);
    assert_eq!(
        stdlib_function_param_types(after_method),
        Some(&["time.Time", "time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(after_method),
        Some(&["bool"][..])
    );

    let equal = resolve_stdlib_method("time.Time", "Equal").expect("time.Time.Equal should exist");
    assert!(stdlib_function_returns_value(equal));
    assert_eq!(stdlib_function_result_count(equal), 1);
    assert_eq!(
        stdlib_function_param_types(equal),
        Some(&["time.Time", "time.Time"][..])
    );
    assert_eq!(stdlib_function_result_types(equal), Some(&["bool"][..]));

    let is_zero =
        resolve_stdlib_method("time.Time", "IsZero").expect("time.Time.IsZero should exist");
    assert!(stdlib_function_returns_value(is_zero));
    assert_eq!(stdlib_function_result_count(is_zero), 1);
    assert_eq!(
        stdlib_function_param_types(is_zero),
        Some(&["time.Time"][..])
    );
    assert_eq!(stdlib_function_result_types(is_zero), Some(&["bool"][..]));

    let compare =
        resolve_stdlib_method("time.Time", "Compare").expect("time.Time.Compare should exist");
    assert!(stdlib_function_returns_value(compare));
    assert_eq!(stdlib_function_result_count(compare), 1);
    assert_eq!(
        stdlib_function_param_types(compare),
        Some(&["time.Time", "time.Time"][..])
    );
    assert_eq!(stdlib_function_result_types(compare), Some(&["int"][..]));

    let add = resolve_stdlib_method("time.Time", "Add").expect("time.Time.Add should exist");
    assert!(stdlib_function_returns_value(add));
    assert_eq!(stdlib_function_result_count(add), 1);
    assert_eq!(
        stdlib_function_param_types(add),
        Some(&["time.Time", "time.Duration"][..])
    );
    assert_eq!(stdlib_function_result_types(add), Some(&["time.Time"][..]));

    let sub = resolve_stdlib_method("time.Time", "Sub").expect("time.Time.Sub should exist");
    assert!(stdlib_function_returns_value(sub));
    assert_eq!(stdlib_function_result_count(sub), 1);
    assert_eq!(
        stdlib_function_param_types(sub),
        Some(&["time.Time", "time.Time"][..])
    );
    assert_eq!(
        stdlib_function_result_types(sub),
        Some(&["time.Duration"][..])
    );

    let format =
        resolve_stdlib_method("time.Time", "Format").expect("time.Time.Format should exist");
    assert!(stdlib_function_returns_value(format));
    assert_eq!(stdlib_function_result_count(format), 1);
    assert_eq!(
        stdlib_function_param_types(format),
        Some(&["time.Time", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(format), Some(&["string"][..]));

    let stop = resolve_stdlib_method("*time.Timer", "Stop").expect("time.Timer.Stop should exist");
    assert!(stdlib_function_returns_value(stop));
    assert_eq!(stdlib_function_result_count(stop), 1);
    assert_eq!(
        stdlib_function_param_types(stop),
        Some(&["*time.Timer"][..])
    );
    assert_eq!(stdlib_function_result_types(stop), Some(&["bool"][..]));

    let reset =
        resolve_stdlib_method("*time.Timer", "Reset").expect("time.Timer.Reset should exist");
    assert!(stdlib_function_returns_value(reset));
    assert_eq!(stdlib_function_result_count(reset), 1);
    assert_eq!(
        stdlib_function_param_types(reset),
        Some(&["*time.Timer", "time.Duration"][..])
    );
    assert_eq!(stdlib_function_result_types(reset), Some(&["bool"][..]));

    let nanoseconds = resolve_stdlib_method("time.Duration", "Nanoseconds")
        .expect("time.Duration.Nanoseconds should exist");
    assert!(stdlib_function_returns_value(nanoseconds));
    assert_eq!(stdlib_function_result_count(nanoseconds), 1);
    assert_eq!(
        stdlib_function_param_types(nanoseconds),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(nanoseconds),
        Some(&["int64"][..])
    );

    let microseconds = resolve_stdlib_method("time.Duration", "Microseconds")
        .expect("time.Duration.Microseconds should exist");
    assert!(stdlib_function_returns_value(microseconds));
    assert_eq!(stdlib_function_result_count(microseconds), 1);
    assert_eq!(
        stdlib_function_param_types(microseconds),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(microseconds),
        Some(&["int64"][..])
    );

    let milliseconds = resolve_stdlib_method("time.Duration", "Milliseconds")
        .expect("time.Duration.Milliseconds should exist");
    assert!(stdlib_function_returns_value(milliseconds));
    assert_eq!(stdlib_function_result_count(milliseconds), 1);
    assert_eq!(
        stdlib_function_param_types(milliseconds),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(milliseconds),
        Some(&["int64"][..])
    );

    let seconds = resolve_stdlib_method("time.Duration", "Seconds")
        .expect("time.Duration.Seconds should exist");
    assert!(stdlib_function_returns_value(seconds));
    assert_eq!(stdlib_function_result_count(seconds), 1);
    assert_eq!(
        stdlib_function_param_types(seconds),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(seconds),
        Some(&["float64"][..])
    );

    let minutes = resolve_stdlib_method("time.Duration", "Minutes")
        .expect("time.Duration.Minutes should exist");
    assert!(stdlib_function_returns_value(minutes));
    assert_eq!(stdlib_function_result_count(minutes), 1);
    assert_eq!(
        stdlib_function_param_types(minutes),
        Some(&["time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(minutes),
        Some(&["float64"][..])
    );

    let hours =
        resolve_stdlib_method("time.Duration", "Hours").expect("time.Duration.Hours should exist");
    assert!(stdlib_function_returns_value(hours));
    assert_eq!(stdlib_function_result_count(hours), 1);
    assert_eq!(
        stdlib_function_param_types(hours),
        Some(&["time.Duration"][..])
    );
    assert_eq!(stdlib_function_result_types(hours), Some(&["float64"][..]));
}

#[test]
fn resolves_time_duration_constants_from_the_registry() {
    let second = resolve_stdlib_constant("time", "Second").expect("time.Second should exist");
    assert_eq!(second.typ, "time.Duration");
    assert_eq!(second.value, super::StdlibConstantValue::Int(1_000_000_000));

    let minute = resolve_stdlib_constant("time", "Minute").expect("time.Minute should exist");
    assert_eq!(minute.typ, "time.Duration");
    assert_eq!(
        minute.value,
        super::StdlibConstantValue::Int(60_000_000_000)
    );
}
