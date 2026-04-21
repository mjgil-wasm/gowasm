use super::{compile_source, compile_workspace, SourceInput};
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_a_basic_program() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println("hello", 42)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 42\n");
}

#[test]
fn compiles_and_runs_nested_function_calls() {
    let source = r#"
package main
import "fmt"

func helper() {
    fmt.Println("from helper")
}

func main() {
    helper()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "from helper\n");
}

#[test]
fn compiles_and_runs_deferred_calls_on_return() {
    let source = r#"
package main
import "fmt"

func main() {
    defer fmt.Println("later")
    fmt.Println("now")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "now\nlater\n");
}

#[test]
fn compiles_and_runs_deferred_calls_in_lifo_order_with_captured_args() {
    let source = r#"
package main
import "fmt"

func show(value int) {
    fmt.Println(value)
}

func main() {
    value := 1
    defer show(value)
    value = 2
    defer show(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n1\n");
}

#[test]
fn compiles_and_runs_a_multi_file_package() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "main.go",
                source: r#"
package main

func main() {
    helper()
}
"#,
            },
            SourceInput {
                path: "helper.go",
                source: r#"
package main
import "fmt"

func helper() {
    fmt.Println("from helper", 7)
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "from helper 7\n");
}

#[test]
fn compiles_and_runs_local_short_variable_declarations() {
    let source = r#"
package main
import "fmt"

func main() {
    label := "hello"
    value := 7
    fmt.Println(label, value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 7\n");
}

#[test]
fn compiles_and_runs_parameterized_user_functions() {
    let source = r#"
package main
import "fmt"

func greet(name string, count int) {
    fmt.Println(name, count)
}

func main() {
    greet("hello", 3)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 3\n");
}

#[test]
fn compiles_and_runs_parameterized_calls_with_local_arguments() {
    let source = r#"
package main
import "fmt"

func greet(name string, count int) {
    fmt.Println(name, count)
}

func main() {
    label := "hello"
    count := 4
    greet(label, count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 4\n");
}

#[test]
fn compiles_and_runs_assignment_to_existing_locals() {
    let source = r#"
package main
import "fmt"

func main() {
    label := "hello"
    value := 1
    label = "updated"
    value = 8
    fmt.Println(label, value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "updated 8\n");
}

#[test]
fn compiles_and_runs_assignment_from_parameter_values() {
    let source = r#"
package main
import "fmt"

func greet(name string, count int) {
    label := "start"
    label = name
    count = 9
    fmt.Println(label, count)
}

func main() {
    greet("hello", 3)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 9\n");
}

#[test]
fn compiles_and_runs_integer_addition() {
    let source = r#"
package main
import "fmt"

func main() {
    value := 2 + 3
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn compiles_and_runs_string_concatenation() {
    let source = r#"
package main
import "fmt"

func main() {
    value := "go" + "wasm"
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "gowasm\n");
}

#[test]
fn compiles_and_runs_integer_arithmetic() {
    let source = r#"
package main
import "fmt"

func main() {
    value := 10 - 3*2 + 8/4
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn compiles_and_runs_increment_and_decrement_statements() {
    let source = r#"
package main
import "fmt"

func main() {
    count := 1
    count++
    count--
    count++
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_unary_minus() {
    let source = r#"
package main
import "fmt"

func main() {
    value := -(2 + 3)
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-5\n");
}

#[test]
fn compiles_and_runs_boolean_comparisons() {
    let source = r#"
package main
import "fmt"

func main() {
    same := 2 + 3 == 5
    other := "go" != "wasm"
    ready := true == false
    fmt.Println(same, other, ready)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false\n");
}

#[test]
fn compiles_and_runs_ordered_comparisons() {
    let source = r#"
package main
import "fmt"

func main() {
    less := 3 < 4
    more := 5 >= 5
    text := "go" < "rust"
    fmt.Println(less, more, text)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\n");
}

#[test]
fn compiles_and_runs_unary_and_logical_boolean_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    ready := !false
    combined := true && ready
    fallback := false || combined
    fmt.Println(ready, combined, fallback)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\n");
}

#[test]
fn short_circuits_logical_boolean_expressions() {
    let source = r#"
package main
import "fmt"

func side() bool {
    fmt.Println("side")
    return true
}

func main() {
    left := false && side()
    right := true || side()
    fmt.Println(left, right)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false true\n");
}

#[test]
fn compiles_and_runs_single_value_returns() {
    let source = r#"
package main
import "fmt"

func answer() int {
    return 42
}

func main() {
    value := answer()
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn compiles_and_runs_parameterized_return_values() {
    let source = r#"
package main
import "fmt"

func echo(name string) string {
    return name
}

func main() {
    fmt.Println(echo("hello"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn compiles_and_runs_returned_addition_results() {
    let source = r#"
package main
import "fmt"

func add(left int, right int) int {
    return left + right
}

func main() {
    fmt.Println(add(4, 5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn rejects_duplicate_function_names_across_files() {
    let error = compile_workspace(
        &[
            SourceInput {
                path: "main.go",
                source: "package main\nfunc main() {}\n",
            },
            SourceInput {
                path: "helper.go",
                source: "package main\nfunc main() {}\n",
            },
        ],
        "main.go",
    )
    .expect_err("workspace should reject duplicates");

    assert!(error
        .to_string()
        .contains("function `main` is defined more than once"));
}

#[test]
fn ignores_files_from_other_packages_when_building_entry_package() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "main.go",
                source: "package main\nfunc main() { helper() }\n",
            },
            SourceInput {
                path: "helper.go",
                source: "package main\nimport \"fmt\"\nfunc helper() { fmt.Println(\"ok\") }\n",
            },
            SourceInput {
                path: "util.go",
                source: "package util\nfunc shadow() {}\n",
            },
        ],
        "main.go",
    )
    .expect("workspace should compile entry package only");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ok\n");
}

#[test]
fn rejects_short_redeclarations_without_new_names() {
    let error = compile_source(
        r#"
package main

func main() {
    value := 1
    value := 2
}
"#,
    )
    .expect_err("duplicate locals should fail");

    assert!(error
        .to_string()
        .contains("no new variables on the left side of `:=`"));
}

#[test]
fn rejects_assignment_to_unknown_names() {
    let error = compile_source(
        r#"
package main

func main() {
    value = 1
}
"#,
    )
    .expect_err("assignment to unknown local should fail");

    assert!(error
        .to_string()
        .contains("assignment target `value` is not defined"));
}
