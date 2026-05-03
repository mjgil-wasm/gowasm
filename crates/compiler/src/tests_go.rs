use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_go_local_function_value_calls() {
    let source = r#"
package main
import "fmt"

func greet(name string) {
    fmt.Println(name)
}

func main() {
    run := greet
    go run("worker")
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn compiles_and_runs_go_package_function_value_calls() {
    let source = r#"
package main
import "fmt"

func greet(name string) {
    fmt.Println(name)
}

var run = greet

func main() {
    go run("worker")
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn compiles_and_runs_go_immediate_function_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    go func(name string) {
        fmt.Println(name)
    }("worker")
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn compiles_and_runs_go_returned_closure_values() {
    let source = r#"
package main
import "fmt"

func wrap(name string) func() {
    return func() {
        fmt.Println(name)
    }
}

func main() {
    go wrap("worker")()
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}

#[test]
fn compiles_and_runs_go_interface_method_calls() {
    let source = r#"
package main
import "fmt"

type Runner interface {
    Run()
}

type Greeter struct{}

func (Greeter) Run() {
    fmt.Println("worker")
}

func main() {
    var runner Runner = Greeter{}
    go runner.Run()
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "worker\nmain\n");
}
