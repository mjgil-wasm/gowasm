use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_typed_var_zero_values() {
    let source = r#"
package main
import "fmt"

func main() {
    var name string
    var count int
    var ready bool
    fmt.Println(name, count, ready)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), " 0 false\n");
}

#[test]
fn compiles_and_runs_var_declarations_with_initializers() {
    let source = r#"
package main
import "fmt"

func main() {
    var label = "hello"
    var count int = 3
    fmt.Println(label, count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello 3\n");
}
