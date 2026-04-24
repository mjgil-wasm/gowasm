use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn select_ignores_nil_receive_channel() {
    let source = r#"
package main
import "fmt"

func main() {
    var nilCh chan int
    ch := make(chan int, 1)
    ch <- 42
    select {
    case v := <-nilCh:
        fmt.Println("nil", v)
    case v := <-ch:
        fmt.Println("live", v)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "live 42\n");
}

#[test]
fn select_ignores_nil_send_channel() {
    let source = r#"
package main
import "fmt"

func main() {
    var nilCh chan int
    ch := make(chan int, 1)
    ch <- 10
    select {
    case nilCh <- 99:
        fmt.Println("nil send")
    case v := <-ch:
        fmt.Println("live recv", v)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "live recv 10\n");
}

#[test]
fn select_all_nil_takes_default() {
    let source = r#"
package main
import "fmt"

func main() {
    var ch1 chan int
    var ch2 chan string
    select {
    case <-ch1:
        fmt.Println("ch1")
    case ch2 <- "hello":
        fmt.Println("ch2")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "default\n");
}

#[test]
fn select_all_nil_without_default_deadlocks() {
    let source = r#"
package main

func main() {
    var ch1 chan int
    var ch2 chan string
    select {
    case <-ch1:
    case ch2 <- "hello":
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("all-nil select without default should deadlock");
    assert!(error.to_string().contains("all goroutines are blocked"));
}

#[test]
fn select_nil_recv_with_live_buffered_send() {
    let source = r#"
package main
import "fmt"

func main() {
    var nilCh chan int
    ch := make(chan int, 1)
    select {
    case <-nilCh:
        fmt.Println("nil recv")
    case ch <- 55:
        fmt.Println("sent", 55)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "sent 55\n");
}

#[test]
fn channel_range_drains_buffered_values() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int, 3)
    ch <- 1
    ch <- 2
    ch <- 3
    close(ch)
    for v := range ch {
        fmt.Println(v)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n3\n");
}

#[test]
fn select_recv_ok_on_closed_channel() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int, 1)
    ch <- 42
    close(ch)
    select {
    case v, ok := <-ch:
        fmt.Println(v, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true\n");
}

#[test]
fn select_recv_ok_from_closed_empty_channel() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int)
    close(ch)
    select {
    case v, ok := <-ch:
        fmt.Println(v, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 false\n");
}
