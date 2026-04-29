use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn defer_executes_in_lifo_order() {
    let source = r#"
package main
import "fmt"

func main() {
    defer fmt.Println("first")
    defer fmt.Println("second")
    defer fmt.Println("third")
    fmt.Println("body")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "body\nthird\nsecond\nfirst\n");
}

#[test]
fn defer_captures_argument_values_at_defer_time() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 1
    defer fmt.Println("deferred x:", x)
    x = 2
    fmt.Println("current x:", x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "current x: 2\ndeferred x: 1\n");
}

#[test]
fn defer_in_loop_stacks_multiple_calls() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := 0; i < 3; i++ {
        defer fmt.Println(i)
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "done\n2\n1\n0\n");
}

#[test]
fn defer_runs_on_early_return() {
    let source = r#"
package main
import "fmt"

func work(fail bool) {
    defer fmt.Println("cleanup")
    if fail {
        fmt.Println("early exit")
        return
    }
    fmt.Println("normal exit")
}

func main() {
    work(true)
    work(false)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "early exit\ncleanup\nnormal exit\ncleanup\n");
}

#[test]
fn defer_with_anonymous_function() {
    let source = r#"
package main
import "fmt"

func main() {
    defer func() {
        fmt.Println("deferred anon")
    }()
    fmt.Println("body")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "body\ndeferred anon\n");
}

#[test]
fn panic_with_integer_value() {
    let source = r#"
package main
import "fmt"

func cleanup() {
    r := recover()
    fmt.Println("recovered:", r)
}

func main() {
    defer cleanup()
    panic(42)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "recovered: 42\n");
}

#[test]
fn recover_returns_nil_when_no_panic() {
    let source = r#"
package main
import "fmt"

func tryRecover() {
    r := recover()
    fmt.Println(r == nil)
}

func main() {
    defer tryRecover()
    fmt.Println("no panic")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "no panic\ntrue\n");
}

#[test]
fn nested_defer_with_panic_and_recover() {
    let source = r#"
package main
import "fmt"

func inner() {
    defer fmt.Println("inner defer")
    panic("inner panic")
}

func middle() {
    defer func() {
        r := recover()
        fmt.Println("recovered:", r)
    }()
    defer fmt.Println("middle defer")
    inner()
    fmt.Println("unreachable")
}

func main() {
    middle()
    fmt.Println("main continues")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(
        vm.stdout(),
        "inner defer\nmiddle defer\nrecovered: inner panic\nmain continues\n"
    );
}

#[test]
fn defer_with_method_call() {
    let source = r#"
package main
import "fmt"

type Logger struct {
    name string
}

func (l Logger) Close() {
    fmt.Println("closing", l.name)
}

func main() {
    log := Logger{name: "db"}
    defer log.Close()
    fmt.Println("working")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "working\nclosing db\n");
}

#[test]
fn multiple_recovers_only_first_catches() {
    let source = r#"
package main
import "fmt"

func main() {
    defer func() {
        r := recover()
        fmt.Println("second recover:", r == nil)
    }()
    defer func() {
        r := recover()
        fmt.Println("first recover:", r)
    }()
    panic("oops")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("panic should be recovered");
    assert_eq!(vm.stdout(), "first recover: oops\nsecond recover: true\n");
}
