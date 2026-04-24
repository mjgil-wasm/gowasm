#![cfg(test)]

use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn promoted_methods_satisfy_embedded_interface_targets() {
    let source = r#"
package main
import "fmt"

type Reader interface {
    Read() string
}

type NamedReader interface {
    Reader
    Name() string
}

type inner struct {
    label string
}

func (value inner) Read() string {
    return "read:" + value.label
}

type outer struct {
    inner
}

func (outer) Name() string {
    return "outer"
}

func main() {
    var value NamedReader = outer{inner: inner{label: "ada"}}
    fmt.Println(value.Read(), value.Name())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "read:ada outer\n");
}

#[test]
fn generic_interface_assertions_preserve_satisfaction_rules() {
    let source = r#"
package main
import "fmt"

type Reader[T any] interface {
    Value() T
}

type Box struct {
    value int
}

func (box Box) Value() int {
    return box.value
}

func main() {
    var reader Reader[int] = Box{value: 9}
    var any interface{} = Box{value: 9}
    asserted, ok := any.(Reader[int])
    fmt.Println(reader.Value(), ok, asserted.Value())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 true 9\n");
}

#[test]
fn interface_assertions_require_exact_method_signatures() {
    let source = r#"
package main
import "fmt"

type Any interface {}

type Needs interface {
    Read(value int) int
}

type WrongParam struct{}

func (WrongParam) Read(text string) int {
    return 1
}

type WrongResult struct{}

func (WrongResult) Read(value int) string {
    return "bad"
}

func main() {
    var value Any = WrongParam{}
    _, wrongParam := value.(Needs)
    value = WrongResult{}
    _, wrongResult := value.(Needs)
    fmt.Println(wrongParam, wrongResult)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false false\n");
}

#[test]
fn interface_satisfaction_errors_report_mismatched_signatures() {
    let source = r#"
package main

type Needs interface {
    Read(value int) int
}

type Wrong struct{}

func (Wrong) Read(text string) int {
    return 1
}

func main() {
    var value Needs = Wrong{}
    _ = value
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    let detail = error.to_string();
    assert!(detail.contains("does not satisfy interface `Needs`"));
    assert!(detail.contains("method `Read` has signature `Read(string) int`, want `Read(int) int`"));
}
