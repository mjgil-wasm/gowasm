use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn regexp_match_string_true() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    matched, err := regexp.MatchString("^[a-z]+$", "hello")
    fmt.Println(matched, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true <nil>\n");
}

#[test]
fn regexp_match_string_false() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    matched, err := regexp.MatchString("^[a-z]+$", "Hello123")
    fmt.Println(matched, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false <nil>\n");
}

#[test]
fn regexp_match_string_invalid_pattern() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    matched, err := regexp.MatchString("[invalid", "test")
    fmt.Println(matched)
    fmt.Println(err != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\ntrue\n");
}

#[test]
fn regexp_match_string_digit_pattern() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    m1, _ := regexp.MatchString(`\d+`, "abc123def")
    m2, _ := regexp.MatchString(`^\d+$`, "abc123def")
    m3, _ := regexp.MatchString(`^\d+$`, "12345")
    fmt.Println(m1, m2, m3)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true\n");
}

#[test]
fn regexp_quote_meta() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    fmt.Println(regexp.QuoteMeta("hello.world"))
    fmt.Println(regexp.QuoteMeta("a+b*c?"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\\.world\na\\+b\\*c\\?\n");
}

#[test]
fn regexp_match_string_email_like() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    pattern := `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`
    m1, _ := regexp.MatchString(pattern, "user@example.com")
    m2, _ := regexp.MatchString(pattern, "not-an-email")
    fmt.Println(m1, m2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn regexp_compile_and_match_string_method() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    re, err := regexp.Compile("^[a-z]+$")
    fmt.Println(err == nil)
    fmt.Println(re.MatchString("hello"))
    fmt.Println(re.MatchString("Hello123"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nfalse\n");
}

#[test]
fn regexp_compile_invalid_pattern_returns_error() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    _, err := regexp.Compile("[invalid")
    fmt.Println(err != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn regexp_must_compile_find_string_and_replace_all_string() {
    let source = r##"
package main
import "fmt"
import "regexp"

func main() {
    words := regexp.MustCompile(`[a-z]+`)
    digits := regexp.MustCompile(`\d+`)

    fmt.Println(words.FindString("123abc456"))
    fmt.Println(words.FindString("123456") == "")
    fmt.Println(words.ReplaceAllString("go1 wasm22 zig333", "id"))
    fmt.Println(digits.ReplaceAllString("go1 wasm22 zig333", "#"))
}
"##;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "abc\ntrue\nid1 id22 id333\ngo# wasm# zig#\n");
}

#[test]
fn regexp_find_string_submatch_and_split_methods() {
    let source = r#"
package main
import "fmt"
import "regexp"

func main() {
    re := regexp.MustCompile(`(\pL+)-(\d+)`)
    fmt.Println(re.FindStringSubmatch("xx cafe-12 yy 東京-34"))
    fmt.Println(re.FindStringSubmatch("nope") == nil)

    ws := regexp.MustCompile(`\s+`)
    all := ws.Split("go wasm zig", -1)
    headTail := ws.Split("go wasm zig", 2)
    fmt.Println(len(all), all[0], all[1], all[2])
    fmt.Println(len(headTail), headTail[0], headTail[1])
    fmt.Println(ws.Split("go wasm zig", 0) == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[cafe-12 cafe 12]\ntrue\n3 go wasm zig\n2 go wasm zig\ntrue\n"
    );
}

#[test]
fn regexp_runtime_methods_work_in_value_position() {
    let source = r##"
package main
import "fmt"
import "regexp"

func main() {
    digits := regexp.MustCompile(`(\d+)`)
    commas := regexp.MustCompile(`,+`)

    var replace func(string, string) string = digits.ReplaceAllString
    var split func(string, int) []string = commas.Split
    var submatch func(string) []string = digits.FindStringSubmatch

    fmt.Println(replace("go1 wasm22 zig333", "<$1> $$"))
    parts := split("go,,wasm,zig", -1)
    fmt.Println(len(parts), parts[0], parts[1], parts[2])
    fmt.Println(submatch("id42")[0], submatch("id42")[1])
}
"##;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "go<1> $ wasm<22> $ zig<333> $\n3 go wasm zig\n42 42\n"
    );
}
