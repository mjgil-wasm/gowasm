use crate::{compile_workspace, SourceInput};
use gowasm_vm::Vm;

use super::compile_source;

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

#[test]
fn nested_generic_function_values_assign_structurally() {
    let source = r#"
package main

import "fmt"

type Pair[T any, U any] struct {
    First T
    Second U
}

func project(values map[string]Pair[int, string]) Pair[int, string] {
    return values["item"]
}

func bounce(run func(map[string]Pair[int, string]) Pair[int, string]) func(map[string]Pair[int, string]) Pair[int, string] {
    return run
}

func wrap(
    run func(func(map[string]Pair[int, string]) Pair[int, string]) func(map[string]Pair[int, string]) Pair[int, string],
) func(map[string]Pair[int, string]) Pair[int, string] {
    return run(project)
}

func main() {
    run := wrap(bounce)
    value := run(map[string]Pair[int, string]{
        "item": Pair[int, string]{First: 7, Second: "go"},
    })
    fmt.Println(value.First, value.Second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go\n");
}

#[test]
fn nested_generic_function_values_reject_structural_result_mismatches() {
    let source = r#"
package main

type Pair[T any, U any] struct {
    First T
    Second U
}

func install(run func(map[string]Pair[int, string]) Pair[int, string]) {}

func wrong(values map[string]Pair[int, string]) Pair[string, int] {
    var pair Pair[string, int]
    return pair
}

func main() {
    install(wrong)
}
"#;

    let error = compile_source(source).expect_err("program should fail");
    assert!(format!("{error:?}").contains(
        "function value of type `func(map[string]Pair[int,string]) Pair[string,int]` is not assignable to `func(map[string]Pair[int,string]) Pair[int,string]`"
    ));
}

#[test]
fn method_values_assign_structurally() {
    let source = r#"
package main

import "fmt"

type Calculator struct {
    base int
}

func (c Calculator) Add(x int) int {
    return c.base + x
}

func main() {
    calc := Calculator{base: 10}
    var add func(int) int = calc.Add
    fmt.Println(add(5))
    fmt.Println(add(20))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15\n30\n");
}

#[test]
fn method_values_reject_structural_parameter_mismatches() {
    let source = r#"
package main

type Calculator struct {
    base int
}

func (c Calculator) Add(x int) int {
    return c.base + x
}

func main() {
    calc := Calculator{base: 10}
    var add func(string) int = calc.Add
    _ = add
}
"#;

    let error = compile_source(source).expect_err("program should fail");
    assert!(format!("{error:?}").contains(
        "function value of type `func(int) int` is not assignable to `func(string) int`"
    ));
}

#[test]
fn stdlib_function_values_assign_structurally() {
    let source = r#"
package main

import (
    "fmt"
    "net/url"
)

func install(run func(string) (url.Values, error), input string) (url.Values, error) {
    return run(input)
}

func main() {
    values, err := install(url.ParseQuery, "a=1&b=2")
    fmt.Println(values.Get("a"), values.Get("b"), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2 <nil>\n");
}

#[test]
fn stdlib_function_values_reject_structural_result_mismatches() {
    let source = r#"
package main

import "net/url"

var parse func(string) (map[string]string, error) = url.ParseQuery

func main() {
    _ = parse
}
"#;

    let error = compile_source(source).expect_err("program should fail");
    assert!(format!("{error:?}").contains(
        "function value of type `func(string) (url.Values, error)` is not assignable to `func(string) (map[string]string, error)`"
    ));
}

#[test]
fn imported_generic_type_function_values_assign_structurally() {
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
    var pair lib.Pair[int, string]
    pair.First = 13
    pair.Second = "imported"
    result := lib.ApplyPair(lib.Project, pair)
    fmt.Println(result.First, result.Second)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Pair[T any, U any] struct {
    First T
    Second U
}

func Project(value Pair[int, string]) Pair[int, string] {
    return value
}

func ApplyPair(run func(Pair[int, string]) Pair[int, string], value Pair[int, string]) Pair[int, string] {
    return run(value)
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "13 imported\n");
}

#[test]
fn imported_generic_type_function_values_reject_structural_mismatches() {
    let sources = [
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
    var pair lib.Pair[int, string]
    _ = lib.ApplyPair(lib.Project, pair)
}
"#,
        },
        SourceInput {
            path: "lib/lib.go",
            source: r#"
package lib

type Pair[T any, U any] struct {
    First T
    Second U
}

func Project(value Pair[string, int]) Pair[string, int] {
    return value
}

func ApplyPair(run func(Pair[int, string]) Pair[int, string], value Pair[int, string]) Pair[int, string] {
    return run(value)
}
"#,
        },
    ];

    let error = compile_workspace(&sources, "main.go").expect_err("program should fail");
    assert!(format!("{error:?}").contains(
        "function value of type `func(lib.Pair[string,int]) lib.Pair[string,int]` is not assignable to `func(lib.Pair[int,string]) lib.Pair[int,string]`"
    ));
}
