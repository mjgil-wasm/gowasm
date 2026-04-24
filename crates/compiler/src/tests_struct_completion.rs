use super::compile_source;
use crate::{compile_workspace, SourceInput};
use gowasm_vm::Vm;

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("workspace should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("workspace should run");
    vm.stdout().to_string()
}

#[test]
fn compiles_and_runs_generic_struct_literals_with_nested_pointer_fields() {
    let source = r#"
package main
import "fmt"

type Pair[T any] struct {
    Left T
    Right T
}

type Node[T any] struct {
    Value T
    Next *Node[T]
    Pair Pair[T]
}

func main() {
    tail := Node[int]{Value: 2, Pair: Pair[int]{Left: 20, Right: 21}}
    head := Node[int]{Value: 1, Next: &tail, Pair: Pair[int]{Left: 10, Right: 11}}
    fmt.Println(head.Value, head.Next.Value, head.Pair.Right, head.Next.Pair.Left)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2 11 20\n");
}

#[test]
fn compiles_and_runs_imported_struct_literals_assignment_and_json_reflect_interaction() {
    let output = run_workspace(
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
    "encoding/json"
    "example.com/app/lib"
    "fmt"
    "reflect"
)

func main() {
    value := lib.Payload{Name: "Ada"}
    value.Name = "Go"
    payload, err := json.Marshal(value)
    field := reflect.TypeOf(value).Field(0)
    fmt.Println(string(payload), err)
    fmt.Println(field.Name, field.PkgPath == "")
    fmt.Println(reflect.ValueOf(value).Field(0).String())
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Payload struct {
    Name string
    hidden int
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "{\"Name\":\"Go\"} <nil>\nName true\nGo\n");
}

#[test]
fn compiles_and_runs_imported_promoted_exported_field_selectors() {
    let output = run_workspace(
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
    value := lib.MakeOuter()
    fmt.Println(value.Visible)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Inner struct {
    Visible string
    hidden string
}

type Outer struct {
    Inner
}

func MakeOuter() Outer {
    return Outer{Inner: Inner{Visible: "ok", hidden: "no"}}
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "ok\n");
}

#[test]
fn rejects_imported_struct_keyed_literals_using_unexported_fields() {
    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {
    _ = lib.Payload{hidden: 1}
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Payload struct {
    Name string
    hidden int
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("workspace should fail");

    assert!(
        error.to_string().contains(
            "cannot use unexported field `hidden` in imported struct literal `lib.Payload`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_imported_struct_positional_literals_when_any_field_is_unexported() {
    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {
    _ = lib.Payload{"Ada", 1}
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Payload struct {
    Name string
    hidden int
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("workspace should fail");

    assert!(
        error.to_string().contains(
            "cannot use unexported field `hidden` in imported struct literal `lib.Payload`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_imported_struct_selector_reads_of_unexported_fields() {
    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {
    value := lib.NewPayload()
    _ = value.hidden
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Payload struct {
    Name string
    hidden int
}

func NewPayload() Payload {
    return Payload{Name: "Ada", hidden: 7}
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("workspace should fail");

    assert!(
        error
            .to_string()
            .contains("cannot access unexported field selector `lib.Payload.hidden`"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_imported_struct_selector_writes_of_unexported_fields() {
    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {
    value := lib.NewPayload()
    value.hidden = 9
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Payload struct {
    Name string
    hidden int
}

func NewPayload() Payload {
    return Payload{Name: "Ada", hidden: 7}
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("workspace should fail");

    assert!(
        error
            .to_string()
            .contains("cannot access unexported field selector `lib.Payload.hidden`"),
        "unexpected error: {error}"
    );
}
