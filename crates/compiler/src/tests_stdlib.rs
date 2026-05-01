use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_strings_search_helpers() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.Contains("gowasm", "was"),
        strings.HasPrefix("gowasm", "go"),
        strings.HasSuffix("gowasm", "asm")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\n");
}

#[test]
fn compiles_and_runs_strings_transform_helpers() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.TrimSpace("  go wasm  "),
        strings.ToUpper("go"),
        strings.ToLower("WASM")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go wasm GO wasm\n");
}

#[test]
fn compiles_and_runs_strconv_format_helpers() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.Itoa(42),
        strconv.FormatBool(true),
        strconv.Quote("go")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true \"go\"\n");
}

#[test]
fn compiles_and_runs_strconv_atoi() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.Atoi("42")
    bad, badErr := strconv.Atoi("go")
    fmt.Println(value, err == nil, bad, badErr != nil, badErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "42 true 0 true strconv.Atoi: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_strconv_parse_bool() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.ParseBool("TRUE")
    bad, badErr := strconv.ParseBool("go")
    fmt.Println(value, err == nil, bad, badErr != nil, badErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true false true strconv.ParseBool: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_errors_new() {
    let source = r#"
package main
import "errors"
import "fmt"

func main() {
    err := errors.New("bad")
    fmt.Println(err != nil, err, err.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true bad bad\n");
}

#[test]
fn compiles_and_runs_strconv_parse_int() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.ParseInt("0xff", 0, 16)
    neg, negErr := strconv.ParseInt("-42", 10, 8)
    bad, badErr := strconv.ParseInt("zig", 10, 64)
    overflow, overflowErr := strconv.ParseInt("128", 10, 8)
    fmt.Println(value, err == nil, neg, negErr == nil, bad, badErr != nil, overflow, overflowErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "255 true -42 true 0 true 0 true\n");
}

#[test]
fn compiles_and_runs_strconv_unquote() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    value, err := strconv.Unquote("\"go\"")
    raw, rawErr := strconv.Unquote("`wasm`")
    bad, badErr := strconv.Unquote("go")
    fmt.Println(value, err == nil, raw, rawErr == nil, bad, badErr != nil, badErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "go true wasm true  true strconv.Unquote: parsing \"go\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_strconv_unquote_hex_and_unicode_escapes() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    hex, hexErr := strconv.Unquote("\"\\x41\"")
    unicode, unicodeErr := strconv.Unquote("\"\\u03bb\"")
    long, longErr := strconv.Unquote("\"\\U0001f642\"")
    runeValue, runeErr := strconv.Unquote("'\\x41'")
    bad, badErr := strconv.Unquote("\"\\u00zz\"")
    fmt.Println(
        hex, hexErr == nil,
        unicode, unicodeErr == nil,
        long, longErr == nil,
        runeValue, runeErr == nil,
        bad, badErr != nil, badErr
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "A true λ true 🙂 true A true  true strconv.Unquote: parsing \"\\\"\\\\u00zz\\\"\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_strings_count() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Count("go go wasm", "go"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_strings_repeat() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Repeat("go", 3))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "gogogo\n");
}

#[test]
fn compiles_and_runs_strings_split() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Split("go,wasm,go", ","))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[go wasm go]\n");
}

#[test]
fn compiles_and_runs_strings_join() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Join([]string{"go", "wasm", "go"}, "-"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go-wasm-go\n");
}

#[test]
fn compiles_and_runs_strings_replace_all() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.ReplaceAll("go wasm go", "go", "zig"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "zig wasm zig\n");
}

#[test]
fn compiles_and_runs_strings_fields() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Fields("  go\twasm  zig  "))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[go wasm zig]\n");
}

#[test]
fn compiles_and_runs_strings_index() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Index("go wasm", "wasm"), strings.Index("go wasm", "zig"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 -1\n");
}

#[test]
fn compiles_and_runs_strings_trim_prefix() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.TrimPrefix("gowasm", "go"),
        strings.TrimPrefix("gowasm", "zig")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "wasm gowasm\n");
}

#[test]
fn compiles_and_runs_strings_trim_suffix() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.TrimSuffix("gowasm", "wasm"),
        strings.TrimSuffix("gowasm", "zig")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go gowasm\n");
}

#[test]
fn compiles_and_runs_strings_last_index() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.LastIndex("go wasm go", "go"), strings.LastIndex("go wasm", "zig"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8 -1\n");
}

#[test]
fn compiles_and_runs_strings_trim_left() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.TrimLeft("..go..", "."))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go..\n");
}

#[test]
fn compiles_and_runs_strings_trim_right() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.TrimRight("..go..", "."))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "..go\n");
}

#[test]
fn compiles_and_runs_strings_trim() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Trim("..go..", "."))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go\n");
}

#[test]
fn compiles_and_runs_strings_contains_any() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.ContainsAny("gowasm", "mx"),
        strings.ContainsAny("gowasm", "zx")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn compiles_and_runs_strings_index_any() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.IndexAny("gowasm", "mx"),
        strings.IndexAny("gowasm", "zx")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 -1\n");
}

#[test]
fn compiles_and_runs_strings_last_index_any() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.LastIndexAny("gowasm", "ma"),
        strings.LastIndexAny("gowasm", "zx")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 -1\n");
}

#[test]
fn compiles_and_runs_strconv_can_backquote() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.CanBackquote("go\twasm"),
        strconv.CanBackquote("go\nwasm"),
        strconv.CanBackquote("`go`")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false false\n");
}

#[test]
fn compiles_and_runs_strconv_format_int() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.FormatInt(255, 16),
        strconv.FormatInt(-5, 2)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ff -101\n");
}

#[test]
fn compiles_and_runs_strings_clone() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.Clone("gowasm"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "gowasm\n");
}

#[test]
fn compiles_and_runs_strings_contains_rune() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.ContainsRune("goλ", 955),
        strings.ContainsRune("gowasm", 955)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn compiles_and_runs_strings_index_rune() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.IndexRune("goλ", 955),
        strings.IndexRune("gowasm", 955)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 -1\n");
}

#[test]
fn compiles_and_runs_strings_compare() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.Compare("go", "go"),
        strings.Compare("go", "hi"),
        strings.Compare("hi", "go")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 -1 1\n");
}

#[test]
fn compiles_and_runs_strconv_quote_to_ascii() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.QuoteToASCII("go\nλ"),
        strconv.QuoteToASCII("go\"")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"go\\n\\u03bb\" \"go\\\"\"\n");
}

#[test]
fn compiles_and_runs_strconv_quote_rune_to_ascii() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.QuoteRuneToASCII(955),
        strconv.QuoteRuneToASCII(1114112)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "'\\u03bb' '\\ufffd'\n");
}

#[test]
fn compiles_and_runs_strconv_quote_rune() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    fmt.Println(
        strconv.QuoteRune(955),
        strconv.QuoteRune(10)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "'λ' '\\n'\n");
}

#[test]
fn compiles_and_runs_strings_replace() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.Replace("go go go", "go", "zig", 2),
        strings.Replace("abc", "", "-", 2)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "zig zig go -a-bc\n");
}

#[test]
fn compiles_and_runs_strings_index_byte() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.IndexByte("gowasm", 119),
        strings.IndexByte("gowasm", 122)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 -1\n");
}

#[test]
fn compiles_and_runs_strings_last_index_byte() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.LastIndexByte("gowasmwow", 119),
        strings.LastIndexByte("gowasm", 122)
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8 -1\n");
}
