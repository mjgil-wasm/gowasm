use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strings_to_title_basic() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.ToTitle("hello world"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "HELLO WORLD\n");
}

#[test]
fn strings_to_title_mixed_case() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.ToTitle("hElLo WoRlD"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "HELLO WORLD\n");
}

#[test]
fn strings_to_title_empty() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.ToTitle(""))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\n");
}
