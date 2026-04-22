use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn bits_ones_count() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.OnesCount(0))
    fmt.Println(bits.OnesCount(1))
    fmt.Println(bits.OnesCount(255))
    fmt.Println(bits.OnesCount(7))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n8\n3\n");
}

#[test]
fn bits_leading_zeros() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.LeadingZeros(0))
    fmt.Println(bits.LeadingZeros(1))
    fmt.Println(bits.LeadingZeros(256))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n63\n55\n");
}

#[test]
fn bits_trailing_zeros() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.TrailingZeros(0))
    fmt.Println(bits.TrailingZeros(1))
    fmt.Println(bits.TrailingZeros(8))
    fmt.Println(bits.TrailingZeros(12))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n0\n3\n2\n");
}

#[test]
fn bits_len() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.Len(0))
    fmt.Println(bits.Len(1))
    fmt.Println(bits.Len(255))
    fmt.Println(bits.Len(256))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n8\n9\n");
}

#[test]
fn bits_rotate_left() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.RotateLeft(1, 3))
    fmt.Println(bits.RotateLeft(16, 60))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8\n1\n");
}

#[test]
fn bits_reverse_bytes() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.ReverseBytes(256))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "281474976710656\n");
}

#[test]
fn bits_reverse_and_negative_rotate_left() {
    let source = r#"
package main
import "fmt"
import "math/bits"

func main() {
    fmt.Println(bits.Reverse(6))
    fmt.Println(bits.RotateLeft(16, -4))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6917529027641081856\n1\n");
}
