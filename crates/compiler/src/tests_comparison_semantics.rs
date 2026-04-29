use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_array_equality() {
    let source = r#"
package main
import "fmt"

func main() {
    a := [2]int{1, 2}
    b := [2]int{1, 2}
    c := [2]int{2, 1}
    fmt.Println(a == b)
    fmt.Println(a != c)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\n");
}

#[test]
fn rejects_slice_equality_outside_nil_checks() {
    let source = r#"
package main

func main() {
    a := []int{1}
    b := []int{1}
    _ = a == b
}
"#;

    let error = compile_source(source).expect_err("slice equality should fail");
    assert!(error.to_string().contains("type `[]int` is not comparable"));
}

#[test]
fn rejects_map_equality_outside_nil_checks() {
    let source = r#"
package main

func main() {
    a := map[string]int{"go": 1}
    b := map[string]int{"go": 1}
    _ = a == b
}
"#;

    let error = compile_source(source).expect_err("map equality should fail");
    assert!(error
        .to_string()
        .contains("type `map[string]int` is not comparable"));
}

#[test]
fn rejects_function_equality_outside_nil_checks() {
    let source = r#"
package main

func main() {
    a := func() int { return 1 }
    b := func() int { return 1 }
    _ = a == b
}
"#;

    let error = compile_source(source).expect_err("function equality should fail");
    assert!(error
        .to_string()
        .contains("type `func() int` is not comparable"));
}

#[test]
fn rejects_struct_equality_with_non_comparable_fields() {
    let source = r#"
package main

type Bag struct {
    values []int
}

func main() {
    a := Bag{values: []int{1}}
    b := Bag{values: []int{1}}
    _ = a == b
}
"#;

    let error = compile_source(source).expect_err("struct equality should fail");
    assert!(error.to_string().contains("type `Bag` is not comparable"));
}

#[test]
fn rejects_array_equality_with_non_comparable_elements() {
    let source = r#"
package main

func main() {
    a := [1][]int{[]int{1}}
    b := [1][]int{[]int{1}}
    _ = a == b
}
"#;

    let error = compile_source(source).expect_err("array equality should fail");
    assert!(error.to_string().contains("not comparable"));
}

#[test]
fn rejects_ordered_comparisons_for_unordered_types() {
    let source = r#"
package main

func main() {
    _ = true < false
}
"#;

    let error = compile_source(source).expect_err("bool ordering should fail");
    assert!(error.to_string().contains("type `bool` is not ordered"));
}

#[test]
fn rejects_interface_comparison_with_non_comparable_concrete_values() {
    let source = r#"
package main

func main() {
    var value interface{}
    other := []int{1}
    _ = value == other
}
"#;

    let error = compile_source(source).expect_err("interface vs slice equality should fail");
    assert!(error.to_string().contains("type `[]int` is not comparable"));
}

#[test]
fn compiles_and_runs_interface_comparison_with_different_dynamic_types() {
    let source = r#"
package main
import "fmt"

func main() {
    var left interface{} = []int{1}
    var right interface{} = 1
    fmt.Println(left == right)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn rejects_interface_comparison_with_non_comparable_dynamic_values_at_runtime() {
    let source = r#"
package main

func main() {
    var left interface{} = []int{1}
    var right interface{} = []int{1}
    _ = left == right
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("dynamic slice comparison should fail at runtime");
    assert!(error.to_string().contains("unsupported operands for `==`"));
}

#[test]
fn compiles_and_runs_generic_comparable_equality() {
    let source = r#"
package main
import "fmt"

func Equal[T comparable](left T, right T) bool {
    return left == right
}

func main() {
    fmt.Println(Equal(3, 3))
    fmt.Println(Equal("go", "wasm"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}
