use super::compile_source;
use gowasm_vm::{CapabilityRequest, RunOutcome, Vm};

#[test]
fn context_type_assertion_recognizes_stdlib_backed_context_values() {
    let source = r#"
package main
import "context"
import "fmt"

func main() {
    var any interface{} = context.Background()
    ctx := any.(context.Context)
    deadline, ok := ctx.Deadline()
    fmt.Println(ok)
    fmt.Println(deadline.UnixNano())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n0\n");
}

#[test]
fn context_with_cancel_propagates_parent_cancellation() {
    let source = r#"
package main
import "context"
import "fmt"

func main() {
    parent, cancel := context.WithCancel(context.Background())
    child, childCancel := context.WithCancel(parent)
    defer childCancel()

    cancel()
    <-child.Done()
    fmt.Println(child.Err().Error())
    deadline, ok := child.Deadline()
    fmt.Println(ok)
    fmt.Println(deadline.UnixNano())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "context canceled\nfalse\n0\n");
}

#[test]
fn context_with_timeout_uses_host_clock_and_sleep_resume() {
    let source = r#"
package main
import "context"
import "fmt"

func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 2000000)
    defer cancel()

    <-ctx.Done()
    fmt.Println(ctx.Err().Error())
    deadline, ok := ctx.Deadline()
    fmt.Println(ok)
    fmt.Println(deadline.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::ClockNow) => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_clock_now_result_unix_nanos(1_700_000_000_123_000_000);
    match vm
        .resume_program(&program)
        .expect("program should resume after clock")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_000_125_000_000))
        .expect("timers should advance");
    match vm
        .resume_program(&program)
        .expect("program should resume after sleep")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(
        vm.stdout(),
        "context deadline exceeded\ntrue\n1700000000125\n"
    );
}

#[test]
fn context_with_value_shadows_keys_and_inherits_parent_cancellation() {
    let source = r#"
package main
import "context"
import "fmt"

func main() {
    parent, cancel := context.WithCancel(context.Background())
    ctx := context.WithValue(parent, "key", "outer")
    ctx = context.WithValue(ctx, "key", "inner")
    ctx = context.WithValue(ctx, "other", 7)

    fmt.Println(ctx.Value("key"))
    fmt.Println(ctx.Value("other"))
    fmt.Println(ctx.Value("missing"))

    cancel()
    <-ctx.Done()
    fmt.Println(ctx.Err().Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner\n7\n<nil>\ncontext canceled\n");
}

#[test]
fn context_with_value_panics_on_nil_key() {
    let source = r#"
package main
import "context"

func main() {
    var key interface{}
    context.WithValue(context.Background(), key, 1)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error.to_string().contains("nil key"));
}

#[test]
fn context_with_value_rejects_literal_nil_key_at_compile_time() {
    let source = r#"
package main
import "context"

func main() {
    context.WithValue(context.Background(), nil, 1)
}
"#;

    let error = compile_source(source).expect_err("literal nil keys should fail at compile time");
    assert!(error.to_string().contains("cannot be `nil`"));
}

#[test]
fn context_with_value_rejects_non_comparable_key_at_compile_time() {
    let source = r#"
package main
import "context"

func main() {
    key := []int{1, 2, 3}
    context.WithValue(context.Background(), key, 1)
}
"#;

    let error =
        compile_source(source).expect_err("non-comparable keys should fail at compile time");
    assert!(error.to_string().contains("non-comparable type `[]int`"));
}

#[test]
fn context_exported_error_values_work_in_value_position_and_compare_to_err_results() {
    let source = r#"
package main
import "context"
import "fmt"
import "time"

func main() {
    canceled, cancel := context.WithCancel(context.Background())
    cancel()
    <-canceled.Done()
    fmt.Println(context.Canceled.Error())
    fmt.Println(canceled.Err() == context.Canceled)

    deadline, deadlineCancel := context.WithDeadline(context.Background(), time.UnixMilli(10))
    defer deadlineCancel()
    <-deadline.Done()
    fmt.Println(context.DeadlineExceeded.Error())
    fmt.Println(deadline.Err() == context.DeadlineExceeded)

    err := context.Canceled
    fmt.Println(err == context.Canceled)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(20_000_000);
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "context canceled\ntrue\ncontext deadline exceeded\ntrue\ntrue\n"
    );
}

#[test]
fn package_var_can_use_exported_context_error_value() {
    let source = r#"
package main
import "context"
import "fmt"

var globalErr = context.Canceled

func main() {
    fmt.Println(globalErr == context.Canceled)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn context_derivation_supports_custom_parent_contexts_through_value_wrappers() {
    let source = r#"
package main
import "context"
import "fmt"
import "time"

type customCtx struct {
    done chan struct{}
    deadline time.Time
    hasDeadline bool
}

func (c customCtx) Deadline() (time.Time, bool) {
    return c.deadline, c.hasDeadline
}

func (c customCtx) Done() <-chan struct{} {
    return c.done
}

func (c customCtx) Err() error {
    select {
    case <-c.done:
        return context.Canceled
    default:
        return nil
    }
}

func (c customCtx) Value(key interface{}) interface{} {
    if key == "outer" {
        return "parent"
    }
    return nil
}

func main() {
    done := make(chan struct{})
    parent := customCtx{
        done: done,
        deadline: time.UnixMilli(55),
        hasDeadline: true,
    }
    base := context.WithValue(parent, "inner", "child")
    child, cancel := context.WithCancel(base)
    defer cancel()

    deadline, ok := child.Deadline()
    fmt.Println(ok)
    fmt.Println(deadline.UnixMilli())
    fmt.Println(child.Value("outer"))
    fmt.Println(child.Value("inner"))

    close(done)
    <-child.Done()
    fmt.Println(base.Err() == context.Canceled)
    fmt.Println(child.Err() == context.Canceled)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n55\nparent\nchild\ntrue\ntrue\n");
}

#[test]
fn context_derivation_accepts_already_canceled_custom_parents() {
    let source = r#"
package main
import "context"
import "fmt"
import "time"

type customCtx struct {
    done chan struct{}
}

func (c customCtx) Deadline() (time.Time, bool) {
    return time.UnixMilli(0), false
}

func (c customCtx) Done() <-chan struct{} {
    return c.done
}

func (c customCtx) Err() error {
    select {
    case <-c.done:
        return context.Canceled
    default:
        return nil
    }
}

func (c customCtx) Value(key interface{}) interface{} {
    return nil
}

func main() {
    done := make(chan struct{})
    close(done)
    ctx, cancel := context.WithCancel(customCtx{done: done})
    defer cancel()

    <-ctx.Done()
    fmt.Println(ctx.Err() == context.Canceled)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}
