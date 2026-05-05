use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strings_index_func_found() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    idx := strings.IndexFunc("Hello, World!", unicode.IsSpace)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn strings_index_func_not_found() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    idx := strings.IndexFunc("hello", unicode.IsUpper)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}

#[test]
fn strings_index_func_closure() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    idx := strings.IndexFunc("abc123", func(r int) bool {
        return r >= 48 && r <= 57
    })
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn strings_last_index_func_found() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    idx := strings.LastIndexFunc("Hello World", unicode.IsUpper)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn strings_last_index_func_not_found() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    idx := strings.LastIndexFunc("hello", unicode.IsUpper)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}

#[test]
fn strings_trim_func_spaces() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.TrimFunc("  hello  ", unicode.IsSpace)
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn strings_trim_func_digits() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.TrimFunc("123abc456", unicode.IsDigit)
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "abc\n");
}

#[test]
fn strings_trim_left_func() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.TrimLeftFunc("  hello  ", unicode.IsSpace)
    fmt.Printf("[%s]\n", result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello  ]\n");
}

#[test]
fn strings_trim_right_func() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.TrimRightFunc("  hello  ", unicode.IsSpace)
    fmt.Printf("[%s]\n", result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[  hello]\n");
}

#[test]
fn strings_fields_func_spaces() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.FieldsFunc("  foo  bar  baz  ", unicode.IsSpace)
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[foo bar baz]\n");
}

#[test]
fn strings_fields_func_custom_separator() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    result := strings.FieldsFunc("a,b,,c", func(r int) bool {
        return r == 44
    })
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[a b c]\n");
}

#[test]
fn strings_fields_func_empty() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.FieldsFunc("", unicode.IsSpace)
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[]\n");
}

#[test]
fn strings_trim_func_empty() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.TrimFunc("", unicode.IsSpace)
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\n");
}

#[test]
fn strings_index_func_empty() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    idx := strings.IndexFunc("", unicode.IsSpace)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}
