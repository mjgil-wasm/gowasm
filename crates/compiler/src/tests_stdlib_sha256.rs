use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sha256_sum256_hello() {
    let source = r#"
package main
import "fmt"
import "crypto/sha256"

func main() {
    data := []byte("hello")
    hash := sha256.Sum256(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[28], hash[29], hash[30], hash[31])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "44 242 77 186\n147 139 152 36\n");
}

#[test]
fn sha256_sum256_empty() {
    let source = r#"
package main
import "fmt"
import "crypto/sha256"

func main() {
    data := []byte("")
    hash := sha256.Sum256(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[28], hash[29], hash[30], hash[31])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "227 176 196 66\n120 82 184 85\n");
}

#[test]
fn sha256_sum256_len() {
    let source = r#"
package main
import "fmt"
import "crypto/sha256"

func main() {
    data := []byte("test")
    hash := sha256.Sum256(data)
    fmt.Println(len(hash))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "32\n");
}

#[test]
fn sha256_block_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha256"

func main() {
    fmt.Println(sha256.BlockSize)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n");
}

#[test]
fn sha256_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha256"

func main() {
    fmt.Println(sha256.Size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "32\n");
}
