use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn base64_std_encoding_encode_to_string_basic() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    src := []byte{72, 101, 108, 108, 111}
    fmt.Println(base64.StdEncodingEncodeToString(src))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "SGVsbG8=\n");
}

#[test]
fn base64_std_encoding_decode_string_basic() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    decoded, err := base64.StdEncodingDecodeString("SGVsbG8=")
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
fn base64_url_encoding_wrappers_use_url_alphabet() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    src := []byte{251, 255}
    fmt.Println(base64.URLEncodingEncodeToString(src))
    decoded, err := base64.URLEncodingDecodeString("-_8=")
    fmt.Println(len(decoded), decoded[0], decoded[1], err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-_8=\n2 251 255 <nil>\n");
}

#[test]
fn base64_raw_encoding_wrappers_round_trip_without_padding() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    fmt.Println(base64.RawStdEncodingEncodeToString([]byte{102}))
    stdDecoded, stdErr := base64.RawStdEncodingDecodeString("Zg")
    fmt.Println(len(stdDecoded), stdDecoded[0], stdErr)

    fmt.Println(base64.RawURLEncodingEncodeToString([]byte{251, 255}))
    urlDecoded, urlErr := base64.RawURLEncodingDecodeString("-_8")
    fmt.Println(len(urlDecoded), urlDecoded[0], urlDecoded[1], urlErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Zg\n1 102 <nil>\n-_8\n2 251 255 <nil>\n");
}

#[test]
fn base64_decode_string_invalid_input_returns_error() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    _, err := base64.StdEncodingDecodeString("@@==")
    fmt.Println(err != nil)
    _, rawErr := base64.RawStdEncodingDecodeString("Zg=")
    fmt.Println(rawErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\n");
}

#[test]
fn base64_encoding_values_support_method_calls() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    enc := base64.StdEncoding
    fmt.Println(enc.EncodeToString([]byte{72, 101, 108, 108, 111}))
    decoded, err := enc.DecodeString("SGVsbG8=")
    fmt.Println(len(decoded), err)
    fmt.Println(decoded[0], decoded[1], decoded[2], decoded[3], decoded[4])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "SGVsbG8=\n5 <nil>\n72 101 108 108 111\n");
}

#[test]
fn base64_url_and_raw_encoding_values_round_trip() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    fmt.Println(base64.URLEncoding.EncodeToString([]byte{251, 255}))
    decoded, err := base64.RawURLEncoding.DecodeString("-_8")
    fmt.Println(len(decoded), decoded[0], decoded[1], err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-_8=\n2 251 255 <nil>\n");
}

#[test]
fn base64_encoding_method_values_are_callable() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    encode := base64.StdEncoding.EncodeToString

    fmt.Println(encode([]byte("Hello")))
    decoded, err := base64.RawURLEncoding.DecodeString("-_8")
    fmt.Println(len(decoded), decoded[0], decoded[1], err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "SGVsbG8=\n2 251 255 true\n");
}

#[test]
fn base64_invalid_decode_matches_go_error_text() {
    let source = r#"
package main
import "fmt"
import "encoding/base64"

func main() {
    _, err := base64.StdEncoding.DecodeString("@@==")
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "illegal base64 data at input byte 0\n");
}
