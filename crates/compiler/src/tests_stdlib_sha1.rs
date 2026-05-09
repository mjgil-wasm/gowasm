use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sha1_sum_hello() {
    let source = r#"
package main
import "fmt"
import "crypto/sha1"

func main() {
    data := []byte("hello")
    hash := sha1.Sum(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[16], hash[17], hash[18], hash[19])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // sha1("hello") = aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
    assert_eq!(vm.stdout(), "170 244 198 29\n174 169 67 77\n");
}

#[test]
fn sha1_sum_empty() {
    let source = r#"
package main
import "fmt"
import "crypto/sha1"

func main() {
    data := []byte("")
    hash := sha1.Sum(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[16], hash[17], hash[18], hash[19])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // sha1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
    assert_eq!(vm.stdout(), "218 57 163 238\n175 216 7 9\n");
}

#[test]
fn sha1_sum_len() {
    let source = r#"
package main
import "fmt"
import "crypto/sha1"

func main() {
    data := []byte("test")
    hash := sha1.Sum(data)
    fmt.Println(len(hash))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "20\n");
}

#[test]
fn sha1_block_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha1"

func main() {
    fmt.Println(sha1.BlockSize)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n");
}

#[test]
fn sha1_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha1"

func main() {
    fmt.Println(sha1.Size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "20\n");
}
