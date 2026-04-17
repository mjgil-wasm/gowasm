use super::{compile_source, compile_workspace, SourceInput};
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_inline_interface_constraints() {
    let source = r#"
package main
import "fmt"

type Label int

func (l Label) String() string {
    return "label"
}

func Show[T interface {
    comparable
    String() string
}](value T) string {
    return value.String()
}

func main() {
    fmt.Println(Show(Label(7)))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "label\n");
}

#[test]
fn compiles_and_runs_inline_embedded_constraints_and_type_sets() {
    let source = r#"
package main
import "fmt"

type Stringer interface {
    String() string
}

type Label string

func (l Label) String() string {
    return string(l)
}

func Pick[T interface {
    Stringer
    comparable
}](value T) string {
    return value.String()
}

func Echo[T interface { int | string }](value T) T {
    return value
}

func main() {
    fmt.Println(Pick(Label("ok")))
    fmt.Println(Echo(7))
    fmt.Println(Echo("go"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ok\n7\ngo\n");
}

#[test]
fn rejects_inline_constraint_type_set_mismatches() {
    let source = r#"
package main

func Echo[T interface { int | string }](value T) T {
    return value
}

func main() {
    _ = Echo(true)
}
"#;

    let error = compile_source(source).expect_err("bool should fail the type set");
    assert!(error.to_string().contains("type set `int | string`"));
}

#[test]
fn rejects_inline_embedded_comparable_mismatches() {
    let source = r#"
package main

type Named interface {
    String() string
}

type Bag struct {
    values map[string]int
}

func (Bag) String() string {
    return "bag"
}

func Show[T interface {
    Named
    comparable
}](value T) string {
    return value.String()
}

func main() {
    _ = Show(Bag{})
}
"#;

    let error = compile_source(source).expect_err("non-comparable type should fail");
    assert!(error.to_string().contains("constraint `comparable`"));
}

#[test]
fn compiles_and_runs_imported_inline_constraints() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import (
    "example.com/app/lib"
    "fmt"
)

func main() {
    fmt.Println(lib.Echo[string]("imported"))
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

func Echo[T interface {
    comparable
    int | string
}](value T) T {
    return value
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "imported\n");
}

#[test]
fn compiles_and_runs_recursive_generic_types_with_inline_constraints() {
    let source = r#"
package main
import "fmt"

type Node[T interface { int | string }] struct {
    value T
    next *Node[T]
}

func main() {
    var node Node[int]
    node.value = 11
    fmt.Println(node.value, node.next == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "11 true\n");
}
