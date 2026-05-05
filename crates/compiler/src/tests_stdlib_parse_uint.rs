use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn parse_uint_decimal() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("42", 10, 64)
    fmt.Println(n, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 <nil>\n");
}

#[test]
fn parse_uint_hex() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("ff", 16, 64)
    fmt.Println(n, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "255 <nil>\n");
}

#[test]
fn parse_uint_binary() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("1010", 2, 64)
    fmt.Println(n, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10 <nil>\n");
}

#[test]
fn parse_uint_invalid() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("abc", 10, 64)
    fmt.Println(n)
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0\nstrconv.ParseUint: parsing \"abc\": invalid syntax\n"
    );
}

#[test]
fn parse_uint_negative_is_error() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("-1", 10, 64)
    fmt.Println(n)
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0\nstrconv.ParseUint: parsing \"-1\": invalid syntax\n"
    );
}

#[test]
fn parse_uint_auto_base() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseUint("0xff", 0, 64)
    fmt.Println(n, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "255 <nil>\n");
}
