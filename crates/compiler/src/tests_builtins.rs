use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_len_on_strings_and_collections() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3}
    table := map[string]int{"go": 1, "wasm": 2}
    fmt.Println(len("go"), len(values), len([2]int{4, 5}), len(table))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 3 2 2\n");
}

#[test]
fn compiles_and_runs_len_on_zero_value_maps() {
    let source = r#"
package main
import "fmt"

func main() {
    var values map[string]int
    fmt.Println(len(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn len_respects_local_shadowing() {
    let source = r#"
package main

func main() {
    len := 7
    len("go")
}
"#;

    let error = compile_source(source).expect_err("shadowed len should not resolve as builtin");
    assert!(error
        .to_string()
        .contains("calling local variable `len` is not supported"));
}

#[test]
fn compiles_and_runs_append_on_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2}
    values = append(values, 3, 4)
    fmt.Println(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4]\n");
}

#[test]
fn compiles_and_runs_append_on_zero_value_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    var values []int
    values = append(values, 7)
    fmt.Println(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[7]\n");
}

#[test]
fn compiles_and_runs_cap_on_arrays_and_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3}
    fmt.Println(cap(values), cap([2]int{4, 5}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 2\n");
}

#[test]
fn cap_respects_local_shadowing() {
    let source = r#"
package main

func main() {
    cap := 7
    cap([]int{1})
}
"#;

    let error = compile_source(source).expect_err("shadowed cap should not resolve as builtin");
    assert!(error
        .to_string()
        .contains("calling local variable `cap` is not supported"));
}

#[test]
fn rejects_len_calls_with_the_wrong_arity() {
    let source = r#"
package main

func main() {
    len()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`len` expects 1 argument(s), found 0"));
}

#[test]
fn rejects_cap_calls_with_the_wrong_arity() {
    let source = r#"
package main

func main() {
    cap()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`cap` expects 1 argument(s), found 0"));
}

#[test]
fn rejects_append_calls_with_the_wrong_arity() {
    let source = r#"
package main

func main() {
    append()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`append` expects at least 1 argument in the current subset"));
}

#[test]
fn rejects_len_calls_with_the_wrong_argument_type() {
    let source = r#"
package main

func main() {
    len(1)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `len` has type `int`, expected string, array, slice, map, or channel"));
}

#[test]
fn rejects_cap_calls_with_the_wrong_argument_type() {
    let source = r#"
package main

func main() {
    cap("go")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `cap` has type `string`, expected array, slice, or channel"));
}

#[test]
fn compiles_len_and_cap_on_channels() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int, 3)
    fmt.Println(len(ch), cap(ch))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 3\n");
}

#[test]
fn compiles_len_and_cap_on_nil_channel() {
    let source = r#"
package main
import "fmt"

func main() {
    var ch chan int
    fmt.Println(len(ch), cap(ch))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 0\n");
}

#[test]
fn rejects_append_calls_with_the_wrong_target_type() {
    let source = r#"
package main

func main() {
    append("go", 1)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `append` has type `string`, expected slice"));
}

#[test]
fn rejects_append_calls_with_the_wrong_element_type() {
    let source = r#"
package main

func main() {
    append([]int{1}, "go")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 2 to `append` has type `string`, expected `int`"));
}

#[test]
fn compiles_and_runs_copy_on_local_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    dst := []int{1, 2, 3}
    src := []int{7, 8}
    copied := copy(dst, src)
    fmt.Println(copied, dst)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 [7 8 3]\n");
}

#[test]
fn compiles_and_runs_copy_on_package_slices() {
    let source = r#"
package main
import "fmt"

var dst = []int{1, 2, 3}

func main() {
    copied := copy(dst, []int{9, 10, 11, 12})
    fmt.Println(copied, dst)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 [9 10 11]\n");
}

#[test]
fn compiles_and_runs_delete_on_local_maps() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1, "wasm": 2}
    delete(values, "go")
    delete(values, "missing")
    fmt.Println(len(values), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 map[wasm:2]\n");
}

#[test]
fn compiles_and_runs_delete_on_nil_maps() {
    let source = r#"
package main
import "fmt"

func main() {
    var values map[string]int
    delete(values, "go")
    fmt.Println(values == nil, len(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 0\n");
}

#[test]
fn compiles_and_runs_delete_on_addressable_map_fields() {
    let source = r#"
package main
import "fmt"

type Box struct {
    values map[string]int
}

func main() {
    box := Box{values: map[string]int{"go": 1, "wasm": 2}}
    delete(box.values, "go")
    fmt.Println(len(box.values), box.values["go"], box.values["wasm"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 0 2\n");
}

#[test]
fn delete_cannot_be_used_in_value_position() {
    let source = r#"
package main

func main() {
    values := map[string]int{"go": 1}
    removed := delete(values, "go")
}
"#;

    let error = compile_source(source).expect_err("delete should not compile in value position");
    assert!(error
        .to_string()
        .contains("`delete` cannot be used in value position"));
}

#[test]
fn compiles_and_runs_make_map_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(map[string]int)
    values["go"] = 1
    fmt.Println(len(values), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 map[go:1]\n");
}

#[test]
fn compiles_and_runs_make_slice_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make([]int, 3)
    values[1] = 7
    fmt.Println(len(values), cap(values), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 3 [0 7 0]\n");
}

#[test]
fn compiles_and_runs_three_arg_make_slice_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make([]int, 2, 5)
    values = append(values, 7)
    fmt.Println(values == nil, len(values), cap(values), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false 3 5 [0 0 7]\n");
}

#[test]
fn compiles_and_runs_nil_slice_zero_values() {
    let source = r#"
package main
import "fmt"

func main() {
    var values []int
    fmt.Println(values == nil, len(values), cap(values), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 0 0 []\n");
}

#[test]
fn compiles_and_runs_typed_nil_slice_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    var values []int = nil
    values = append(values, 7)
    fmt.Println(values == nil, values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false [7]\n");
}

#[test]
fn make_slice_rejects_capacities_smaller_than_length() {
    let source = r#"
package main

func main() {
    _ = make([]int, 3, 2)
}
"#;

    let error = compile_source(source).expect_err("smaller capacities should fail at compile time");
    assert!(error.to_string().contains("must be >= length 3"));
}

#[test]
fn make_slice_rejects_negative_constant_length() {
    let source = r#"
package main

func main() {
    _ = make([]int, -1)
}
"#;

    let error = compile_source(source).expect_err("negative lengths should fail at compile time");
    assert!(error.to_string().contains("length -1 must not be negative"));
}

#[test]
fn make_slice_rejects_negative_constant_capacity() {
    let source = r#"
package main

func main() {
    _ = make([]int, 1, -2)
}
"#;

    let error =
        compile_source(source).expect_err("negative capacities should fail at compile time");
    assert!(error
        .to_string()
        .contains("capacity -2 must not be negative"));
}

#[test]
fn compiles_and_runs_append_spread() {
    let source = r#"
package main
import "fmt"

func main() {
    a := []int{1, 2}
    b := []int{3, 4, 5}
    a = append(a, b...)
    fmt.Println(a)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4 5]\n");
}

#[test]
fn compiles_and_runs_append_spread_empty_source() {
    let source = r#"
package main
import "fmt"

func main() {
    a := []int{1, 2}
    b := []int{}
    a = append(a, b...)
    fmt.Println(a)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2]\n");
}

#[test]
fn compiles_and_runs_append_spread_empty_target() {
    let source = r#"
package main
import "fmt"

func main() {
    var a []int
    b := []int{10, 20}
    a = append(a, b...)
    fmt.Println(a)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[10 20]\n");
}

#[test]
fn compiles_and_runs_append_spread_strings() {
    let source = r#"
package main
import "fmt"

func main() {
    a := []string{"hello"}
    b := []string{"world", "!"}
    a = append(a, b...)
    fmt.Println(a)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello world !]\n");
}

#[test]
fn compiles_and_runs_new_int() {
    let source = r#"
package main
import "fmt"

func main() {
    p := new(int)
    fmt.Println(*p)
    *p = 42
    fmt.Println(*p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n42\n");
}

#[test]
fn compiles_and_runs_new_bool() {
    let source = r#"
package main
import "fmt"

func main() {
    p := new(bool)
    fmt.Println(*p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn compiles_and_runs_new_string() {
    let source = r#"
package main
import "fmt"

func main() {
    p := new(string)
    fmt.Println(*p == "")
    *p = "hello"
    fmt.Println(*p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nhello\n");
}
