use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_nil_channel_zero_values() {
    let source = r#"
package main
import "fmt"

func main() {
    var values chan int
    fmt.Println(values == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn compiles_and_runs_receive_only_channel_params() {
    let source = r#"
package main
import "fmt"

func recv(values <-chan int) {
    fmt.Println(<-values)
}

func main() {
    values := make(chan int, 1)
    values <- 7
    recv(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_send_only_channel_params() {
    let source = r#"
package main
import "fmt"

func send(values chan<- int) {
    values <- 7
}

func main() {
    values := make(chan int)
    go send(values)
    fmt.Println(<-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn rejects_send_on_receive_only_channels() {
    let source = r#"
package main

func send(values <-chan int) {
    values <- 7
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("cannot send on receive-only channel type `<-chan int`"));
}

#[test]
fn rejects_receive_from_send_only_channels() {
    let source = r#"
package main

func recv(values chan<- int) {
    _ = <-values
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("cannot receive from send-only channel type `chan<- int`"));
}

#[test]
fn rejects_close_of_receive_only_channels() {
    let source = r#"
package main

func closeRecv(values <-chan int) {
    close(values)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("expected bidirectional or send-only channel"));
}

#[test]
fn compiles_and_runs_bidirectional_to_directional_assignments() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 1)
    var recv <-chan int
    var send chan<- int
    recv = values
    send = values
    send <- 7
    fmt.Println(<-recv)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn rejects_assigning_send_only_channels_to_receive_only_targets() {
    let source = r#"
package main

func main() {
    var recv <-chan int
    var send chan<- int
    recv = send
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("channel value of type `chan<- int` is not assignable to `<-chan int`"));
}

#[test]
fn rejects_returning_send_only_channels_as_receive_only() {
    let source = r#"
package main

func wrong(values chan<- int) <-chan int {
    return values
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("channel value of type `chan<- int` is not assignable to `<-chan int`"));
}

#[test]
fn compiles_and_runs_make_channel_values() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    fmt.Println(values == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn compiles_and_runs_unbuffered_channel_handoff() {
    let source = r#"
package main
import "fmt"

func send(values chan int) {
    values <- 7
    fmt.Println("sent")
}
func main() {
    values := make(chan int)
    go send(values)
    fmt.Println(<-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\nsent\n");
}

#[test]
fn nil_channel_receive_deadlocks() {
    let source = r#"
package main

func main() {
    var values chan int
    result := <-values
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil channel should deadlock");
    assert!(error.to_string().contains("all goroutines are blocked"));
}

#[test]
fn compiles_and_runs_buffered_channel_round_trips() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    fmt.Println(<-values, <-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 8\n");
}

#[test]
fn compiles_and_runs_comma_ok_receive_forms() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 1)
    values <- 7
    first, ok := <-values
    fmt.Println(first, ok)

    values <- 9
    var second int
    var present bool
    second, present = <-values
    fmt.Println(second, present)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 true\n9 true\n");
}

#[test]
fn close_of_nil_channel_fails() {
    let source = r#"
package main

func main() {
    var values chan int
    close(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close(nil) should fail");
    assert!(error.to_string().contains("close of nil channel"));
}

#[test]
fn send_on_closed_channel_fails() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    close(values)
    values <- 7
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("send on closed channel should fail");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn close_of_closed_channel_fails() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    close(values)
    close(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close(closed) should fail");
    assert!(error.to_string().contains("close of closed channel"));
}

#[test]
fn receive_from_closed_channel_returns_zero_value() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    fmt.Println(<-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("closed receive should return zero");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn buffered_closed_channel_drains_values_before_zero_false() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    fmt.Println(<-values)
    fmt.Println(<-values)
    value, ok := <-values
    fmt.Println(value, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("buffered closed receives should drain values first");
    assert_eq!(vm.stdout(), "7\n8\n0 false\n");
}

#[test]
fn buffered_closed_channel_range_drains_buffer_before_stopping() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    for value := range values {
        fmt.Println(value)
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("buffered closed channel range should drain buffer");
    assert_eq!(vm.stdout(), "7\n8\ndone\n");
}

#[test]
fn close_wakes_blocked_receiver_with_zero_value() {
    let source = r#"
package main
import "fmt"

func closer(values chan int) {
    close(values)
}

func main() {
    values := make(chan int)
    go closer(values)
    fmt.Println(<-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake blocked receivers");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn close_wakes_blocked_sender_with_error() {
    let source = r#"
package main

func send(values chan int, value int) {
    values <- value
}

func main() {
    values := make(chan int)
    go send(values, 7)
    close(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail blocked senders");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn comma_ok_receive_on_closed_channel_reports_false() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    value, ok := <-values
    fmt.Println(value, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("comma-ok receive on closed channel should run");
    assert_eq!(vm.stdout(), "0 false\n");
}

