use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_default_only_select() {
    let source = r#"
package main
import "fmt"

func main() {
    select {
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
fn compiles_and_runs_blocking_receive_select_case() {
    let source = r#"
package main
import "fmt"

func send(values chan int) {
    values <- 7
}

func main() {
    values := make(chan int)
    go send(values)
    select {
    case result := <-values:
        fmt.Println(result)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_blocking_closed_receive_select_case() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 false\n");
}

#[test]
fn close_wakes_blocked_receive_select_case() {
    let source = r#"
package main
import "fmt"

func closer(values chan int) {
    close(values)
}

func main() {
    values := make(chan int)
    go closer(values)
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake blocked receive select");
    assert_eq!(vm.stdout(), "0 false\n");
}

#[test]
fn short_decl_in_select_case_can_rebind_name_in_case_scope() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 1)
    values <- 7
    select {
    case value, ok := <-values:
        x := 0
        x, y := value, 99
        fmt.Println(x, y, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 99 true\n");
}

#[test]
fn compiles_and_runs_blocking_send_select_case() {
    let source = r#"
package main
import "fmt"

func recv(values chan int) {
    fmt.Println(<-values)
}

func main() {
    values := make(chan int)
    go recv(values)
    select {
    case values <- 7:
        fmt.Println("sent")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\nsent\n");
}

#[test]
fn blocking_select_rotates_ready_case_priority() {
    let source = r#"
package main
import "fmt"

func main() {
    left := make(chan int, 1)
    right := make(chan int, 1)
    left <- 1
    right <- 2
    select {
    case value := <-left:
        fmt.Println(value)
    case value := <-right:
        fmt.Println(value)
    }
    left <- 3
    select {
    case value := <-left:
        fmt.Println(value)
    case value := <-right:
        fmt.Println(value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n");
}

#[test]
fn blocking_select_rotates_buffered_closed_receive_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    left := make(chan int, 1)
    right := make(chan int, 1)
    left <- 1
    right <- 2
    close(left)
    close(right)
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "left 1 true\nright 2 true\nleft 0 false\nright 0 false\n"
    );
}

#[test]
fn compiles_and_runs_receive_select_with_default_fallback() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    select {
    case result := <-values:
        fmt.Println(result)
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
fn receive_select_with_default_rotates_ready_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    left := make(chan int, 1)
    right := make(chan int, 1)
    left <- 1
    right <- 2
    select {
    case value := <-left:
        fmt.Println(value)
    case value := <-right:
        fmt.Println(value)
    default:
        fmt.Println("default")
    }
    left <- 3
    select {
    case value := <-left:
        fmt.Println(value)
    case value := <-right:
        fmt.Println(value)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n");
}

#[test]
fn receive_select_with_default_rotates_buffered_closed_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    left := make(chan int, 1)
    right := make(chan int, 1)
    left <- 1
    right <- 2
    close(left)
    close(right)
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    default:
        fmt.Println("default")
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    default:
        fmt.Println("default")
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    default:
        fmt.Println("default")
    }
    select {
    case value, ok := <-left:
        fmt.Println("left", value, ok)
    case value, ok := <-right:
        fmt.Println("right", value, ok)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "left 1 true\nright 2 true\nleft 0 false\nright 0 false\n"
    );
}

#[test]
fn compiles_and_runs_ready_receive_select_case() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 1)
    values <- 7
    select {
    case result := <-values:
        fmt.Println(result)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_comma_ok_receive_select_case() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 false\n");
}

#[test]
fn compiles_and_runs_buffered_closed_receive_select_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    }
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    }
    select {
    case result, ok := <-values:
        fmt.Println(result, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 true\n8 true\n0 false\n");
}

#[test]
fn continue_in_select_case_advances_enclosing_loop() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 1
    values <- 2
    close(values)
    done := false
    for !done {
        select {
        case value, ok := <-values:
            if !ok {
                done = true
                continue
            }
            if value == 2 {
                continue
            }
            fmt.Println(value)
        }
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\ndone\n");
}

#[test]
fn break_in_select_case_only_exits_select() {
    let source = r#"
package main
import "fmt"

func main() {
    for index := 0; index < 2; index++ {
        select {
        default:
            fmt.Println("case", index)
            break
        }
        fmt.Println("loop", index)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "case 0\nloop 0\ncase 1\nloop 1\n");
}

#[test]
fn compiles_and_runs_send_select_with_default_fallback() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    select {
    case values <- 7:
        fmt.Println("sent")
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
fn send_select_with_default_rotates_ready_cases() {
    let source = r#"
package main
import "fmt"

func main() {
    left := make(chan int, 1)
    right := make(chan int, 1)
    select {
    case left <- 1:
        fmt.Println("left")
    case right <- 2:
        fmt.Println("right")
    default:
        fmt.Println("default")
    }
    fmt.Println(<-left)
    select {
    case left <- 3:
        fmt.Println("left")
    case right <- 4:
        fmt.Println("right")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "left\n1\nright\n");
}

#[test]
fn compiles_and_runs_ready_send_select_case() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 1)
    select {
    case values <- 7:
        fmt.Println("sent")
    default:
        fmt.Println("default")
    }
    fmt.Println(<-values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "sent\n7\n");
}

#[test]
fn send_select_on_closed_channel_fails() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    close(values)
    select {
    case values <- 7:
    default:
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("closed send select should fail");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn blocking_send_select_on_closed_channel_fails() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    close(values)
    select {
    case values <- 7:
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("closed blocking send select should fail");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn close_wakes_blocked_send_select_with_error() {
    let source = r#"
package main

func sender(values chan int) {
    select {
    case values <- 7:
    }
}

func main() {
    values := make(chan int)
    go sender(values)
    close(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail blocked send select");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn rejects_send_select_on_receive_only_channels() {
    let source = r#"
package main

func main() {
    var values <-chan int
    select {
    case values <- 7:
    default:
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("cannot send on receive-only channel type `<-chan int`"));
}

#[test]
fn rejects_receive_select_on_send_only_channels() {
    let source = r#"
package main

func main() {
    var values chan<- int
    select {
    case <-values:
    default:
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("cannot receive from send-only channel type `chan<- int`"));
}
