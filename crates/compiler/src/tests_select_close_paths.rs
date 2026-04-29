use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn blocking_select_reaches_multi_case_closed_send_error() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    closed := make(chan int, 1)

    recvA <- 11
    close(closed)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case closed <- 33:
        fmt.Println("closed")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case closed <- 33:
        fmt.Println("closed")
    }

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case closed <- 33:
        fmt.Println("closed")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("multi-case blocking select should fail on the later closed send");
    assert_eq!(vm.stdout(), "recvA 11 true\nsendB\n");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn default_select_reaches_multi_case_closed_send_error() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    closed := make(chan int, 1)

    recvA <- 11
    close(closed)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-recvA:
        fmt.Println("recvA", value, ok)
    case sendB <- 22:
        fmt.Println("sendB")
    case closed <- 33:
        fmt.Println("closed")
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
    case closed <- 33:
        fmt.Println("closed")
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
    case closed <- 33:
        fmt.Println("closed")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("multi-case default select should fail on the later closed send");
    assert_eq!(vm.stdout(), "recvA 11 true\nsendB\n");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn blocking_select_reaches_broader_multi_case_closed_send_error() {
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("broader blocking select should fail on the later closed send");
    assert_eq!(vm.stdout(), "recvA 11 true\nsendB\nrecvC 33 true\nsendD\n");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn default_select_reaches_broader_multi_case_closed_send_error() {
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
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
    case closed <- 55:
        fmt.Println("closed")
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("broader default select should fail on the later closed send");
    assert_eq!(vm.stdout(), "recvA 11 true\nsendB\nrecvC 33 true\nsendD\n");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn blocking_select_reaches_broader_multi_case_closed_receive() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    recvC := make(chan int, 1)
    closed := make(chan int)

    recvA <- 11
    recvC <- 33
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
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("broader blocking select should rotate onto the closed receive");
    assert_eq!(
        vm.stdout(),
        "recvA 11 true\nsendB\nrecvC 33 true\nclosed 0 false\n"
    );
}

#[test]
fn default_select_reaches_broader_multi_case_closed_receive() {
    let source = r#"
package main
import "fmt"

func main() {
    var never chan int
    recvA := make(chan int, 1)
    sendB := make(chan int, 1)
    recvC := make(chan int, 1)
    closed := make(chan int)

    recvA <- 11
    recvC <- 33
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
    case value, ok := <-closed:
        fmt.Println("closed", value, ok)
    default:
        fmt.Println("default")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("broader default select should rotate onto the closed receive");
    assert_eq!(
        vm.stdout(),
        "recvA 11 true\nsendB\nrecvC 33 true\nclosed 0 false\n"
    );
}

#[test]
fn close_wakes_broader_multi_case_receive_select_case() {
    let source = r#"
package main
import "fmt"

func closer(values chan int) {
    close(values)
}

func main() {
    var never chan int
    stalledRecvA := make(chan int)
    stalledSendB := make(chan int)
    stalledRecvC := make(chan int)
    values := make(chan int)

    go closer(values)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-stalledRecvA:
        fmt.Println("recvA", value, ok)
    case stalledSendB <- 22:
        fmt.Println("sendB")
    case value, ok := <-stalledRecvC:
        fmt.Println("recvC", value, ok)
    case value, ok := <-values:
        fmt.Println("closed", value, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake the broader blocked receive select");
    assert_eq!(vm.stdout(), "closed 0 false\n");
}

#[test]
fn close_wakes_broader_multi_case_send_select_with_error() {
    let source = r#"
package main

func closer(values chan int) {
    close(values)
}

func main() {
    var never chan int
    stalledRecvA := make(chan int)
    stalledSendB := make(chan int)
    stalledRecvC := make(chan int)
    values := make(chan int)

    go closer(values)

    select {
    case <-never:
    case <-stalledRecvA:
    case stalledSendB <- 22:
    case <-stalledRecvC:
    case values <- 7:
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail the broader blocked send select");
    assert!(error.to_string().contains("send on closed channel"));
}

#[test]
fn close_wakes_multi_case_receive_select_case() {
    let source = r#"
package main
import "fmt"

func closer(values chan int) {
    close(values)
}

func main() {
    var never chan int
    stalledRecv := make(chan int)
    stalledSend := make(chan int)
    values := make(chan int)

    go closer(values)

    select {
    case value, ok := <-never:
        fmt.Println("nil", value, ok)
    case value, ok := <-stalledRecv:
        fmt.Println("stalled", value, ok)
    case stalledSend <- 22:
        fmt.Println("send")
    case value, ok := <-values:
        fmt.Println("closed", value, ok)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("close should wake the larger blocked receive select");
    assert_eq!(vm.stdout(), "closed 0 false\n");
}

#[test]
fn close_wakes_multi_case_send_select_with_error() {
    let source = r#"
package main

func closer(values chan int) {
    close(values)
}

func main() {
    var never chan int
    stalledRecv := make(chan int)
    stalledSend := make(chan int)
    values := make(chan int)

    go closer(values)

    select {
    case <-never:
    case <-stalledRecv:
    case stalledSend <- 22:
    case values <- 7:
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("close should fail the larger blocked send select");
    assert!(error.to_string().contains("send on closed channel"));
}
