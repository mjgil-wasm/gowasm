use super::compile_source;
use gowasm_vm::{Vm, VmError};

#[test]
fn runs_deferred_calls_before_unhandled_panic() {
    let source = r#"
package main
import "fmt"

func main() {
    defer fmt.Println("cleanup")
    panic("boom")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert_eq!(vm.stdout(), "cleanup\n");
    assert!(matches!(error.root_cause(), VmError::UnhandledPanic { .. }));
    let text = error.to_string();
    assert!(text.contains("panic in function `main`: boom"));
    assert!(text.contains("at main (main.go:7:5)"));
}

#[test]
fn unwinds_panics_through_nested_frames_in_lifo_order() {
    let source = r#"
package main
import "fmt"

func explode() {
    defer fmt.Println("inner")
    panic("boom")
}

func main() {
    defer fmt.Println("outer")
    explode()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert_eq!(vm.stdout(), "inner\nouter\n");
    assert!(matches!(error.root_cause(), VmError::UnhandledPanic { .. }));
    let text = error.to_string();
    assert!(text.contains("panic in function `main`: boom"));
    let explode = text
        .find("at explode (main.go:7:5)")
        .expect("stack trace should include the panicking frame");
    let main = text
        .find("at main (main.go:12:5)")
        .expect("stack trace should include the caller frame");
    assert!(explode < main, "stack trace should be leaf-first: {text}");
}

#[test]
fn recover_stops_a_top_level_panic_in_a_deferred_call() {
    let source = r#"
package main
import "fmt"

func cleanup() {
    fmt.Println(recover())
}

func main() {
    defer cleanup()
    panic("boom")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "boom\n");
}

#[test]
fn recover_allows_callers_to_continue_after_a_panicking_call() {
    let source = r#"
package main
import "fmt"

func cleanup() {
    fmt.Println("cleanup", recover())
}

func explode() {
    defer cleanup()
    panic("boom")
}

func main() {
    fmt.Println("before")
    explode()
    fmt.Println("after")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "before\ncleanup boom\nafter\n");
}

#[test]
fn runtime_faults_render_source_mapped_stack_traces() {
    let source = r#"
package main

func explode() {
    value := 0
    _ = 1 / value
}

func main() {
    explode()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("program should fail at runtime");
    assert!(matches!(error.root_cause(), VmError::DivisionByZero { .. }));
    let text = error.to_string();
    assert!(text.contains("division by zero in function `explode`"));
    let explode = text
        .find("at explode (main.go:6:5)")
        .expect("stack trace should include the faulting frame");
    let main = text
        .find("at main (main.go:10:5)")
        .expect("stack trace should include the caller frame");
    assert!(explode < main, "stack trace should be leaf-first: {text}");
}

#[test]
fn deferred_panics_replace_the_original_panic_value_and_stack() {
    let source = r#"
package main

func cleanup() {
    panic("replacement")
}

func main() {
    defer cleanup()
    panic("initial")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(matches!(error.root_cause(), VmError::UnhandledPanic { .. }));
    let text = error.to_string();
    assert!(text.contains("panic in function `main`: replacement"));
    assert!(text.contains("at cleanup (main.go:5:5)"));
    assert!(text.contains("at main (main.go:10:5)"));
    assert!(!text.contains("panic in function `main`: initial"));
}

#[test]
fn unhandled_goroutine_panics_render_goroutine_local_stack_traces() {
    let source = r#"
package main

func worker() {
    panic("boom")
}

func main() {
    done := make(chan bool)
    go worker()
    <-done
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("goroutine panic should fail the run");
    assert!(matches!(error.root_cause(), VmError::UnhandledPanic { .. }));
    let text = error.to_string();
    assert!(text.contains("panic in function `worker`: boom"));
    assert!(text.contains("at worker (main.go:5:5)"));
    assert!(
        !text.contains("at main (main.go:11:5)"),
        "goroutine panic stack should stay local to the panicking goroutine: {text}"
    );
}
