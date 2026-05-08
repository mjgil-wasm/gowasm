use crate::{compile_workspace, SourceInput};

use super::compile_source;
use gowasm_vm::Vm;

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

#[test]
fn rejects_ambiguous_promoted_field_selectors() {
    let source = r#"
package main

type Left struct {
    Name string
}

type Right struct {
    Name string
}

type Outer struct {
    Left
    Right
}

func main() {
    value := Outer{}
    _ = value.Name
}
"#;

    let error = compile_source(source).expect_err("selector should be ambiguous");
    assert!(
        error
            .to_string()
            .contains("ambiguous promoted field selector `Outer.Name`"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_ambiguous_promoted_method_selectors() {
    let source = r#"
package main

type Left struct {}
type Right struct {}

func (Left) Speak() string { return "left" }
func (Right) Speak() string { return "right" }

type Outer struct {
    Left
    Right
}

func main() {
    value := Outer{}
    _ = value.Speak()
}
"#;

    let error = compile_source(source).expect_err("method selector should be ambiguous");
    assert!(
        error
            .to_string()
            .contains("ambiguous promoted method selector `Outer.Speak`"),
        "unexpected error: {error}"
    );
}

#[test]
fn direct_methods_shadow_promoted_methods() {
    let source = r#"
package main
import "fmt"

type Inner struct {}

func (Inner) Speak() string { return "inner" }

type Outer struct {
    Inner
}

func (Outer) Speak() string { return "outer" }

func main() {
    value := Outer{}
    fmt.Println(value.Speak(), value.Inner.Speak())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "outer inner\n");
}

#[test]
fn compiles_and_runs_imported_promoted_methods() {
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
    "fmt"
    "example.com/app/lib"
)

func main() {
    value := lib.Make("ada")
    fmt.Println(value.Speak())
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Inner struct {
    Label string
}

func (inner Inner) Speak() string {
    return "lib:" + inner.Label
}

type Outer struct {
    Inner
}

func Make(label string) Outer {
    return Outer{Inner: Inner{Label: label}}
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "lib:ada\n");
}

#[test]
fn compiles_and_runs_generic_receiver_methods() {
    let source = r#"
package main
import "fmt"

type Box[T any] struct {
    Value T
}

func (box Box[T]) Speak() string {
    return fmt.Sprint(box.Value)
}

func main() {
    value := Box[int]{Value: 7}
    fmt.Println(value.Speak())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}
