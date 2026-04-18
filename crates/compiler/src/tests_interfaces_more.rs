use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_interface_embedding() {
    let source = r#"
package main
import "fmt"

type Greeter interface {
    Greet() string
}

type Farewell interface {
    Bye() string
}

type Social interface {
    Greeter
    Farewell
}

type Person struct {
    Name string
}

func (p Person) Greet() string {
    return "hello from " + p.Name
}

func (p Person) Bye() string {
    return "goodbye from " + p.Name
}

func useSocial(s Social) {
    fmt.Println(s.Greet())
    fmt.Println(s.Bye())
}

func main() {
    p := Person{Name: "Alice"}
    useSocial(p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello from Alice\ngoodbye from Alice\n");
}

#[test]
fn compiles_and_runs_embedded_interface_type_check() {
    let source = r#"
package main
import "fmt"

type Stringer interface {
    String() string
}

type Printer interface {
    Stringer
    Print()
}

type Doc struct {
    Text string
}

func (d Doc) String() string {
    return d.Text
}

func (d Doc) Print() {
    fmt.Println(d.String())
}

func useStringer(s Stringer) {
    fmt.Println(s.String())
}

func usePrinter(p Printer) {
    p.Print()
}

func main() {
    d := Doc{Text: "document"}
    useStringer(d)
    usePrinter(d)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "document\ndocument\n");
}

#[test]
fn compiles_and_runs_inline_empty_interface_parameter() {
    let source = r#"
package main
import "fmt"

func show(v interface{}) {
    fmt.Println(v)
}

func main() {
    show(42)
    show("hello")
    show(true)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\nhello\ntrue\n");
}

#[test]
fn compiles_and_runs_inline_empty_interface_variable() {
    let source = r#"
package main
import "fmt"

func main() {
    var x interface{}
    fmt.Println(x)
    x = 99
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil>\n99\n");
}

#[test]
fn compiles_and_runs_any_type_alias() {
    let source = r#"
package main
import "fmt"

func show(v any) {
    fmt.Println(v)
}

func main() {
    var x any
    fmt.Println(x)
    x = "hello"
    show(x)
    show(42)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil>\nhello\n42\n");
}

#[test]
fn compiles_and_runs_interface_read_calls_that_mutate_buffers() {
    let source = r#"
package main
import "errors"
import "fmt"

type Reader interface {
    Read([]byte) (int, error)
}

type customReader struct {
    data string
    offset int
}

func (r *customReader) Read(p []byte) (int, error) {
    if r.offset >= len(r.data) {
        return 0, errors.New("EOF")
    }

    remaining := []byte(r.data[r.offset:])
    if len(remaining) > len(p) {
        remaining = remaining[:len(p)]
    }

    n := copy(p, remaining)
    r.offset += n
    if r.offset >= len(r.data) {
        return n, errors.New("EOF")
    }
    return n, nil
}

func consume(reader Reader) {
    head := []byte("....")
    n, err := reader.Read(head)
    fmt.Println(n, err == nil, string(head[:n]), string(head))

    tail := []byte("...")
    n, err = reader.Read(tail)
    fmt.Println(n, err != nil, string(tail[:n]), string(tail))
}

func main() {
    reader := &customReader{data: "chunked"}
    consume(reader)
    fmt.Println(reader.offset)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 true chun chun\n3 true ked ked\n7\n");
}
