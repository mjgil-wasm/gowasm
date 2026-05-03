use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strconv_itoa() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.Itoa(42))
    fmt.Println(strconv.Itoa(-7))
    fmt.Println(strconv.Itoa(0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n-7\n0\n");
}

#[test]
fn strconv_format_bool() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.FormatBool(true))
    fmt.Println(strconv.FormatBool(false))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}

#[test]
fn strconv_format_int_bases() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.FormatInt(255, 16))
    fmt.Println(strconv.FormatInt(255, 2))
    fmt.Println(strconv.FormatInt(255, 8))
    fmt.Println(strconv.FormatInt(255, 10))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ff\n11111111\n377\n255\n");
}

#[test]
fn strconv_atoi_success_and_error() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.Atoi("123")
    fmt.Println(n, err)
    n2, err2 := strconv.Atoi("abc")
    fmt.Println(n2, err2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    let output = vm.stdout();
    assert!(output.starts_with("123 <nil>\n0 "));
}

#[test]
fn strconv_parse_bool() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    v, err := strconv.ParseBool("true")
    fmt.Println(v, err)
    v2, err2 := strconv.ParseBool("0")
    fmt.Println(v2, err2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true <nil>\nfalse <nil>\n");
}

#[test]
fn strconv_parse_int_with_base() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    n, err := strconv.ParseInt("ff", 16, 64)
    fmt.Println(n, err)
    n2, err2 := strconv.ParseInt("777", 8, 64)
    fmt.Println(n2, err2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "255 <nil>\n511 <nil>\n");
}

#[test]
fn strconv_quote_and_can_backquote() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.Quote("hello world"))
    fmt.Println(strconv.CanBackquote("hello world"))
    fmt.Println(strconv.CanBackquote("hello\nworld"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"hello world\"\ntrue\nfalse\n");
}

#[test]
fn strconv_quote_rune() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.QuoteRune(65))
    fmt.Println(strconv.QuoteRuneToASCII(19990))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "'A'\n'\\u4e16'\n");
}

#[test]
fn strconv_format_float() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.FormatFloat(3.14159, 'f', 2, 64))
    fmt.Println(strconv.FormatFloat(100.0, 'e', 3, 64))
    fmt.Println(strconv.FormatFloat(3.14, 'g', -1, 64))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3.14\n1.000e+02\n3.14\n");
}

#[test]
fn strconv_parse_float() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    f, err := strconv.ParseFloat("3.14", 64)
    fmt.Println(f, err)
    f2, err2 := strconv.ParseFloat("not_a_number", 64)
    fmt.Println(f2, err2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    let output = vm.stdout();
    assert!(output.starts_with("3.14 <nil>\n0"));
}

#[test]
fn strconv_atoi_max_and_overflow() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.Atoi("9223372036854775807")
    _, overflowErr := strconv.Atoi("9223372036854775808")
    fmt.Println(value, err == nil, overflowErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9223372036854775807 true true\n");
}

#[test]
fn strconv_parse_float_32_bit_overflow() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.ParseFloat("3.5e38", 32)
    fmt.Printf("%g %t\n", value, err != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "+Inf true\n");
}

#[test]
fn strconv_format_float_respects_bit_size() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.FormatFloat(1.23456789, 'g', -1, 32))
    fmt.Println(strconv.FormatFloat(1.23456789, 'g', -1, 64))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1.2345679\n1.23456789\n");
}

#[test]
fn strconv_unquote() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    s, err := strconv.Unquote("\"hello\"")
    fmt.Println(s, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello <nil>\n");
}

#[test]
fn strconv_quote_matches_supported_go_escapes() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(strconv.Quote(string([]byte{7, 10}) + "世"))
    fmt.Println(strconv.Quote(string([]byte{0, 34, 92})))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"\\a\\n世\"\n\"\\x00\\\"\\\\\"\n");
}

#[test]
fn strconv_unquote_and_unquote_char_support_octal_escapes() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    text, textErr := strconv.Unquote("\"\\141\\n\"")
    value, multibyte, tail, err := strconv.UnquoteChar("\\377x", '"')
    fmt.Println(text, textErr)
    fmt.Println(value, multibyte, tail, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a\n <nil>\n255 false x <nil>\n");
}

#[test]
fn strconv_parse_helpers_match_supported_error_text_and_bit_size_rules() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    _, parseIntBaseErr := strconv.ParseInt("10", 1, 64)
    _, parseIntBitsErr := strconv.ParseInt("10", 10, 65)
    _, parseUintBaseErr := strconv.ParseUint("10", 1, 64)
    value, parseFloatErr := strconv.ParseFloat("1.5", 16)
    fmt.Println(parseIntBaseErr)
    fmt.Println(parseIntBitsErr)
    fmt.Println(parseUintBaseErr)
    fmt.Println(value, parseFloatErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "strconv.ParseInt: parsing \"10\": invalid base 1\nstrconv.ParseInt: parsing \"10\": invalid bit size 65\nstrconv.ParseUint: parsing \"10\": invalid base 1\n1.5 <nil>\n"
    );
}
