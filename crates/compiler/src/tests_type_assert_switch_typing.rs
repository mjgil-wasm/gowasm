use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn pointer_type_assertions_preserve_typed_nil_pointers() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Box struct { value int }

func main() {
    var ptr *Box
    var value Any = ptr
    typed, ok := value.(*Box)
    fmt.Println(ok, typed == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\n");
}

#[test]
fn nil_interfaces_do_not_match_interface_assertions_or_non_nil_type_switch_cases() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Named interface { Name() string }

type Box struct { value int }

func main() {
    var value Any
    _, anyOk := value.(Any)
    _, namedOk := value.(Named)
    fmt.Println(anyOk, namedOk)

    switch typed := value.(type) {
    case Any:
        fmt.Println("any", typed == nil)
    case nil:
        fmt.Println("nil")
    default:
        fmt.Println("default")
    }

    var ptr *Box
    value = ptr
    switch typed := value.(type) {
    case nil:
        fmt.Println("wrong")
    case *Box:
        fmt.Println(typed == nil)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false false\nnil\ntrue\n");
}

#[test]
fn single_type_switch_interface_cases_narrow_bindings() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Shape interface { Area() int }
type Point struct { x int }

func (point Point) Area() int { return point.x }

func main() {
    var value Any = Point{x: 7}
    switch shape := value.(type) {
    case Shape:
        fmt.Println(shape.Area())
    default:
        fmt.Println("other")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn generic_interface_type_switch_cases_narrow_bindings() {
    let source = r#"
package main
import "fmt"

type Any interface {}

type Reader[T any] interface {
    Value() T
}

type Box[T any] struct {
    value T
}

func (box Box[T]) Value() T {
    return box.value
}

func main() {
    var value Any = Box[int]{value: 9}
    switch reader := value.(type) {
    case Reader[int]:
        fmt.Println(reader.Value())
    default:
        fmt.Println("other")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}
