use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn recursive_generic_functions_reuse_a_single_concrete_instance() {
    let source = r#"
package main
import "fmt"

func CountDown[T any](n int, value T) T {
    if n == 0 {
        return value
    }
    return CountDown[T](n-1, value)
}

func main() {
    fmt.Println(CountDown[int](4, 7))
}
"#;

    let program = compile_source(source).expect("program should compile");
    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.name == "CountDown[int]")
            .count(),
        1
    );
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn recursive_generic_struct_zero_values_compile() {
    let source = r#"
package main
import "fmt"

type Node[T any] struct {
    value T
    next *Node[T]
}

func main() {
    var node Node[int]
    fmt.Println(node.value, node.next == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 true\n");
}

#[test]
fn inferred_interface_constrained_generic_calls_compile() {
    let source = r#"
package main
import "fmt"

type Stringer interface {
    String() string
}

type Label struct {}

func (Label) String() string {
    return "label"
}

func Show[T Stringer](value T) string {
    return value.String()
}

func main() {
    fmt.Println(Show(Label{}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "label\n");
}

#[test]
fn comparable_constrained_generic_calls_reject_non_comparable_types() {
    let source = r#"
package main

type Bag struct {
    values map[string]int
}

func Identity[T comparable](value T) T {
    return value
}

func main() {
    _ = Identity(Bag{})
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("type argument `Bag` does not satisfy `T` (constraint `comparable`)"));
}
