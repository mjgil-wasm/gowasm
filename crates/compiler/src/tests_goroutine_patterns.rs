use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn goroutine_sends_value_to_main_via_channel() {
    let source = r#"
package main
import "fmt"

func producer(ch chan int) {
    ch <- 42
}

func main() {
    ch := make(chan int)
    go producer(ch)
    v := <-ch
    fmt.Println(v)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn main_sends_value_to_goroutine_via_channel() {
    let source = r#"
package main
import "fmt"

func consumer(ch chan int) {
    v := <-ch
    fmt.Println("got", v)
}

func main() {
    ch := make(chan int)
    go consumer(ch)
    ch <- 99
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "got 99\n");
}

#[test]
fn two_goroutines_send_to_same_channel() {
    let source = r#"
package main
import "fmt"

func sender(ch chan int, val int) {
    ch <- val
}

func main() {
    ch := make(chan int, 2)
    go sender(ch, 10)
    go sender(ch, 20)
    a := <-ch
    b := <-ch
    sum := a + b
    fmt.Println(sum)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "30\n");
}

#[test]
fn goroutine_pipeline_through_two_stages() {
    let source = r#"
package main
import "fmt"

func double(in chan int, out chan int) {
    v := <-in
    out <- v * 2
}

func addTen(in chan int, out chan int) {
    v := <-in
    out <- v + 10
}

func main() {
    ch1 := make(chan int)
    ch2 := make(chan int)
    ch3 := make(chan int)
    go double(ch1, ch2)
    go addTen(ch2, ch3)
    ch1 <- 5
    result := <-ch3
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "20\n");
}

#[test]
fn goroutine_with_closure_captures_local() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := make(chan int)
    x := 7
    go func() {
        ch <- x * 3
    }()
    fmt.Println(<-ch)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "21\n");
}

#[test]
fn goroutine_closes_channel_after_sending() {
    let source = r#"
package main
import "fmt"

func produce(ch chan int) {
    ch <- 1
    ch <- 2
    ch <- 3
    close(ch)
}

func main() {
    ch := make(chan int)
    go produce(ch)
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
fn select_receives_from_first_ready_goroutine() {
    let source = r#"
package main
import "fmt"

func sendTo(ch chan string, msg string) {
    ch <- msg
}

func main() {
    ch1 := make(chan string, 1)
    ch2 := make(chan string, 1)
    go sendTo(ch1, "hello")
    go sendTo(ch2, "world")
    count := 0
    for count < 2 {
        select {
        case v := <-ch1:
            fmt.Println("ch1:", v)
            count = count + 1
        case v := <-ch2:
            fmt.Println("ch2:", v)
            count = count + 1
        }
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    let out = vm.stdout();
    assert!(out.contains("ch1: hello"), "should receive from ch1");
    assert!(out.contains("ch2: world"), "should receive from ch2");
}

#[test]
fn goroutine_with_multiple_sends_and_buffered_channel() {
    let source = r#"
package main
import "fmt"

func fillBuffer(ch chan int) {
    ch <- 10
    ch <- 20
    ch <- 30
}

func main() {
    ch := make(chan int, 3)
    go fillBuffer(ch)
    a := <-ch
    b := <-ch
    c := <-ch
    fmt.Println(a, b, c)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10 20 30\n");
}

#[test]
fn done_channel_pattern() {
    let source = r#"
package main
import "fmt"

func worker(done chan bool) {
    fmt.Println("working")
    done <- true
}

func main() {
    done := make(chan bool)
    go worker(done)
    <-done
    fmt.Println("finished")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "working\nfinished\n");
}
