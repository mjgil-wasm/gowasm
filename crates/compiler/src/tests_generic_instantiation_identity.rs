use super::{compile_source, compile_workspace, module_cache_source_path, SourceInput};
use gowasm_vm::{program_type_inventory, Vm};

fn inventory_display_name_count(program: &gowasm_vm::Program, display_name: &str) -> usize {
    program_type_inventory(program)
        .expect("compiled program should register type inventory")
        .types_by_id
        .values()
        .filter(|info| info.display_name == display_name)
        .count()
}

#[test]
fn reuses_imported_generic_function_instances_across_sibling_packages() {
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
    "fmt"
    "example.com/app/left"
    "example.com/app/right"
)

func main() {
    fmt.Println(left.Value(), right.Value())
}
"#,
            },
            SourceInput {
                path: "left/left.go",
                source: r#"
package left

import "example.com/app/lib"

func Value() int {
    return lib.Identity[int](7)
}
"#,
            },
            SourceInput {
                path: "right/right.go",
                source: r#"
package right

import "example.com/app/lib"

func Value() int {
    return lib.Identity[int](9)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

func Identity[T any](value T) T {
    return value
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
            .filter(|function| function.name == "example.com/app/lib.Identity[int]")
            .count(),
        1
    );

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 9\n");
}

#[test]
fn imported_generic_type_identity_stays_shared_across_sibling_packages() {
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
    "example.com/app/left"
    "example.com/app/lib"
    "example.com/app/right"
    "fmt"
)

func main() {
    leftValue := left.MakeInt(3)
    rightValue := right.MakeInt(4)
    stringValue := right.MakeString("go")

    anyValue := left.Wrap(leftValue)
    typed, ok := anyValue.(lib.Box[int])
    anyStringValue := right.WrapString(stringValue)
    _, wrongType := anyStringValue.(lib.Box[int])

    fmt.Println(ok, typed.Value)
    fmt.Println(wrongType)
}
"#,
            },
            SourceInput {
                path: "left/left.go",
                source: r#"
package left

import "example.com/app/lib"

func MakeInt(value int) lib.Box[int] {
    return lib.Box[int]{Value: value}
}

func Wrap(value lib.Box[int]) interface{} {
    return value
}
"#,
            },
            SourceInput {
                path: "right/right.go",
                source: r#"
package right

import "example.com/app/lib"

func MakeInt(value int) lib.Box[int] {
    return lib.Box[int]{Value: value}
}

func MakeString(value string) lib.Box[string] {
    return lib.Box[string]{Value: value}
}

func WrapString(value lib.Box[string]) interface{} {
    return value
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Box[T any] struct {
    Value T `json:"value"`
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 3\nfalse\n");
}

#[test]
fn reuses_remote_module_generic_function_instances_across_packages() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "lib/lib.go");
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
    "fmt"
    "example.com/app/left"
    "example.com/app/right"
)

func main() {
    fmt.Println(left.Value(), right.Value())
}
"#,
            },
            SourceInput {
                path: "left/left.go",
                source: r#"
package left

import "example.com/remote/lib"

func Value() int {
    return lib.Identity[int](11)
}
"#,
            },
            SourceInput {
                path: "right/right.go",
                source: r#"
package right

import "example.com/remote/lib"

func Value() int {
    return lib.Identity[int](13)
}
"#,
            },
            SourceInput {
                path: module_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: module_go_file_path.as_str(),
                source: r#"
package lib

func Identity[T any](value T) T {
    return value
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
            .filter(|function| function.name == "example.com/remote/lib.Identity[int]")
            .count(),
        1
    );

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "11 13\n");
}

#[test]
fn reflect_and_json_keep_local_generic_instantiation_identity_distinct() {
    let source = r#"
package main

import (
    "encoding/json"
    "fmt"
    "reflect"
)

type Box[T any] struct {
    Value T `json:"value"`
}

type Payload struct {
    IntBox Box[int] `json:"int_box"`
    StringBox Box[string] `json:"string_box"`
}

func main() {
    intValue := Box[int]{Value: 5}
    otherIntValue := Box[int]{Value: 8}
    stringValue := Box[string]{Value: "go"}
    var anyValue interface{}
    anyValue = intValue
    typed, ok := anyValue.(Box[int])
    _, wrongType := anyValue.(Box[string])
    blob, err := json.Marshal(Payload{IntBox: intValue, StringBox: stringValue})
    var decoded Payload
    decodeErr := json.Unmarshal(blob, &decoded)
    fmt.Println(reflect.TypeOf(intValue) == reflect.TypeOf(otherIntValue))
    fmt.Println(reflect.TypeOf(intValue) != reflect.TypeOf(stringValue))
    fmt.Println(ok, typed.Value, wrongType)
    fmt.Println(string(blob), err == nil, decodeErr == nil, decoded.IntBox.Value, decoded.StringBox.Value)
}
"#;

    let program = compile_source(source).expect("program should compile");

    assert_eq!(inventory_display_name_count(&program, "Box[int]"), 1);
    assert_eq!(inventory_display_name_count(&program, "Box[string]"), 1);

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue\ntrue 5 false\n{\"int_box\":{\"value\":5},\"string_box\":{\"value\":\"go\"}} true true 5 go\n"
    );
}
