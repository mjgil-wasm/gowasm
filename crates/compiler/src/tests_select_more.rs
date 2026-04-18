use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn blocking_select_compiles_and_runs_dense_ready_sets() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    recvC := make(chan int, 1)
    sendD := make(chan int, 1)
    closed := make(chan int, 1)

    recvA <- 11
    recvC <- 33
    closed <- 55
    close(closed)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }

    fmt.Println(<-sendB)
    fmt.Println(<-sendD)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "recvA 11 true\nsendB\nrecvC 33 true\nsendD\nclosed 55 true\n22\n44\n"
    );
}

#[test]
fn default_select_compiles_and_runs_dense_ready_sets() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    recvC := make(chan int, 1)
    sendD := make(chan int, 1)
    closed := make(chan int, 1)

    recvA <- 11
    recvC <- 33
    closed <- 55
    close(closed)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-recvC:
        fmt.Println("recvC", value, ok)
    case sendD <- 44:
        fmt.Println("sendD")
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }

    fmt.Println(<-sendB)
    fmt.Println(<-sendD)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "recvA 11 true\nsendB\nrecvC 33 true\nsendD\nclosed 55 true\n22\n44\n"
    );
}
