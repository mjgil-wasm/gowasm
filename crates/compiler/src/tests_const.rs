use super::{compile_source, CompileError};
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_local_const_declarations() {
    let source = r#"
package main
import "fmt"

func main() {
    const answer = 42
    const greeting string = "go"
    const ready = true
    fmt.Println(answer, greeting, ready)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 go true\n");
}

#[test]
fn rejects_non_const_local_const_initializers() {
    let source = r#"
package main

func main() {
    value := 7
    const answer = value
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("const initializers can only reference const bindings")
    ));
}

#[test]
fn compiles_and_runs_package_const_declarations() {
    let source = r#"
package main
import "fmt"

const answer = 42
const greeting string = "go"

func main() {
    fmt.Println(answer, greeting)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 go\n");
}

#[test]
fn rejects_non_const_package_const_initializers() {
    let source = r#"
package main

var value = 7
const answer = value

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("package const initializers can only reference earlier consts")
    ));
}

#[test]
fn rejects_assignments_to_local_consts() {
    let source = r#"
package main

func main() {
    const answer = 42
    answer = 7
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("cannot assign to const `answer`")
    ));
}

#[test]
fn rejects_modifying_package_consts() {
    let source = r#"
package main

const answer = 42

func main() {
    answer++
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("cannot modify const `answer`")
    ));
}

#[test]
fn compiles_and_runs_const_expression_initializers() {
    let source = r#"
package main
import "fmt"

const base = 40
const answer = base + 2
const ready = answer == 42
const label = "go" + "wasm"

func main() {
    const localBase = answer
    const local = localBase + 5
    fmt.Println(local, ready, label)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "47 true gowasm\n");
}

#[test]
fn compiles_and_runs_grouped_const_declarations() {
    let source = r#"
package main
import "fmt"

const (
    answer = 42
    greeting string = "go"
)

func main() {
    const (
        ready = true
        label = greeting + "wasm"
    )
    fmt.Println(answer, ready, label)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true gowasm\n");
}

#[test]
fn compiles_and_runs_iota_const_declarations() {
    let source = r#"
package main
import "fmt"

const (
    first = iota
    second = iota + 1
)

func main() {
    const local = iota
    fmt.Println(first, second, local)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 2 0\n");
}

#[test]
fn compiles_and_runs_elided_grouped_const_initializers() {
    let source = r#"
package main
import "fmt"

const (
    first = iota
    second
    third
)

func main() {
    const (
        greeting = "go"
        label
    )
    fmt.Println(first, second, third, label)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 1 2 go\n");
}

#[test]
fn compiles_and_runs_left_shift_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 1 << iota
    second = 1 << iota
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n");
}

#[test]
fn compiles_and_runs_right_shift_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 8 >> iota
    second = 8 >> iota
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8 4\n");
}

#[test]
fn compiles_and_runs_bitwise_or_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 1 | 2
    second = (1 << iota) | 1
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 3\n");
}

#[test]
fn compiles_and_runs_bitwise_and_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 7 & 3
    second = (7 >> iota) & 3
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 3\n");
}

#[test]
fn compiles_and_runs_bitwise_xor_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 7 ^ 3
    second = (4 << iota) ^ 3
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 11\n");
}

#[test]
fn compiles_and_runs_bitwise_clear_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = 7 &^ 3
    second = (7 >> iota) &^ 1
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 2\n");
}

#[test]
fn compiles_and_runs_unary_bitwise_not_const_expressions() {
    let source = r#"
package main
import "fmt"

const (
    first = ^0
    second = ^iota
)

func main() {
    fmt.Println(first, second)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1 -2\n");
}

#[test]
fn rejects_forward_package_const_references() {
    let source = r#"
package main

const answer = base + 2
const base = 40

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("package const initializers can only reference earlier consts")
    ));
}

#[test]
fn compiles_and_runs_untyped_const_defaulting_and_assignment() {
    let source = r#"
package main
import "fmt"

const answer = 6
const ratio = answer + 0.5

func main() {
    var asFloat float64 = answer
    var asByte byte = 255
    var asRune rune = 'A'
    fmt.Println(answer, asFloat, ratio, asByte, asRune)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6 6.0 6.5 255 65\n");
}

#[test]
fn compiles_and_runs_const_identifiers_in_make_bounds() {
    let source = r#"
package main
import "fmt"

const size = 2
const capacity = size + 1

func main() {
    values := make([]int, size, capacity)
    fmt.Println(len(values), cap(values))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 3\n");
}

#[test]
fn compiles_and_runs_generic_calls_with_untyped_const_arguments() {
    let source = r#"
package main
import "fmt"

const answer = 7
const ratio = 1.5

func echo[T any](value T) T {
    return value
}

func main() {
    fmt.Println(echo(answer), echo(ratio))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 1.5\n");
}

#[test]
fn rejects_byte_const_overflow() {
    let source = r#"
package main

const limit = 256
var value byte = limit

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("not representable as `byte`")
    ));
}

#[test]
fn rejects_float_const_assignment_to_int() {
    let source = r#"
package main

const ratio = 3.5
var count int = ratio

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("constant of type `float64` is not assignable to `int`")
    ));
}

#[test]
fn rejects_const_shift_overflow() {
    let source = r#"
package main

const tooBig = 1 << 63

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(matches!(
        error,
        CompileError::Unsupported { detail }
        if detail.contains("overflows supported int range")
    ));
}
