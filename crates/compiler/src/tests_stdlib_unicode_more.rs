use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_unicode_letter_category_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsLetter(65),
        unicode.IsLetter(688),
        unicode.IsLetter(837),
        unicode.IsLetter(8544),
        unicode.IsMark(837),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false true\n");
}

#[test]
fn compiles_and_runs_unicode_upper_and_lower_category_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsUpper(923),
        unicode.IsLower(955),
        unicode.IsUpper(8544),
        unicode.IsLower(837),
        unicode.IsNumber(8544),
        unicode.IsMark(837),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false true true\n");
}

#[test]
fn compiles_and_runs_unicode_space_and_control_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsSpace(160),
        unicode.IsSpace(133),
        unicode.IsSpace(5760),
        unicode.IsSpace(8203),
        unicode.IsControl(133),
        unicode.IsControl(160),
        unicode.IsControl(8203),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false true false false\n");
}

#[test]
fn compiles_and_runs_unicode_non_latin_white_space_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsSpace(8232),
        unicode.IsSpace(8233),
        unicode.IsSpace(8203),
        unicode.IsSpace(8288),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false\n");
}

#[test]
fn compiles_and_runs_unicode_number_category_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsNumber(189),
        unicode.IsNumber(8544),
        unicode.IsNumber(12295),
        unicode.IsNumber(65),
        unicode.IsLetter(12295),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false false\n");
}
