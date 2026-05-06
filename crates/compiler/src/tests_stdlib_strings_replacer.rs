use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn strings_new_replacer_supports_direct_method_calls() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(strings.NewReplacer("a", "b", "b", "c").Replace("abba"))
    fmt.Println(strings.NewReplacer("aaa", "3", "aa", "2", "a", "1").Replace("aaaa"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "bccb\n31\n");
}

#[test]
fn strings_new_replacer_handles_empty_old_strings_and_argument_order() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    r := strings.NewReplacer("", "X", "o", "O")
    fmt.Println(r.Replace("oiio"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "XOXiXiXOX\n");
}

#[test]
fn strings_new_replacer_panics_on_odd_argument_count() {
    let source = r#"
package main
import "strings"

func main() {
    strings.NewReplacer("a", "b", "c")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("strings.NewReplacer: odd argument count"));
}
