use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn go_fmt_println_in_closure() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan bool)
    go func() {
        fmt.Println("from goroutine")
        ch <- true
    }()
    <-ch
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "from goroutine\n");
}

#[test]
fn go_stdlib_println_direct() {
    let source = r#"
package main
import "fmt"

func main() {
    go fmt.Println("hello from go")
    fmt.Println("from main")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello from go\nfrom main\n");
}

#[test]
fn go_stdlib_print_direct() {
    let source = r#"
package main
import "fmt"

func main() {
    go fmt.Print("no newline")
    fmt.Println()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "no newline\n");
}

#[test]
fn go_stdlib_multiple_args() {
    let source = r#"
package main
import "fmt"

func main() {
    go fmt.Println("a", "b", "c")
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a b c\ndone\n");
}

#[test]
fn go_stdlib_with_channel_sync() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int)
    go func() {
        go fmt.Println("nested go stdlib")
        ch <- 1
    }()
    <-ch
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "nested go stdlib\n");
}
