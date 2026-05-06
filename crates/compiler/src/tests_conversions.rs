use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn int_to_float64_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 42
    f := float64(x)
    fmt.Println(f)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42.0\n");
}

#[test]
fn float64_to_int_truncation() {
    let source = r#"
package main
import "fmt"

func main() {
    f := 3.7
    n := int(f)
    fmt.Println(n)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn int_to_string_rune_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    n := 65
    s := string(n)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "A\n");
}

#[test]
fn string_to_byte_slice_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    s := "hello"
    b := []byte(s)
    fmt.Println(len(b))
    fmt.Println(b[0], b[4])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n104 111\n");
}

#[test]
fn float64_negative_to_int() {
    let source = r#"
package main
import "fmt"

func main() {
    f := -2.9
    n := int(f)
    fmt.Println(n)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-2\n");
}

#[test]
fn conversion_in_arithmetic_expression() {
    let source = r#"
package main
import "fmt"

func main() {
    a := 10
    b := 3
    result := float64(a) / float64(b)
    fmt.Printf("%.4f\n", result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.3333\n");
}

#[test]
fn byte_to_int_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    s := "Z"
    b := []byte(s)
    n := int(b[0])
    fmt.Println(n)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "90\n");
}

#[test]
fn int_to_string_unicode_rune() {
    let source = r#"
package main
import "fmt"

func main() {
    heart := 9829
    fmt.Println(string(heart))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\u{2665}\n");
}

#[test]
fn byte_conversion_wraps_runtime_values() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 257
    y := -1
    z := 258.75
    fmt.Println(byte(x), byte(y), byte(z))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 255 2\n");
}

#[test]
fn string_from_utf8_byte_slice_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    snowman := []byte{226, 152, 131}
    fmt.Printf("%q\n", string(snowman))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"☃\"\n");
}

#[test]
fn rune_conversion_flows_through_string() {
    let source = r#"
package main
import "fmt"

func main() {
    r := rune(9731)
    fmt.Printf("%q\n", string(r))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"☃\"\n");
}

#[test]
fn rune_slice_from_string_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    runes := []rune("hé世")
    fmt.Println(len(runes))
    fmt.Println(runes[0], runes[1], runes[2])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n104 233 19990\n");
}

#[test]
fn string_from_rune_slice_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    runes := []rune{9731, 233, 65}
    fmt.Printf("%q\n", string(runes))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"☃éA\"\n");
}

#[test]
fn named_byte_conversion_wraps_runtime_values() {
    let source = r#"
package main
import "fmt"

type Octet byte

func main() {
    value := 257
    fmt.Println(Octet(value))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn rejects_string_to_int_conversion() {
    let source = r#"
package main

func main() {
    _ = int("65")
}
"#;

    let error = compile_source(source).expect_err("string to int conversion should fail");
    assert!(
        error
            .to_string()
            .contains("cannot convert expression of type `string` to `int`"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_const_byte_conversion_overflow() {
    let source = r#"
package main

func main() {
    _ = byte(300)
}
"#;

    let error = compile_source(source).expect_err("const byte overflow should fail");
    assert!(
        error
            .to_string()
            .contains("constant 300 is not representable as `byte`"),
        "unexpected error: {error}"
    );
}
