use gowasm_vm::Vm;

use crate::{compile_source, CompileError};

#[test]
fn compiles_and_runs_multi_result_functions_in_statement_position() {
    let source = r#"
package main
import "fmt"

func pair() (int, string) {
    return 7, "go"
}

func main() {
    pair()
    fmt.Println("ok")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ok\n");
}

#[test]
fn rejects_return_count_mismatches_for_multi_result_functions() {
    let source = r#"
package main

func pair() (int, string) {
    return 7
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should fail to compile");
    match error {
        CompileError::Unsupported { detail } => {
            assert!(detail.contains("return value count 1 does not match 2 declared result"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compiles_and_runs_short_decl_from_multi_result_calls() {
    let source = r#"
package main
import "fmt"

func pair() (int, string) {
    return 7, "go"
}

func main() {
    number, word := pair()
    fmt.Println(number, word)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go\n");
}

#[test]
fn compiles_and_runs_multi_result_return_forwarding() {
    let source = r#"
package main
import "fmt"

func pair() (int, string) {
    return 7, "go"
}

func forward() (int, string) {
    return pair()
}

func main() {
    number, word := forward()
    fmt.Println(number, word)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go\n");
}

#[test]
fn compiles_and_runs_strings_cut_prefix_pairs() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    after, found := strings.CutPrefix("gowasm", "go")
    fmt.Println(after, found)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "wasm true\n");
}

#[test]
fn compiles_and_runs_strings_cut_suffix_pairs() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    before, found := strings.CutSuffix("gowasm", "wasm")
    fmt.Println(before, found)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go true\n");
}

#[test]
fn compiles_and_runs_triple_short_decls_from_multi_result_calls() {
    let source = r#"
package main
import "fmt"

func triple() (int, string, bool) {
    return 7, "go", true
}

func main() {
    number, word, ok := triple()
    fmt.Println(number, word, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go true\n");
}

#[test]
fn compiles_and_runs_triple_assignments_from_multi_result_calls() {
    let source = r#"
package main
import "fmt"

func triple() (int, string, bool) {
    return 7, "go", true
}

func main() {
    var number int
    var word string
    var ok bool
    number, word, ok = triple()
    fmt.Println(number, word, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go true\n");
}

#[test]
fn compiles_and_runs_strings_cut_triples() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    before, after, found := strings.Cut("go:wasm", ":")
    fmt.Println(before, after, found)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go wasm true\n");
}

#[test]
fn compiles_and_runs_quad_assignments_from_multi_result_calls() {
    let source = r#"
package main
import "fmt"

func quad() (int, string, bool, int) {
    return 7, "go", true, 9
}

func main() {
    var number int
    var word string
    var ok bool
    var extra int
    number, word, ok, extra = quad()
    fmt.Println(number, word, ok, extra)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 go true 9\n");
}

#[test]
fn compiles_and_runs_strconv_unquote_char() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, multibyte, tail, err := strconv.UnquoteChar("\\nrest", 34)
    runeValue, runeMultibyte, runeTail, runeErr := strconv.UnquoteChar("λx", 34)
    badValue, badMultibyte, badTail, badErr := strconv.UnquoteChar("\"rest", 34)
    fmt.Println(
        value, multibyte, tail, err == nil,
        runeValue, runeMultibyte, runeTail, runeErr == nil,
        badValue, badMultibyte, badTail, badErr != nil, badErr
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "10 false rest true 955 true x true 0 false  true strconv.UnquoteChar: parsing \"\\\"rest\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_blank_identifier_multi_result_discards() {
    let source = r#"
package main
import "fmt"
import "strconv"

func pair() (int, string) {
    return 7, "go"
}

func main() {
    _, firstWord := pair()
    var secondWord string
    _, secondWord = pair()
    _, _, tail, err := strconv.UnquoteChar("\\nrest", 34)
    fmt.Println(firstWord, secondWord, tail, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go go rest true\n");
}

#[test]
fn compiles_and_runs_strconv_unquote_char_hex_and_unicode_escapes() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    hexValue, hexMultibyte, hexTail, hexErr := strconv.UnquoteChar("\\x41rest", 34)
    unicodeValue, unicodeMultibyte, unicodeTail, unicodeErr := strconv.UnquoteChar("\\u03bbx", 34)
    longValue, longMultibyte, longTail, longErr := strconv.UnquoteChar("\\U0001f642!", 34)
    badValue, badMultibyte, badTail, badErr := strconv.UnquoteChar("\\u00zz", 34)
    fmt.Println(
        hexValue, hexMultibyte, hexTail, hexErr == nil,
        unicodeValue, unicodeMultibyte, unicodeTail, unicodeErr == nil,
        longValue, longMultibyte, longTail, longErr == nil,
        badValue, badMultibyte, badTail, badErr != nil, badErr
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "65 false rest true 955 true x true 128578 true ! true 0 false  true strconv.UnquoteChar: parsing \"\\\\u00zz\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_named_return_bare() {
    let source = r#"
package main
import "fmt"

func divide(a int, b int) (result int, ok bool) {
    if b == 0 {
        return
    }
    result = a / b
    ok = true
    return
}

func main() {
    r, ok := divide(10, 2)
    fmt.Println(r, ok)
    r2, ok2 := divide(10, 0)
    fmt.Println(r2, ok2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 true\n0 false\n");
}

#[test]
fn compiles_and_runs_named_return_with_explicit_values() {
    let source = r#"
package main
import "fmt"

func greet(name string) (msg string) {
    msg = "hello " + name
    return msg
}

func main() {
    fmt.Println(greet("world"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}

#[test]
fn compiles_and_runs_named_return_single() {
    let source = r#"
package main
import "fmt"

func doubled(x int) (result int) {
    result = x * 2
    return
}

func main() {
    fmt.Println(doubled(21))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}
