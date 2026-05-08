use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn hex_encode_to_string_basic() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    src := []byte{0x48, 0x65, 0x6c, 0x6c, 0x6f}
    fmt.Println(hex.EncodeToString(src))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "48656c6c6f\n");
}

#[test]
fn hex_encode_to_string_empty() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    src := []byte{}
    fmt.Println(hex.EncodeToString(src))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\n");
}

#[test]
fn hex_decode_string_basic() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    decoded, err := hex.DecodeString("48656c6c6f")
    fmt.Println(len(decoded), err)
    fmt.Println(decoded[0], decoded[1], decoded[2], decoded[3], decoded[4])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 <nil>\n72 101 108 108 111\n");
}

#[test]
fn hex_decode_string_odd_length() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    _, err := hex.DecodeString("4865f")
    fmt.Println(err != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn hex_decode_string_invalid_char() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    _, err := hex.DecodeString("zz")
    fmt.Println(err != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn hex_decode_string_invalid_char_matches_go_error_text() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    _, err := hex.DecodeString("zz")
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "encoding/hex: invalid byte: U+007A 'z'\n");
}

#[test]
fn hex_encoded_len() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    fmt.Println(hex.EncodedLen(5))
    fmt.Println(hex.EncodedLen(0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10\n0\n");
}

#[test]
fn hex_decoded_len() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    fmt.Println(hex.DecodedLen(10))
    fmt.Println(hex.DecodedLen(0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n0\n");
}

#[test]
fn hex_decode_string_uppercase() {
    let source = r#"
package main
import "fmt"
import "encoding/hex"

func main() {
    decoded, err := hex.DecodeString("4A4B")
    fmt.Println(len(decoded), err)
    fmt.Println(decoded[0], decoded[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 <nil>\n74 75\n");
}
