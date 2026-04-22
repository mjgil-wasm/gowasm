use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strings_map_to_uppercase() {
    let source = r#"
package main
import "fmt"
import "strings"
import "unicode"

func main() {
    result := strings.Map(unicode.ToUpper, "hello world")
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "HELLO WORLD\n");
}

#[test]
fn strings_map_with_closure() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    result := strings.Map(func(r rune) rune {
        if r == 'a' {
            return 'A'
        }
        return r
    }, "abcabc")
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "AbcAbc\n");
}

#[test]
fn strings_map_drop_characters() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    result := strings.Map(func(r rune) rune {
        if r == ' ' {
            return -1
        }
        return r
    }, "hello world")
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "helloworld\n");
}

#[test]
fn strings_map_empty_string() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    result := strings.Map(func(r rune) rune {
        return r + 1
    }, "")
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\n");
}

#[test]
fn strings_map_rot13() {
    let source = r#"
package main
import "fmt"
import "strings"

func rot13(r rune) rune {
    if r >= 'a' && r <= 'z' {
        return 'a' + (r - 'a' + 13) % 26
    }
    if r >= 'A' && r <= 'Z' {
        return 'A' + (r - 'A' + 13) % 26
    }
    return r
}

func main() {
    result := strings.Map(rot13, "Hello World!")
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Uryyb Jbeyq!\n");
}
