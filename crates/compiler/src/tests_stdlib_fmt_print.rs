use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn print_single_value() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print("hello")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello");
}

#[test]
fn print_multiple_strings_no_spaces() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print("hello", "world")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "helloworld");
}

#[test]
fn print_ints_with_spaces() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print(1, 2, 3)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2 3");
}

#[test]
fn print_mixed_string_int() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print("x=", 42)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "x=42");
}

#[test]
fn print_no_trailing_newline() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print("a")
    fmt.Print("b")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ab");
}

#[test]
fn print_then_println() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Print("hello ")
    fmt.Println("world")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}
