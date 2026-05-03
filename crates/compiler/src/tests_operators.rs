use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_raw_string_literal() {
    let source = r#"
package main
import "fmt"

func main() {
    s := `hello\nworld`
    fmt.Println(s)
    fmt.Println(len(s))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\\nworld\n12\n");
}

#[test]
fn compiles_and_runs_modulo_operator() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(10 % 3)
    fmt.Println(15 % 5)
    fmt.Println(7 % 4)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n0\n3\n");
}

#[test]
fn compiles_and_runs_compound_assignment_operators() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 10
    x += 5
    fmt.Println(x)
    x -= 3
    fmt.Println(x)
    x *= 2
    fmt.Println(x)
    x /= 4
    fmt.Println(x)
    x %= 2
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15\n12\n24\n6\n0\n");
}

#[test]
fn compiles_and_runs_string_concat_with_plus_equal() {
    let source = r#"
package main
import "fmt"

func main() {
    s := "hello"
    s += " world"
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}

#[test]
fn compiles_and_runs_grouped_imports() {
    let source = r#"
package main

import (
    "fmt"
    "strings"
)

func main() {
    fmt.Println(strings.ToUpper("hello"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "HELLO\n");
}

#[test]
fn compiles_and_runs_int_to_float64_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 42
    y := float64(x)
    fmt.Println(y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42.0\n");
}

#[test]
fn compiles_and_runs_float64_to_int_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 3.7
    y := int(x)
    fmt.Println(y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn compiles_and_runs_int_to_string_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 65
    s := string(x)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "A\n");
}

#[test]
fn compiles_and_runs_byte_conversion() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 300
    b := byte(x)
    fmt.Println(b)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "44\n");
}

#[test]
fn compiles_and_runs_hex_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(0xFF)
    fmt.Println(0x1A)
    fmt.Println(0x0)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "255\n26\n0\n");
}

#[test]
fn compiles_and_runs_rune_literals() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println('A')
    fmt.Println('z')
    fmt.Println('\n' == 10)
    fmt.Println('\t' == 9)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65\n122\ntrue\ntrue\n");
}

#[test]
fn compiles_and_runs_os_exit() {
    let source = r#"
package main

import (
    "fmt"
    "os"
)

func main() {
    fmt.Println("before")
    os.Exit(0)
    fmt.Println("after")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let result = vm.run_program(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("os.Exit(0)"));
    assert_eq!(vm.stdout(), "before\n");
}

#[test]
fn compiles_and_runs_rune_in_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    ch := 'H'
    fmt.Println(string(ch))
    fmt.Println(ch >= 'A' && ch <= 'Z')
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "H\ntrue\n");
}

#[test]
fn compiles_and_runs_bitwise_compound_assignments() {
    let source = r#"
package main
import "fmt"

func main() {
    x := 0xFF
    x &= 0x0F
    fmt.Println(x)
    x |= 0xF0
    fmt.Println(x)
    x ^= 0x55
    fmt.Println(x)
    y := 1
    y <<= 4
    fmt.Println(y)
    y >>= 2
    fmt.Println(y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15\n255\n170\n16\n4\n");
}

#[test]
fn compiles_and_runs_string_slice_expressions() {
    let source = r#"
package main
import "fmt"

func main() {
    s := "hello world"
    fmt.Println(s[0:5])
    fmt.Println(s[6:])
    fmt.Println(s[:5])
    fmt.Println(s[:])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\nworld\nhello\nhello world\n");
}

#[test]
fn compiles_and_runs_slice_of_slice() {
    let source = r#"
package main
import "fmt"

func main() {
    s := []int{1, 2, 3, 4, 5}
    fmt.Println(s[1:3])
    fmt.Println(s[:2])
    fmt.Println(s[3:])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[2 3]\n[1 2]\n[4 5]\n");
}


#[test]
fn integer_overflow_wraps_in_twos_complement() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    max := math.MaxInt64
    min := math.MinInt64
    fmt.Println(max + 1)
    fmt.Println(min - 1)
    fmt.Println(max * 2)
    fmt.Println(min + min)
    fmt.Println(-min)
    fmt.Println(max % -1)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "-9223372036854775808\n",
            "9223372036854775807\n",
            "-2\n",
            "0\n",
            "-9223372036854775808\n",
            "0\n",
        )
    );
}
