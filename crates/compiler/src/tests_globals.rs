use super::{compile_source, compile_workspace, SourceInput};
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_package_var_initialization() {
    let source = r#"
package main
import "fmt"

var count int = 4
var label = "go"

func main() {
    fmt.Println(label, count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go 4\n");
}

#[test]
fn compiles_and_runs_package_var_assignment() {
    let source = r#"
package main
import "fmt"

var count int

func main() {
    count = 9
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_package_struct_field_assignment() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

var point Point = Point{x: 1, y: 2}

func main() {
    point.x = 7
    fmt.Println(point)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{7 2}\n");
}

#[test]
fn compiles_and_runs_init_before_main() {
    let source = r#"
package main
import "fmt"

func init() {
    fmt.Println("init")
}

func main() {
    fmt.Println("main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "init\nmain\n");
}

#[test]
fn compiles_and_runs_multiple_init_functions_in_lexical_file_order() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "main.go",
                source: r#"
package main
import "fmt"

func init() {
    fmt.Println("first")
}

func main() {
    fmt.Println("done")
}
"#,
            },
            SourceInput {
                path: "helper.go",
                source: r#"
package main
import "fmt"

func init() {
    fmt.Println("second")
}

func init() {
    fmt.Println("third")
}
"#,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "second\nthird\nfirst\ndone\n");
}
