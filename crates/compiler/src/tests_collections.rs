use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_slice_literals_and_indexing() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3}
    fmt.Println(values, values[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3] 2\n");
}

#[test]
fn compiles_and_runs_array_literals_and_indexing() {
    let source = r#"
package main
import "fmt"

func main() {
    values := [3]int{4, 5, 6}
    fmt.Println(values[2], values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6 [4 5 6]\n");
}

#[test]
fn compiles_typed_slice_var_initializers() {
    let source = r#"
package main
import "fmt"

func main() {
    var values []int = []int{7, 8}
    fmt.Println(values[0])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_slice_index_assignment() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3}
    values[1] = 9
    fmt.Println(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 9 3]\n");
}

#[test]
fn compiles_and_runs_global_array_index_assignment() {
    let source = r#"
package main
import "fmt"

var values [3]int = [3]int{4, 5, 6}

func main() {
    values[2] = 8
    fmt.Println(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[4 5 8]\n");
}

#[test]
fn compiles_and_runs_string_indexing() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println("go"[0], "go"[1], "hey"[2])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "103 111 121\n");
}

#[test]
fn compiles_and_runs_utf8_string_indexing_as_bytes() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println("é"[0], "é"[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "195 169\n");
}
