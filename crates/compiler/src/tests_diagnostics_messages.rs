use crate::{compile_source, CompileError};

#[test]
fn plain_call_reports_actual_type_and_expected_shape() {
    let source = r#"
package main

func main() {
    value := 1
    value()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("calling local variable `value` is not supported"));
            assert!(detail.contains("call target `value` has type `int`"));
            assert!(
                detail.contains("expected a function value, named function, or supported builtin")
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn multi_result_call_reports_expected_result_shape() {
    let source = r#"
package main

func main() {
    value := 1
    left, right := value()
    _, _ = left, right
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("calling local variable `value` is not supported"));
            assert!(detail.contains("call target `value` has type `int`"));
            assert!(detail.contains("expected a function value returning 2 value(s)"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn unsupported_expression_call_reports_actual_type() {
    let source = r#"
package main

func main() {
    values := []int{1}
    values[0]()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("unsupported call target"));
            assert!(detail.contains("call target index expression has type `int`"));
            assert!(
                detail.contains("expected a function value, named function, or supported builtin")
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn go_call_reports_actual_type_and_expected_shape() {
    let source = r#"
package main

func main() {
    value := 1
    go value()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("`go` cannot call local variable `value`"));
            assert!(detail.contains("call target `value` has type `int`"));
            assert!(detail.contains("expected a function value or concrete method call"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn defer_shadowing_reports_shadowed_local_type_instead_of_named_function() {
    let source = r#"
package main

func cleanup() {}

func main() {
    cleanup := 1
    defer cleanup()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("`defer` cannot call local variable `cleanup`"));
            assert!(detail.contains("call target `cleanup` has type `int`"));
            assert!(detail.contains("expected a function value, named function, or selector call"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
