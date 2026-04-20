use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn math_abs_positive() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Abs(-3.5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.5\n");
}

#[test]
fn math_sqrt_and_floor() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Sqrt(16.0))
    fmt.Println(math.Floor(3.7))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4.0\n3.0\n");
}

#[test]
fn math_ceil_and_round() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Ceil(2.1))
    fmt.Println(math.Round(2.5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.0\n3.0\n");
}

#[test]
fn math_max_and_min() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Max(1.0, 5.0))
    fmt.Println(math.Min(1.0, 5.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5.0\n1.0\n");
}

#[test]
fn math_max_min_nan_and_signed_zero() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.IsNaN(math.Max(math.NaN(), 1.0)))
    fmt.Println(math.IsNaN(math.Min(1.0, math.NaN())))
    fmt.Println(math.Signbit(math.Max(math.Copysign(0.0, -1.0), 0.0)))
    fmt.Println(math.Signbit(math.Min(math.Copysign(0.0, -1.0), 0.0)))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nfalse\ntrue\n");
}

#[test]
fn math_pow_and_mod() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Pow(2.0, 10.0))
    fmt.Println(math.Mod(10.0, 3.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1024.0\n1.0\n");
}

#[test]
fn math_trig_functions() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Sin(0.0))
    fmt.Println(math.Cos(0.0))
    fmt.Println(math.Tan(0.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n1.0\n0.0\n");
}

#[test]
fn math_log_functions() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Log(1.0))
    fmt.Println(math.Log2(8.0))
    fmt.Println(math.Log10(1000.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n3.0\n3.0\n");
}

#[test]
fn math_exp_function() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Exp(0.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1.0\n");
}

#[test]
fn math_inf_and_nan() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.IsNaN(math.NaN()))
    fmt.Println(math.IsInf(math.Inf(1), 1))
    fmt.Println(math.IsInf(math.Inf(-1), -1))
    fmt.Println(math.IsInf(1.0, 0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\ntrue\nfalse\n");
}

#[test]
fn math_inverse_trig() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Asin(0.0))
    fmt.Println(math.Acos(1.0))
    fmt.Println(math.Atan(0.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n0.0\n0.0\n");
}

#[test]
fn math_hyperbolic() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Sinh(0.0))
    fmt.Println(math.Cosh(0.0))
    fmt.Println(math.Tanh(0.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n1.0\n0.0\n");
}

#[test]
fn math_constants() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Pi > 3.14)
    fmt.Println(math.E > 2.71)
    fmt.Println(math.Sqrt2 > 1.41)
    fmt.Println(math.Ln2 > 0.69)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\ntrue\ntrue\n");
}

#[test]
fn math_trunc_and_dim() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Trunc(2.9))
    fmt.Println(math.Dim(5.0, 3.0))
    fmt.Println(math.Dim(3.0, 5.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2.0\n2.0\n0.0\n");
}

#[test]
fn math_copysign_and_signbit() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Copysign(3.0, -1.0))
    fmt.Println(math.Signbit(-5.0))
    fmt.Println(math.Signbit(5.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-3.0\ntrue\nfalse\n");
}

#[test]
fn math_atan2_and_hypot() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Atan2(0.0, 1.0))
    fmt.Println(math.Hypot(3.0, 4.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n5.0\n");
}

#[test]
fn math_remainder() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Remainder(10.0, 3.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1.0\n");
}

#[test]
fn math_ldexp() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Ldexp(1.0, 10))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1024.0\n");
}

#[test]
fn math_expm1_and_log1p() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    fmt.Println(math.Expm1(0.0))
    fmt.Println(math.Log1p(0.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0\n0.0\n");
}

#[test]
fn math_frexp() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    frac, exp := math.Frexp(8.0)
    fmt.Println(frac, exp)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.5 4\n");
}

#[test]
fn math_frexp_zero() {
    let source = r#"
package main
import "fmt"
import "math"

func main() {
    frac, exp := math.Frexp(0.0)
    fmt.Println(frac, exp)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0.0 0\n");
}
