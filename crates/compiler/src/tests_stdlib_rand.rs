use crate::workspace::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_rand_intn() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(42)
    fmt.Println(rand.Intn(100))
    fmt.Println(rand.Intn(100))
    fmt.Println(rand.Intn(100))
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    let lines: Vec<&str> = vm.stdout().trim().lines().collect();
    assert_eq!(lines.len(), 3);
    for line in &lines {
        let n: i64 = line.parse().expect("should be a number");
        assert!((0..100).contains(&n), "Intn(100) should be in [0,100): got {n}");
    }
}

#[test]
fn compiles_and_runs_rand_float64() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(42)
    f := rand.Float64()
    fmt.Println(f)
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    let f: f64 = vm.stdout().trim().parse().expect("should be a float");
    assert!((0.0..1.0).contains(&f), "Float64() should be in [0,1): got {f}");
}

#[test]
fn compiles_and_runs_rand_seed_determinism() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(123)
    a := rand.Intn(1000)
    b := rand.Intn(1000)
    rand.Seed(123)
    c := rand.Intn(1000)
    d := rand.Intn(1000)
    if a == c && b == d {
        fmt.Println("deterministic")
    } else {
        fmt.Println("not deterministic")
    }
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "deterministic\n");
}

#[test]
fn compiles_and_runs_rand_int_and_int63() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(7)
    n := rand.Int()
    if n >= 0 {
        fmt.Println("int ok")
    }
    n63 := rand.Int63()
    if n63 >= 0 {
        fmt.Println("int63 ok")
    }
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "int ok\nint63 ok\n");
}

#[test]
fn compiles_and_runs_rand_int31_and_int31n() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(99)
    n31 := rand.Int31()
    if n31 >= 0 {
        fmt.Println("int31 ok")
    }
    n31n := rand.Int31n(50)
    if n31n >= 0 && n31n < 50 {
        fmt.Println("int31n ok")
    }
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "int31 ok\nint31n ok\n");
}

#[test]
fn compiles_and_runs_rand_int63n() {
    let source = r#"
package main
import "fmt"
import "math/rand"

func main() {
    rand.Seed(55)
    n := rand.Int63n(1000)
    if n >= 0 && n < 1000 {
        fmt.Println("ok")
    }
}
"#;
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ok\n");
}
