use super::compile_source;
use crate::CompileError;
use gowasm_vm::Vm;

#[test]
fn short_variable_declaration() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 42
    name := "alice"
    ok := true
    fmt.Println(x, name, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 alice true\n");
}

#[test]
fn multiple_short_declarations() {
    let source = r#"
package main
import "fmt"

func main() {
    a := 1
    b := 2
    fmt.Println(a, b)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n");
}

#[test]
fn var_reassignment() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 10
    fmt.Println(x)
    x = 20
    fmt.Println(x)
    x = x + 5
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10\n20\n25\n");
}

#[test]
fn var_shadowing_in_if_scope() {
    let source = r#"
package main
import "fmt"

func main() {
    x := "outer"
    if true {
        x := "inner"
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

#[test]
fn var_if_scope() {
    let source = r#"
package main
import "fmt"

func main() {
    if x := 10; x > 5 {
        fmt.Println("big:", x)
    }
    y := 3
    fmt.Println("y:", y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "big: 10\ny: 3\n");
}

#[test]
fn var_float64_zero_value() {
    let source = r#"
package main
import "fmt"

func main() {
    var f float64
    fmt.Println(f)
    f = 3.14
    fmt.Println(f)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n3.14\n");
}

#[test]
fn var_byte_zero_value() {
    let source = r#"
package main
import "fmt"

func main() {
    var b byte
    fmt.Println(b)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn var_rune_zero_value() {
    let source = r#"
package main
import "fmt"

func main() {
    var r rune
    fmt.Println(r)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn struct_float64_field_zero_value() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x float64
    y float64
}

func main() {
    var p Point
    fmt.Println(p.x, p.y)
    p.x = 1.5
    p.y = 2.5
    fmt.Println(p.x, p.y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0 0.0\n1.5 2.5\n");
}

#[test]
fn var_error_zero_value() {
    let source = r#"
package main
import "fmt"

func main() {
    var err error
    fmt.Println(err)
    fmt.Println(err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil>\ntrue\n");
}

#[test]
fn var_string_zero_value_is_empty() {
    let source = r#"
package main
import "fmt"

func main() {
    var s string
    fmt.Println(len(s))
    fmt.Println(s == "")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\ntrue\n");
}

#[test]
fn short_decl_with_function_result() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.Atoi("123")
    fmt.Println(n, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "123 <nil>\n");
}

#[test]
fn short_decl_rebinds_existing_with_new_name() {
    let source = r#"
package main
import "fmt"

func pair() (int, string) {
    return 1, "new"
}

func main() {
    x := 0
    x, y := pair()
    fmt.Println(x, y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 new\n");
}

#[test]
fn short_decl_allows_blank_discard_when_name_is_new() {
    let source = r#"
package main
import "fmt"

func pair() (int, string) {
    return 7, "ignored"
}

func main() {
    x, _ := pair()
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn short_decl_rejects_all_blank_if_no_new_name() {
    let source = r#"
package main

func pair() (int, string) {
    return 1, "x"
}

func main() {
    x := 0
    x, _ = pair()
    x, _ := pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(
                detail.contains("no new variables on the left side of `:=` in the current scope")
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn increment_and_decrement() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 5
    x++
    fmt.Println(x)
    x--
    x--
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n4\n");
}
