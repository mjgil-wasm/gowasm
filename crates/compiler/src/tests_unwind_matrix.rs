use super::compile_source;
use gowasm_vm::Vm;

fn run_ok(source: &str) -> String {
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

fn run_panic(source: &str) -> (String, String) {
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    (vm.stdout().to_string(), error.to_string())
}

#[test]
fn unwind_matrix_runs_defers_before_unhandled_panics() {
    let source = r#"
package main
import "fmt"

func main() {
    defer fmt.Println("cleanup")
    panic("boom")
}
"#;

    let (stdout, error) = run_panic(source);
    assert_eq!(stdout, "cleanup\n");
    assert!(error.contains("panic in function `main`: boom"));
}

#[test]
fn unwind_matrix_recovers_top_level_panics_from_deferred_calls() {
    let source = r#"
package main
import "fmt"

func cleanup() {
    fmt.Println("recovered", recover())
}

func main() {
    defer cleanup()
    panic("boom")
}
"#;

    assert_eq!(run_ok(source), "recovered boom\n");
}

#[test]
fn unwind_matrix_returns_nil_from_recover_without_an_active_panic() {
    let source = r#"
package main
import "fmt"

func cleanup() {
    r := recover()
    fmt.Println(r == nil)
}

func main() {
    defer cleanup()
    fmt.Println("body")
}
"#;

    assert_eq!(run_ok(source), "body\ntrue\n");
}

#[test]
fn unwind_matrix_recovers_outer_panics_and_returns_after_the_deferred_recover() {
    let source = r#"
package main
import "fmt"

func explode() {
    defer fmt.Println("inner defer")
    panic("boom")
}

func main() {
    defer func() {
        fmt.Println("outer recover", recover())
    }()
    fmt.Println("before")
    explode()
    fmt.Println("after")
}
"#;

    assert_eq!(run_ok(source), "before\ninner defer\nouter recover boom\n");
}

#[test]
fn unwind_matrix_only_the_first_recover_observes_the_active_panic() {
    let source = r#"
package main
import "fmt"

func main() {
    defer func() {
        r := recover()
        fmt.Println("second", r == nil)
    }()
    defer func() {
        r := recover()
        fmt.Println("first", r)
    }()
    panic("boom")
}
"#;

    assert_eq!(run_ok(source), "first boom\nsecond true\n");
}

#[test]
fn unwind_matrix_preserves_named_results_after_deferred_recover_cleanup() {
    let source = r#"
package main
import "fmt"

func demo() (value string) {
    value = "before"
    defer func() {
        recover()
        value = value + "-after"
    }()
    panic("boom")
}

func main() {
    fmt.Println(demo())
}
"#;

    assert_eq!(run_ok(source), "before-after\n");
}

#[test]
fn unwind_matrix_replaces_the_active_panic_when_a_deferred_call_panics() {
    let source = r#"
package main
import "fmt"

func main() {
    defer func() {
        fmt.Println("recover", recover())
    }()
    defer func() {
        fmt.Println("second")
        panic("replacement")
    }()
    defer fmt.Println("first")
    panic("initial")
}
"#;

    assert_eq!(run_ok(source), "first\nsecond\nrecover replacement\n");
}

#[test]
fn unwind_matrix_allows_recover_only_when_called_directly_from_the_deferred_frame() {
    let source = r#"
package main
import "fmt"

func helper() {
    fmt.Println("helper", recover() == nil)
}

func main() {
    defer func() {
        helper()
        fmt.Println("direct", recover())
    }()
    panic("boom")
}
"#;

    assert_eq!(run_ok(source), "helper true\ndirect boom\n");
}

#[test]
fn unwind_matrix_keeps_panic_and_recover_local_to_the_panicking_goroutine() {
    let source = r#"
package main
import "fmt"

func worker(done chan bool) {
    defer func() {
        fmt.Println("worker", recover())
        done <- true
    }()
    panic("boom")
}

func main() {
    done := make(chan bool, 1)
    go worker(done)
    <-done
    fmt.Println("main continues")
}
"#;

    assert_eq!(run_ok(source), "worker boom\nmain continues\n");
}
