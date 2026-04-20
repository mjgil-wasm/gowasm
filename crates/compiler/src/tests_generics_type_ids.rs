use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn generic_type_assertions_accept_instantiated_struct_targets() {
    let source = r#"
package main
import "fmt"

type Any interface {}

type Box[T any] struct {
    value T
}

func main() {
    var value Any
    var box Box[int]
    box.value = 7
    value = box
    typed, ok := value.(Box[int])
    value = 1
    missing, missingOk := value.(Box[int])
    fmt.Println(typed.value, ok, missing.value, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 true 0 false\n");
}

#[test]
fn generic_type_switches_match_distinct_instantiated_struct_type_ids() {
    let source = r#"
package main
import "fmt"

type Any interface {}

type Box[T any] struct {
    value T
}

func describe(value Any) {
    switch typed := value.(type) {
    case Box[int]:
        fmt.Println("int", typed.value)
    case Box[string]:
        fmt.Println("string", typed.value)
    default:
        fmt.Println("other")
    }
}

func main() {
    var intBox Box[int]
    intBox.value = 3
    var stringBox Box[string]
    stringBox.value = "hi"
    describe(intBox)
    describe(stringBox)
    describe(true)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "int 3\nstring hi\nother\n");
}

#[test]
fn generic_interface_assertions_and_deferred_dispatch_use_instantiated_method_sets() {
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

func (b Box[T]) Value() T {
    return b.value
}

func main() {
    var box Box[int]
    box.value = 9
    var value Any
    value = box
    reader, ok := value.(Reader[int])
    fmt.Println(ok)
    defer fmt.Println(reader.Value())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n9\n");
}
