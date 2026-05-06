use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strings_split_after_basic() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    parts := strings.SplitAfter("a,b,c", ",")
    fmt.Println(len(parts))
    fmt.Println(parts[0])
    fmt.Println(parts[1])
    fmt.Println(parts[2])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\na,\nb,\nc\n");
}

#[test]
fn strings_split_after_no_match() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    parts := strings.SplitAfter("hello", ",")
    fmt.Println(len(parts))
    fmt.Println(parts[0])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\nhello\n");
}

#[test]
fn strings_split_after_multichar_sep() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    parts := strings.SplitAfter("one::two::three", "::")
    fmt.Println(len(parts))
    fmt.Println(parts[0])
    fmt.Println(parts[1])
    fmt.Println(parts[2])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\none::\ntwo::\nthree\n");
}
