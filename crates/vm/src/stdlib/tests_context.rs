use super::{
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_value,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value, StdlibValueInit,
};

#[test]
fn resolves_context_package_functions_and_methods_from_the_registry() {
    let background =
        resolve_stdlib_function("context", "Background").expect("context.Background should exist");
    assert!(stdlib_function_returns_value(background));
    assert_eq!(stdlib_function_result_count(background), 1);
    assert_eq!(stdlib_function_param_types(background), Some(&[][..]));
    assert_eq!(
        stdlib_function_result_types(background),
        Some(&["context.Context"][..])
    );

    let with_cancel =
        resolve_stdlib_function("context", "WithCancel").expect("context.WithCancel should exist");
    assert!(!stdlib_function_returns_value(with_cancel));
    assert_eq!(stdlib_function_result_count(with_cancel), 2);
    assert_eq!(
        stdlib_function_param_types(with_cancel),
        Some(&["context.Context"][..])
    );
    assert_eq!(
        stdlib_function_result_types(with_cancel),
        Some(&["context.Context", "context.CancelFunc"][..])
    );

    let with_timeout = resolve_stdlib_function("context", "WithTimeout")
        .expect("context.WithTimeout should exist");
    assert!(!stdlib_function_returns_value(with_timeout));
    assert_eq!(stdlib_function_result_count(with_timeout), 2);
    assert_eq!(
        stdlib_function_param_types(with_timeout),
        Some(&["context.Context", "time.Duration"][..])
    );
    assert_eq!(
        stdlib_function_result_types(with_timeout),
        Some(&["context.Context", "context.CancelFunc"][..])
    );

    let with_value =
        resolve_stdlib_function("context", "WithValue").expect("context.WithValue should exist");
    assert!(stdlib_function_returns_value(with_value));
    assert_eq!(stdlib_function_result_count(with_value), 1);
    assert_eq!(
        stdlib_function_param_types(with_value),
        Some(&["context.Context", "interface{}", "interface{}"][..])
    );
    assert_eq!(
        stdlib_function_result_types(with_value),
        Some(&["context.Context"][..])
    );

    let deadline = resolve_stdlib_method("context.__impl", "Deadline")
        .expect("context.Context.Deadline should exist");
    assert!(!stdlib_function_returns_value(deadline));
    assert_eq!(stdlib_function_result_count(deadline), 2);
    assert_eq!(
        stdlib_function_param_types(deadline),
        Some(&["context.__impl"][..])
    );
    assert_eq!(
        stdlib_function_result_types(deadline),
        Some(&["time.Time", "bool"][..])
    );

    let done =
        resolve_stdlib_method("context.__impl", "Done").expect("context.Context.Done should exist");
    assert!(stdlib_function_returns_value(done));
    assert_eq!(stdlib_function_result_count(done), 1);
    assert_eq!(
        stdlib_function_param_types(done),
        Some(&["context.__impl"][..])
    );
    assert_eq!(
        stdlib_function_result_types(done),
        Some(&["<-chan struct{}"][..])
    );

    let err =
        resolve_stdlib_method("context.__impl", "Err").expect("context.Context.Err should exist");
    assert!(stdlib_function_returns_value(err));
    assert_eq!(stdlib_function_result_count(err), 1);
    assert_eq!(
        stdlib_function_param_types(err),
        Some(&["context.__impl"][..])
    );
    assert_eq!(stdlib_function_result_types(err), Some(&["error"][..]));

    let canceled =
        resolve_stdlib_value("context", "Canceled").expect("context.Canceled should exist");
    assert_eq!(canceled.typ, "error");
    assert_eq!(
        canceled.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error("context canceled"))
    );

    let deadline_exceeded = resolve_stdlib_value("context", "DeadlineExceeded")
        .expect("context.DeadlineExceeded should exist");
    assert_eq!(deadline_exceeded.typ, "error");
    assert_eq!(
        deadline_exceeded.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error(
            "context deadline exceeded"
        ))
    );
}
