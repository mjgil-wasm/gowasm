use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn math_cbrt() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Cbrt(27.0))
    fmt.Println(math.Cbrt(8.0))
    fmt.Println(math.Cbrt(-8.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.0\n2.0\n-2.0\n");
}

#[test]
fn math_float64_bits_and_from_bits() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    bits := math.Float64bits(1.0)
    fmt.Println(bits)
    f := math.Float64frombits(bits)
    fmt.Println(f)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4607182418800017408\n1.0\n");
}

#[test]
fn math_logb() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Logb(8.0))
    fmt.Println(math.Logb(1.0))
    fmt.Println(math.Logb(0.5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.0\n0.0\n-1.0\n");
}

#[test]
fn math_ilogb() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Ilogb(8.0))
    fmt.Println(math.Ilogb(1.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n0\n");
}

#[test]
fn math_subnormal_frexp_logb_and_ilogb() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    frac, exp := math.Frexp(math.SmallestNonzeroFloat64)
    fmt.Println(frac, exp)
    fmt.Println(math.Logb(math.SmallestNonzeroFloat64))
    fmt.Println(math.Ilogb(math.SmallestNonzeroFloat64))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.5 -1073\n-1074.0\n-1074\n");
}

#[test]
fn math_modf() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    integer, frac := math.Modf(3.75)
    fmt.Println(integer)
    fmt.Println(frac)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.0\n0.75\n");
}

#[test]
fn math_min_int_constants() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.MinInt8)
    fmt.Println(math.MinInt16)
    fmt.Println(math.MinInt32)
    fmt.Println(math.MinInt64)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "-128\n-32768\n-2147483648\n-9223372036854775808\n"
    );
}

#[test]
fn math_float32_constants() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.MaxFloat32 > 1.0)
    fmt.Println(math.SmallestNonzeroFloat32 > 0.0)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\n");
}
