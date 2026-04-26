use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_expression_switch_with_multi_value_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    value := 3
    switch value {
    case 1:
        fmt.Println("one")
    case 2, 3:
        fmt.Println("match")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "match\n");
}

#[test]
fn runs_switch_default_when_no_case_matches() {
    let source = r#"
package main
import "fmt"

func main() {
    switch 9 {
    case 1:
        fmt.Println("one")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "default\n");
}

#[test]
fn compiles_and_runs_expressionless_switch_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    value := 2
    switch {
    case value < 0:
        fmt.Println("negative")
    case value == 2:
        fmt.Println("two")
    default:
        fmt.Println("other")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "two\n");
}

#[test]
fn break_exits_switch_without_running_the_rest_of_the_case() {
    let source = r#"
package main
import "fmt"

func main() {
    switch 2 {
    case 2:
        fmt.Println("start")
        break
        fmt.Println("after")
    default:
        fmt.Println("default")
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "start\ndone\n");
}

#[test]
fn continue_inside_switch_targets_the_enclosing_loop() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := 0; i < 3; i++ {
        switch i {
        case 1:
            continue
        }
        fmt.Println(i)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n2\n");
}

#[test]
fn compiles_and_runs_switch_fallthrough() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 1
    switch x {
    case 1:
        fmt.Println("one")
        fallthrough
    case 2:
        fmt.Println("two")
        fallthrough
    case 3:
        fmt.Println("three")
    case 4:
        fmt.Println("four")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "one\ntwo\nthree\n");
}

#[test]
fn compiles_and_runs_switch_fallthrough_to_default() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 5
    switch x {
    case 5:
        fmt.Println("five")
        fallthrough
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "five\ndefault\n");
}

#[test]
fn compiles_and_runs_default_fallthrough_to_a_later_case() {
    let source = r#"
package main
import "fmt"

func main() {
    switch 2 {
    default:
        fmt.Println("default")
        fallthrough
    case 9:
        fmt.Println("nine")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "default\nnine\n");
}

#[test]
fn rejects_fallthrough_in_the_final_switch_clause() {
    let error = compile_source(
        r#"
package main

func main() {
    switch 1 {
    case 1:
        fallthrough
    }
}
"#,
    )
    .expect_err("final switch clause fallthrough should fail");

    assert!(error
        .to_string()
        .contains("`fallthrough` cannot appear in the final `switch` clause"));
}

#[test]
fn compiles_and_runs_labeled_break_from_inner_loop() {
    let source = r#"
package main
import "fmt"

func main() {
    outer:
    for i := 0; i < 3; i++ {
        for j := 0; j < 3; j++ {
            if j == 1 {
                break outer
            }
            fmt.Println(i, j)
        }
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 0\ndone\n");
}

#[test]
fn compiles_and_runs_labeled_continue_from_inner_loop() {
    let source = r#"
package main
import "fmt"

func main() {
    outer:
    for i := 0; i < 3; i++ {
        for j := 0; j < 3; j++ {
            if j == 1 {
                continue outer
            }
            fmt.Println(i, j)
        }
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 0\n1 0\n2 0\ndone\n");
}

#[test]
fn compiles_and_runs_labeled_break_from_switch_inside_loop() {
    let source = r#"
package main
import "fmt"

func main() {
    outer:
    for i := 0; i < 3; i++ {
        switch i {
        case 1:
            break outer
        default:
            fmt.Println(i)
        }
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\ndone\n");
}

#[test]
fn compiles_and_runs_type_switch_int() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func describe(val Any) string {
    switch v := val.(type) {
    case int:
        return fmt.Sprintf("int: %d", v)
    case string:
        return fmt.Sprintf("string: %s", v)
    case bool:
        return fmt.Sprintf("bool: %t", v)
    default:
        return "unknown"
    }
}

func main() {
    fmt.Println(describe(42))
    fmt.Println(describe("hello"))
    fmt.Println(describe(true))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "int: 42\nstring: hello\nbool: true\n");
}

#[test]
fn compiles_and_runs_type_switch_with_default() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func check(val Any) {
    switch v := val.(type) {
    case int:
        fmt.Println("int", v)
    default:
        fmt.Println("other")
    }
}

func main() {
    check(10)
    check("text")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "int 10\nother\n");
}

#[test]
fn compiles_and_runs_type_switch_multi_type_case() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func check(val Any) {
    switch val.(type) {
    case int, bool:
        fmt.Println("numeric-ish")
    case string:
        fmt.Println("string")
    }
}

func main() {
    check(1)
    check(true)
    check("hi")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "numeric-ish\nnumeric-ish\nstring\n");
}

#[test]
fn compiles_and_runs_type_switch_with_init_statement() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func main() {
    var value Any = 7
    switch boxed := value; typed := boxed.(type) {
    case int:
        fmt.Println(typed, boxed == value)
    default:
        fmt.Println("other")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 true\n");
}

#[test]
fn compiles_and_runs_switch_with_init_statement() {
    let source = r#"
package main
import "fmt"

func main() {
    switch x := 2 + 3; x {
    case 5:
        fmt.Println("five")
    default:
        fmt.Println("other")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "five\n");
}

#[test]
fn compiles_and_runs_switch_init_with_function_call() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    switch n, err := strconv.Atoi("99"); {
    case err != nil:
        fmt.Println("error")
    case n > 50:
        fmt.Println("big", n)
    default:
        fmt.Println("small")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "big 99\n");
}

#[test]
fn compiles_and_runs_switch_init_scoping() {
    let source = r#"
package main
import "fmt"

func main() {
    x := "outer"
    switch x := "inner"; x {
    case "inner":
        fmt.Println(x)
    }
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner\nouter\n");
}
