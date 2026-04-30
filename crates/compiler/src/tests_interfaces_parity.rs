use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_interface_to_interface_assignment_return_and_comparison() {
    let source = r#"
package main
import "fmt"

type Reader interface { Read() string }
type ReadCloser interface {
    Reader
    Close() string
}

type File struct { name string }

func (file File) Read() string { return "read:" + file.name }
func (file File) Close() string { return "close:" + file.name }

func promote(value ReadCloser) Reader { return value }

func main() {
    var closer ReadCloser = File{name: "alpha"}
    var reader Reader
    reader = closer
    promoted := promote(closer)
    fmt.Println(reader.Read(), promoted.Read(), reader == closer, promoted == closer)

    closer = File{name: "beta"}
    fmt.Println(reader != closer)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "read:alpha read:alpha true true\ntrue\n");
}

#[test]
fn rejects_assigning_interface_sources_that_lack_required_methods() {
    let source = r#"
package main

type Reader interface { Read() string }
type ReadCloser interface {
    Reader
    Close() string
}

func main() {
    var reader Reader
    var closer ReadCloser
    closer = reader
    _ = closer
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("type `Reader` does not satisfy interface `ReadCloser`"));
}

#[test]
fn rejects_returning_interface_sources_that_lack_required_methods() {
    let source = r#"
package main

type Reader interface { Read() string }
type ReadCloser interface {
    Reader
    Close() string
}

func promote(value Reader) ReadCloser {
    return value
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("type `Reader` does not satisfy interface `ReadCloser`"));
}
