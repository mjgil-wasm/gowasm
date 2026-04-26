use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sha512_sum512_hello() {
    let source = r#"
package main
import "fmt"
import "crypto/sha512"

func main() {
    data := []byte("hello")
    hash := sha512.Sum512(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[60], hash[61], hash[62], hash[63])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // sha512("hello") first 4 bytes: 9b71d224, last 4 bytes: bcdec043
    assert_eq!(vm.stdout(), "155 113 210 36\n188 222 192 67\n");
}

#[test]
fn sha512_sum512_empty() {
    let source = r#"
package main
import "fmt"
import "crypto/sha512"

func main() {
    data := []byte("")
    hash := sha512.Sum512(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[60], hash[61], hash[62], hash[63])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // sha512("") first 4 bytes: cf83e135, last 4 bytes: f927da3e
    assert_eq!(vm.stdout(), "207 131 225 53\n249 39 218 62\n");
}

#[test]
fn sha512_sum512_len() {
    let source = r#"
package main
import "fmt"
import "crypto/sha512"

func main() {
    data := []byte("test")
    hash := sha512.Sum512(data)
    fmt.Println(len(hash))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n");
}

#[test]
fn sha512_block_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha512"

func main() {
    fmt.Println(sha512.BlockSize)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "128\n");
}

#[test]
fn sha512_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/sha512"

func main() {
    fmt.Println(sha512.Size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n");
}
