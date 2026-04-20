use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_local_function_value_calls() {
    let source = r#"
package main
import "fmt"

func greet() {
    fmt.Println("hello")
}

func main() {
    run := greet
    run()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn compiles_and_runs_package_function_value_calls() {
    let source = r#"
package main
import "fmt"

func greet() {
    fmt.Println("hello")
}

var run = greet

func main() {
    run()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn rejects_non_function_local_calls() {
    let source = r#"
package main

func main() {
    value := 1
    value()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("calling local variable `value` is not supported"));
}

#[test]
fn compiles_and_runs_immediately_invoked_function_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    func() {
        fmt.Println("inline")
    }()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inline\n");
}

#[test]
fn compiles_and_runs_local_function_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    run := func(name string) {
        fmt.Println(name)
    }
    run("hello")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn compiles_and_runs_captured_closures() {
    let source = r#"
package main
import "fmt"

func main() {
    value := "before"
    run := func() {
        fmt.Println(value)
    }
    value = "after"
    run()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "after\n");
}

#[test]
fn compiles_and_runs_deferred_captured_closures() {
    let source = r#"
package main
import "fmt"

func main() {
    value := "before"
    defer func() {
        fmt.Println(value)
    }()
    value = "after"
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "after\n");
}

#[test]
fn compiles_and_runs_captured_local_mutation() {
    let source = r#"
package main
import "fmt"

func main() {
    value := 1
    run := func() {
        value++
    }
    run()
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_returned_captured_closures() {
    let source = r#"
package main
import "fmt"

func makeCounter() func() int {
    value := 40
    return func() int {
        value++
        return value
    }
}

func main() {
    run := makeCounter()
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "41\n42\n");
}

#[test]
fn compiles_and_runs_returned_parameter_captures() {
    let source = r#"
package main
import "fmt"

func wrap(value int) func() int {
    return func() int {
        value++
        return value
    }
}

func main() {
    run := wrap(7)
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8\n9\n");
}

#[test]
fn compiles_and_runs_returned_for_loop_captures() {
    let source = r#"
package main
import "fmt"

func captureLoop() func() int {
    i := 0
    for i < 3 {
        run := func() int {
            return i
        }
        i++
        if i == 3 {
            return run
        }
    }
    return nil
}

func main() {
    run := captureLoop()
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn compiles_and_runs_returned_range_captures() {
    let source = r#"
package main
import "fmt"

func captureRange() func() int {
    var run func() int
    for _, value := range []int{1, 2, 3} {
        run = func() int {
            return value
        }
    }
    return run
}

func main() {
    run := captureRange()
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn compiles_and_runs_nested_returned_local_captures() {
    let source = r#"
package main
import "fmt"

func makeNested() func() func() int {
    value := 10
    return func() func() int {
        return func() int {
            value++
            return value
        }
    }
}

func main() {
    outer := makeNested()
    inner := outer()
    fmt.Println(inner())
    fmt.Println(inner())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "11\n12\n");
}

#[test]
fn compiles_and_runs_nested_returned_parameter_captures() {
    let source = r#"
package main
import "fmt"

func makeNested(value int) func() func() int {
    return func() func() int {
        return func() int {
            value++
            return value
        }
    }
}

func main() {
    outer := makeNested(3)
    inner := outer()
    fmt.Println(inner())
    fmt.Println(inner())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4\n5\n");
}

#[test]
fn compiles_and_runs_nested_returned_for_loop_captures() {
    let source = r#"
package main
import "fmt"

func makeNestedLoop() func() func() int {
    i := 0
    for i < 2 {
        outer := func() func() int {
            return func() int {
                return i
            }
        }
        i++
        if i == 2 {
            return outer
        }
    }
    return nil
}

func main() {
    outer := makeNestedLoop()
    inner := outer()
    fmt.Println(inner())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_nested_returned_range_captures() {
    let source = r#"
package main
import "fmt"

func makeNestedRange() func() func() int {
    var outer func() func() int
    for _, value := range []int{1, 2, 3} {
        outer = func() func() int {
            return func() int {
                return value
            }
        }
    }
    return outer
}

func main() {
    outer := makeNestedRange()
    inner := outer()
    fmt.Println(inner())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn compiles_and_runs_explicit_function_typed_locals_and_globals() {
    let source = r#"
package main
import "fmt"

func greet(name string) {
    fmt.Println(name)
}

var global func(string) = greet

func main() {
    var local func(string) = greet
    local("local")
    global("global")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "local\nglobal\n");
}

#[test]
fn compiles_and_runs_function_nil_zero_values() {
    let source = r#"
package main
import "fmt"

func greet() {}

var globalZero func()
var globalNil func() = nil

func main() {
    var localZero func()
    var localNil func() = nil
    var localValue func() = greet
    fmt.Println(
        globalZero == nil,
        globalNil == nil,
        localZero == nil,
        localNil == nil,
        localValue == nil,
        localValue != nil,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true true false true\n");
}

#[test]
fn compiles_and_runs_function_typed_params_and_results() {
    let source = r#"
package main
import "fmt"

func greet(name string) string {
    return name
}

func wrap(run func(string) string) func(string) string {
    return run
}

func main() {
    run := wrap(greet)
    fmt.Println(run("hello"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn compiles_and_runs_function_typed_params_and_results_with_generic_instantiations() {
    let source = r#"
package main
import "fmt"

type Pair[A any, B any] struct {
    First A
    Second B
}

func formatPair(value Pair[int, string]) string {
    return fmt.Sprint(value.First, "-", value.Second)
}

func install(run func(Pair[int, string]) string) func(Pair[int, string]) string {
    return run
}

func main() {
    run := install(formatPair)
    fmt.Println(run(Pair[int, string]{First: 7, Second: "go"}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7-go\n");
}

#[test]
fn compiles_and_runs_nested_function_types_with_generic_map_values() {
    let source = r#"
package main
import "fmt"

type Pair[A any, B any] struct {
    First A
    Second B
}

func project(values map[string]Pair[int, string]) Pair[int, string] {
    return values["item"]
}

func bounce(run func(map[string]Pair[int, string]) Pair[int, string]) func(map[string]Pair[int, string]) Pair[int, string] {
    return run
}

func wrap(
    run func(func(map[string]Pair[int, string]) Pair[int, string]) func(map[string]Pair[int, string]) Pair[int, string],
) func(map[string]Pair[int, string]) Pair[int, string] {
    return run(project)
}

func main() {
    run := wrap(bounce)
    value := run(map[string]Pair[int, string]{
        "item": Pair[int, string]{First: 4, Second: "go"},
    })
    fmt.Println(value.First, value.Second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 go\n");
}

#[test]
fn rejects_mismatched_function_typed_initializers_and_returns() {
    let source = r#"
package main

func greet() {}

func wrap() func(string) {
    return greet
}

func main() {
    var run func(string) = greet
    _ = run
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("function value of type `func()` is not assignable to `func(string)`"));
}

#[test]
fn rejects_named_function_calls_with_the_wrong_arity() {
    let source = r#"
package main

func greet(name string) {}

func main() {
    greet()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`greet` expects 1 argument(s), found 0"));
}

#[test]
fn rejects_function_value_calls_with_the_wrong_arity() {
    let source = r#"
package main

func greet(name string) {}

func main() {
    run := greet
    run()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("function value expects 1 argument(s), found 0"));
}

#[test]
fn rejects_function_literal_calls_with_the_wrong_arity() {
    let source = r#"
package main

func main() {
    func(name string) {}()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("function value expects 1 argument(s), found 0"));
}

#[test]
fn rejects_named_function_calls_with_the_wrong_argument_types() {
    let source = r#"
package main

func greet(name string) {}

func main() {
    greet(1)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `greet` has type `int`, expected `string`"));
}

#[test]
fn rejects_function_value_calls_with_the_wrong_argument_types() {
    let source = r#"
package main

func greet(name string) {}

func main() {
    run := greet
    run(1)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to function value has type `int`, expected `string`"));
}

#[test]
fn compiles_and_runs_variadic_function() {
    let source = r#"
package main
import "fmt"

func sum(nums ...int) int {
    total := 0
    for _, n := range nums {
        total += n
    }
    return total
}

func main() {
    fmt.Println(sum())
    fmt.Println(sum(1))
    fmt.Println(sum(1, 2, 3))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n6\n");
}

#[test]
fn compiles_and_runs_variadic_with_fixed_params() {
    let source = r#"
package main
import "fmt"

func greet(prefix string, names ...string) {
    for _, name := range names {
        fmt.Println(prefix + " " + name)
    }
}

func main() {
    greet("Hello", "Alice", "Bob")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Hello Alice\nHello Bob\n");
}

#[test]
fn compiles_and_runs_variadic_with_spread() {
    let source = r#"
package main
import "fmt"

func sum(nums ...int) int {
    total := 0
    for _, n := range nums {
        total += n
    }
    return total
}

func main() {
    nums := []int{10, 20, 30}
    fmt.Println(sum(nums...))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "60\n");
}
