use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_bound_method_values_through_assignment_and_closure_returns() {
    let source = r#"
package main
import "fmt"

type Greeter struct {
    Name string
}

func (g Greeter) Speak(prefix string) string {
    return prefix + g.Name
}

func build() func(string) string {
    greeter := Greeter{Name: "ada"}
    return greeter.Speak
}

func main() {
    run := build()
    fmt.Println(run("hi:"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hi:ada\n");
}

#[test]
fn compiles_and_runs_method_expressions_for_value_and_pointer_receivers() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    Value int
}

func (c Counter) Sum(extra int) int {
    return c.Value + extra
}

func (c *Counter) Inc(delta int) int {
    c.Value = c.Value + delta
    return c.Value
}

func main() {
    sum := Counter.Sum
    inc := (*Counter).Inc
    counter := Counter{Value: 2}
    fmt.Println(sum(counter, 3), inc(&counter, 4), counter.Value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 6 6\n");
}

#[test]
fn compiles_and_runs_interface_method_values() {
    let source = r#"
package main
import "fmt"

type Named interface {
    Name() string
}

type Person struct {
    Label string
}

func (p Person) Name() string {
    return p.Label
}

func main() {
    var named Named = Person{Label: "ada"}
    run := named.Name
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ada\n");
}

#[test]
fn compiles_and_runs_generic_receiver_method_values() {
    let source = r#"
package main
import "fmt"

type Box[T any] struct {
    Value T
}

func (b Box[T]) Speak(prefix string) string {
    return prefix + fmt.Sprint(b.Value)
}

func main() {
    box := Box[int]{Value: 7}
    bound := box.Speak
    fmt.Println(bound("v:"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "v:7\n");
}

#[test]
fn compiles_and_runs_method_values_in_goroutines_and_defers() {
    let source = r#"
package main
import "fmt"

type Logger struct {
    Ch chan string
}

func (l Logger) Send(label string) {
    l.Ch <- label
}

func (l Logger) Done() {
    fmt.Println("done")
}

func main() {
    logger := Logger{Ch: make(chan string, 1)}
    send := logger.Send
    done := logger.Done
    defer done()
    go send("go")
    fmt.Println(<-logger.Ch)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go\ndone\n");
}

#[test]
fn compiles_and_runs_method_values_in_stdlib_callbacks() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

type Mapper struct {}

func (Mapper) Upper(r int) int {
    return unicode.ToUpper(r)
}

func main() {
    mapper := Mapper{}
    fmt.Println(strings.Map(mapper.Upper, "go!"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "GO!\n");
}
