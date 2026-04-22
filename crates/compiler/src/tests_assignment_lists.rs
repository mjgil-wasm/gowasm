use gowasm_vm::Vm;

use crate::{compile_source, CompileError};

#[test]
fn compiles_and_runs_direct_assignment_expression_lists() {
    let source = r#"
package main
import "fmt"

func left() int { return 1 }
func right() string { return "go" }
func nextLeft() int { return 2 }
func nextRight() string { return "wasm" }

func main() {
    number, word := left(), right()
    fmt.Println(number, word)
    number, word = nextLeft(), nextRight()
    fmt.Println(number, word)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 go\n2 wasm\n");
}

#[test]
fn compiles_and_runs_tail_multi_result_assignment_expression_lists() {
    let source = r#"
package main
import "fmt"

func prefix() string { return "left" }
func nextPrefix() string { return "right" }
func pair() (int, string) { return 7, "go" }
func nextPair() (int, string) { return 9, "wasm" }

func main() {
    label, number, word := prefix(), pair()
    fmt.Println(label, number, word)
    label, number, word = nextPrefix(), nextPair()
    fmt.Println(label, number, word)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "left 7 go\nright 9 wasm\n");
}

#[test]
fn compiles_and_runs_tail_comma_ok_assignment_expression_lists() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 7}
    var any interface{} = 11
    ch := make(chan string, 1)
    ch <- "ready"
    close(ch)

    label, number, found := "map", values["go"]
    fmt.Println(label, number, found)
    label, number, found = "map", values["missing"]
    fmt.Println(label, number, found)

    label, asserted, ok := "type", any.(int)
    fmt.Println(label, asserted, ok)

    label, received, recvOk := "chan", <-ch
    fmt.Println(label, received, recvOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "map 7 true\nmap 0 false\ntype 11 true\nchan ready true\n"
    );
}

#[test]
fn rejects_assignment_expression_list_count_mismatch_with_too_many_values() {
    let source = r#"
package main

func pair() (int, string) { return 7, "go" }

func main() {
    first, second := 1, pair()
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("assignment value count 3 does not match 2 target"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_assignment_expression_list_count_mismatch_with_too_few_values() {
    let source = r#"
package main

func main() {
    var first int
    var second int
    first, second = 1
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains(
                "cannot infer result 2 type for multi-result assignment in the current subset"
            ));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_non_final_multi_result_calls_in_assignment_expression_lists() {
    let source = r#"
package main

func pair() (int, string) { return 7, "go" }

func main() {
    first, second, third := pair(), 9
}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains(
                "multi-result call expressions must appear in the final assignment position"
            ));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
