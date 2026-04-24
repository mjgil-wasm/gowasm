use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn utf8_rune_count_in_string_ascii() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneCountInString("hello"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn utf8_rune_count_in_string_multibyte() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneCountInString("Hello, 世界"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn utf8_valid_string_returns_true() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.ValidString("hello"))
    fmt.Println(utf8.ValidString("世界"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\n");
}

#[test]
fn utf8_rune_len_various() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneLen(65))
    fmt.Println(utf8.RuneLen(228))
    fmt.Println(utf8.RuneLen(19990))
    fmt.Println(utf8.RuneLen(128512))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n3\n4\n");
}

#[test]
fn utf8_valid_rune() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.ValidRune(65))
    fmt.Println(utf8.ValidRune(1114111))
    fmt.Println(utf8.ValidRune(1114112))
    fmt.Println(utf8.ValidRune(55296))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nfalse\nfalse\n");
}

#[test]
fn utf8_constants() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneError)
    fmt.Println(utf8.MaxRune)
    fmt.Println(utf8.UTFMax)
    fmt.Println(utf8.RuneSelf)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65533\n1114111\n4\n128\n");
}

#[test]
fn utf8_full_rune_in_string() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.FullRuneInString("a"))
    fmt.Println(utf8.FullRuneInString("世"))
    fmt.Println(utf8.FullRuneInString(""))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nfalse\n");
}

#[test]
fn utf8_rune_count_in_string_empty() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneCountInString(""))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn utf8_decode_rune_in_string_ascii() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    r, size := utf8.DecodeRuneInString("A")
    fmt.Println(r, size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65 1\n");
}

#[test]
fn utf8_decode_rune_in_string_multibyte() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    r, size := utf8.DecodeRuneInString("世界")
    fmt.Println(r, size)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "19990 3\n");
}

#[test]
fn utf8_rune_count_invalid_byte_sequences() {
    let source = r#"
package main
import "fmt"
import "unicode/utf8"

func main() {
    fmt.Println(utf8.RuneCount([]byte{0xff, 'x'}))
    fmt.Println(utf8.RuneCount([]byte{0xe2, 0x98}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n2\n");
}
