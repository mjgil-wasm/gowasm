use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_recursive_closure_values_where_binding_is_already_in_scope() {
    let source = r#"
package main
import "fmt"

func main() {
    var run func(int) int
    run = func(n int) int {
        if n == 0 {
            return 0
        }
        return 1 + run(n-1)
    }
    fmt.Println(run(5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn compiles_and_runs_function_values_stored_in_maps_structs_and_interfaces() {
    let source = r#"
package main
import "fmt"

type Holder struct {
    Run func(int) int
    Any interface{}
}

func main() {
    values := map[string]func(int) int{
        "double": func(v int) int { return v * 2 },
    }
    holder := Holder{
        Run: values["double"],
        Any: func(v int) int { return v + 1 },
    }
    fmt.Println(holder.Run(21))
    fmt.Println(holder.Any != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\ntrue\n");
}

#[test]
fn compiles_and_runs_generic_returned_closure_values() {
    let source = r#"
package main
import "fmt"

func MakeAdder[T any](base T, combine func(T, T) T) func(T) T {
    return func(next T) T {
        return combine(base, next)
    }
}

func main() {
    add := MakeAdder[int](7, func(left int, right int) int {
        return left + right
    })
    fmt.Println(add(5))
    fmt.Println(add(8))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "12\n15\n");
}

#[test]
fn rejects_struct_literal_function_field_signature_mismatches() {
    let source = r#"
package main

type Holder struct {
    Run func(int) int
}

func main() {
    _ = Holder{
        Run: func(name string) int { return len(name) },
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) int` is not assignable to `func(int) int`"
    ));
}

#[test]
fn rejects_map_literal_function_value_signature_mismatches() {
    let source = r#"
package main

func main() {
    _ = map[string]func(int) int{
        "bad": func(name string) int { return len(name) },
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) int` is not assignable to `func(int) int`"
    ));
}
