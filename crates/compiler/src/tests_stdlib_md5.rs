use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn md5_sum_hello() {
    let source = r#"
package main
import "fmt"
import "crypto/md5"

func main() {
    data := []byte("hello")
    hash := md5.Sum(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[12], hash[13], hash[14], hash[15])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // md5("hello") = 5d41402abc4b2a76b9719d911017c592
    assert_eq!(vm.stdout(), "93 65 64 42\n16 23 197 146\n");
}

#[test]
fn md5_sum_empty() {
    let source = r#"
package main
import "fmt"
import "crypto/md5"

func main() {
    data := []byte("")
    hash := md5.Sum(data)
    fmt.Println(hash[0], hash[1], hash[2], hash[3])
    fmt.Println(hash[12], hash[13], hash[14], hash[15])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    // md5("") = d41d8cd98f00b204e9800998ecf8427e
    assert_eq!(vm.stdout(), "212 29 140 217\n236 248 66 126\n");
}

#[test]
fn md5_sum_len() {
    let source = r#"
package main
import "fmt"
import "crypto/md5"

func main() {
    data := []byte("test")
    hash := md5.Sum(data)
    fmt.Println(len(hash))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "16\n");
}

#[test]
fn md5_block_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/md5"

func main() {
    fmt.Println(md5.BlockSize)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "64\n");
}

#[test]
fn md5_size_constant() {
    let source = r#"
package main
import "fmt"
import "crypto/md5"

func main() {
    fmt.Println(md5.Size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "16\n");
}
