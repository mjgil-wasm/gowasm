use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn type_assert_float64() {
    let source = r#"
package main
import "fmt"

func main() {
    var x interface{} = 3.14
    f := x.(float64)
    fmt.Println(f)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.14\n");
}

#[test]
fn type_assert_float64_comma_ok() {
    let source = r#"
package main
import "fmt"

func main() {
    var x interface{} = 2.5
    f, ok := x.(float64)
    fmt.Println(f, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2.5 true\n");
}

#[test]
fn type_assert_float64_fails() {
    let source = r#"
package main
import "fmt"

func main() {
    var x interface{} = "hello"
    f, ok := x.(float64)
    fmt.Println(f, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0 false\n");
}

#[test]
fn type_assert_error() {
    let source = r#"
package main
import "fmt"
import "errors"

func main() {
    var x interface{} = errors.New("oops")
    e, ok := x.(error)
    fmt.Println(e, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "oops true\n");
}

#[test]
fn math_max_int() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.MaxInt8)
    fmt.Println(math.MaxInt16)
    fmt.Println(math.MaxInt32)
    fmt.Println(math.MaxUint8)
    fmt.Println(math.MaxUint16)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "127\n32767\n2147483647\n255\n65535\n");
}

#[test]
fn type_switch_float64() {
    let source = r#"
package main
import "fmt"

func main() {
    var x interface{} = 3.14
    switch v := x.(type) {
    case int:
        fmt.Println("int:", v)
    case float64:
        fmt.Println("float64:", v)
    case string:
        fmt.Println("string:", v)
    default:
        fmt.Println("unknown")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "float64: 3.14\n");
}

#[test]
fn strconv_format_uint() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.FormatUint(255, 16))
    fmt.Println(strconv.FormatUint(42, 10))
    fmt.Println(strconv.FormatUint(7, 2))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ff\n42\n111\n");
}

#[test]
fn math_max_int_large() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.MaxUint32)
    fmt.Println(math.MaxInt64)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4294967295\n9223372036854775807\n");
}
