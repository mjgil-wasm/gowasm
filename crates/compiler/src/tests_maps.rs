use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_map_literals_and_indexing() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1, "wasm": 2}
    fmt.Println(values["wasm"], values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 map[go:1 wasm:2]\n");
}

#[test]
fn compiles_and_runs_zero_value_map_reads() {
    let source = r#"
package main
import "fmt"

func main() {
    var values map[string]int
    fmt.Println(values, values["missing"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "map[] 0\n");
}

#[test]
fn compiles_and_runs_map_index_assignment() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1}
    values["go"] = 4
    values["wasm"] = 2
    fmt.Println(values["go"], values["wasm"], values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 2 map[go:4 wasm:2]\n");
}

#[test]
fn nil_map_index_assignment_fails_at_runtime() {
    let source = r#"
package main

func main() {
    var values map[string]int
    values["go"] = 1
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil map writes should fail");
    assert!(error.to_string().contains("cannot assign into a nil map"));
}

#[test]
fn compiles_and_runs_comma_ok_short_var_map_lookups() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1}
    found, ok := values["go"]
    missing, missingOk := values["wasm"]
    fmt.Println(found, ok, missing, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 true 0 false\n");
}

#[test]
fn compiles_and_runs_comma_ok_short_var_reads_on_nil_maps() {
    let source = r#"
package main
import "fmt"

func main() {
    var values map[string]int
    value, ok := values["go"]
    fmt.Println(value, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 false\n");
}

#[test]
fn compiles_and_runs_comma_ok_assignment_map_lookups() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1}
    var found int
    var ok bool
    var missing int
    var missingOk bool
    found, ok = values["go"]
    missing, missingOk = values["wasm"]
    fmt.Println(found, ok, missing, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 true 0 false\n");
}

#[test]
fn compiles_and_runs_typed_nil_map_initializers() {
    let source = r#"
package main
import "fmt"

func main() {
    var values map[string]int = nil
    fmt.Println(values == nil, len(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 0\n");
}

#[test]
fn compiles_and_runs_typed_nil_slice_initializers() {
    let source = r#"
package main
import "fmt"

func main() {
    var values []int = nil
    fmt.Println(values == nil, len(values), cap(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 0 0\n");
}

#[test]
fn compiles_and_runs_map_with_int_keys() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[int]string{1: "one", 2: "two"}
    fmt.Println(m[1])
    fmt.Println(m[2])
    m[3] = "three"
    fmt.Println(m[3])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "one\ntwo\nthree\n");
}

#[test]
fn compiles_and_runs_map_with_struct_keys() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    m := map[Point]string{
        Point{1, 2}: "a",
        Point{3, 4}: "b",
    }
    fmt.Println(m[Point{1, 2}])
    fmt.Println(m[Point{3, 4}])
    m[Point{5, 6}] = "c"
    fmt.Println(m[Point{5, 6}])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a\nb\nc\n");
}

#[test]
fn compiles_and_runs_map_struct_key_comma_ok() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    m := map[Point]string{Point{1, 2}: "found"}
    v, ok := m[Point{1, 2}]
    fmt.Println(v, ok)
    v2, ok2 := m[Point{9, 9}]
    fmt.Println(v2, ok2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "found true\n false\n");
}

#[test]
fn compiles_and_runs_shared_map_mutations_across_aliases_and_function_calls() {
    let source = r#"
package main
import "fmt"

func mutate(values map[string]int) {
    values["go"] = values["go"] + 1
    delete(values, "wasm")
}

func main() {
    values := map[string]int{"go": 1, "wasm": 2}
    alias := values
    mutate(alias)
    fmt.Println(values["go"], values["wasm"], len(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 0 1\n");
}
