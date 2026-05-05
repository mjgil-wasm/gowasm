use super::{compile_workspace, SourceInput};
use gowasm_vm::Vm;

#[test]
fn reuses_generic_function_instances_across_package_files() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "identity.go",
                source: r#"
package main

func Identity[T any](x T) T { return x }

func fromHelper() int {
    return Identity[int](7)
}
"#,
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "fmt"

func main() {
    fmt.Println(Identity[int](1), fromHelper())
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.name == "Identity[int]")
            .count(),
        1
    );

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 7\n");
}

#[test]
fn reuses_generic_type_method_instances_across_package_files() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "box.go",
                source: r#"
package main

type Box[T any] struct {
    value T
}

func (b Box[T]) Value() T {
    return b.value
}

func fromHelper() int {
    var box Box[int]
    box.value = 7
    return box.Value()
}
"#,
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "fmt"

func main() {
    var box Box[int]
    box.value = 1
    fmt.Println(box.Value(), fromHelper())
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.name == "Box[int].Value")
            .count(),
        1
    );

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 7\n");
}
