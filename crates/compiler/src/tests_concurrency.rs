use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_go_statements_for_named_functions() {
    let source = r#"
package main
import "fmt"

func worker() {
    fmt.Println("worker")
}

func main() {
    go worker()
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn compiles_go_statements_for_concrete_methods() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (c Counter) show(prefix string) {
    fmt.Println(prefix, c.value)
}

func main() {
    counter := Counter{value: 7}
    go counter.show("count")
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "done\ncount 7\n");
}
