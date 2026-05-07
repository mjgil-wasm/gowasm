use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_type_definition_conversion() {
    let source = r#"
package main
import "fmt"

type MyInt int

func main() {
    x := MyInt(42)
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn compiles_and_runs_type_definition_with_method() {
    let source = r#"
package main
import "fmt"

type MyInt int

func (m MyInt) Double() int {
    return int(m) * 2
}

func main() {
    x := MyInt(5)
    fmt.Println(x.Double())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10\n");
}

#[test]
fn compiles_and_runs_type_definition_string() {
    let source = r#"
package main
import "fmt"

type Name string

func main() {
    n := Name("hello")
    fmt.Println(n)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn compiles_and_runs_stringer_interface_with_println() {
    let source = r#"
package main
import "fmt"

type Color int

func (c Color) String() string {
    if int(c) == 0 {
        return "red"
    }
    return "blue"
}

func main() {
    c := Color(0)
    fmt.Println(c)
    fmt.Println(Color(1))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "red\nblue\n");
}

#[test]
fn compiles_and_runs_stringer_on_struct_with_println() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func (p Point) String() string {
    return fmt.Sprintf("(%d, %d)", p.x, p.y)
}

func main() {
    p := Point{x: 3, y: 7}
    fmt.Println(p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "(3, 7)\n");
}

#[test]
fn does_not_apply_stringer_when_no_string_method() {
    let source = r#"
package main
import "fmt"

type MyInt int

func main() {
    x := MyInt(42)
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn compiles_and_runs_stringer_with_sprint() {
    let source = r#"
package main
import "fmt"

type Color int

func (c Color) String() string {
    return "green"
}

func main() {
    c := Color(0)
    s := fmt.Sprintln(c)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "green\n\n");
}

#[test]
fn compiles_and_runs_alias_type_satisfies_interface() {
    let source = r#"
package main
import "fmt"

type Stringer interface {
    String() string
}

type Color int

func (c Color) String() string {
    return "red"
}

func describe(s Stringer) string {
    return s.String()
}

func main() {
    c := Color(0)
    fmt.Println(describe(c))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "red\n");
}

#[test]
fn compiles_and_runs_go_func_literal() {
    let source = r#"
package main
import "fmt"

func main() {
    done := make(chan int)
    go func() {
        fmt.Println("goroutine")
        done <- 1
    }()
    <-done
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "goroutine\nmain\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_on_alias_type() {
    let source = r#"
package main
import "fmt"

type Counter int

func (c *Counter) Increment() {
    *c = *c + 1
}

func main() {
    c := Counter(0)
    c.Increment()
    c.Increment()
    fmt.Println(int(c))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_string_from_int() {
    let source = r#"
package main
import "fmt"

func main() {
    s := string(65)
    fmt.Println(s)
    fmt.Println(string(72))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "A\nH\n");
}

#[test]
fn compiles_and_runs_int_from_float() {
    let source = r#"
package main
import "fmt"

func main() {
    x := int(3.14)
    fmt.Println(x)
    fmt.Println(int(9.99))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n9\n");
}

#[test]
fn compiles_and_runs_float_from_int() {
    let source = r#"
package main
import "fmt"

func main() {
    x := float64(42)
    fmt.Println(x)
    fmt.Println(float64(7) + 0.5)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42.0\n7.5\n");
}

#[test]
fn compiles_and_runs_byte_slice_from_string() {
    let source = r#"
package main
import "fmt"

func main() {
    b := []byte("hello")
    fmt.Println(b[0], b[1], len(b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "104 101 5\n");
}

#[test]
fn compiles_and_runs_string_from_byte_slice() {
    let source = r#"
package main
import "fmt"

func main() {
    b := []byte{72, 105}
    s := string(b)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Hi\n");
}
