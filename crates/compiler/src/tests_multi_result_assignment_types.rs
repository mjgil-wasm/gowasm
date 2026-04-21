use gowasm_vm::Vm;

use crate::{compile_source, CompileError};

#[test]
fn rejects_global_multi_result_assignment_type_mismatch() {
    let source = r#"
package main

var first int
var second bool

func pair() (string, bool) { return "go", true }

func main() {
    first, second = pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("value of type `string` is not assignable to `int`"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_existing_local_short_decl_rebind_type_mismatch() {
    let source = r#"
package main

func pair() (string, bool) { return "go", true }

func main() {
    first := 0
    first, ok := pair()
    _ = ok
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("value of type `string` is not assignable to `int`"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_deref_multi_result_assignment_type_mismatch() {
    let source = r#"
package main

func pair() (string, bool) { return "go", true }

func main() {
    value := 0
    ptr := &value
    var ok bool
    *ptr, ok = pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("value of type `string` is not assignable to `int`"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_selector_named_type_mismatch_in_multi_result_assignment() {
    let source = r#"
package main

type Label string

type Box struct {
    Name Label
    Ok bool
}

func pair() (string, bool) { return "go", true }

func main() {
    box := Box{}
    box.Name, box.Ok = pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("value of type `string` is not assignable to `Label`"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_index_interface_mismatch_in_multi_result_assignment() {
    let source = r#"
package main

type Shape interface {
    Area() int
}

func pair() (int, bool) { return 7, true }

func main() {
    values := make([]Shape, 1)
    var ok bool
    values[0], ok = pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("type `int` does not satisfy interface `Shape`"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn allows_blank_identifier_discards_in_multi_result_assignment_validation() {
    let source = r#"
package main
import "fmt"

func pair() (string, int) { return "go", 7 }

func main() {
    var value int
    _, value = pair()
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}
