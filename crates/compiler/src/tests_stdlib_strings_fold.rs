use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_strings_equal_fold_queries() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    fmt.Println(
        strings.EqualFold("Go", "go"),
        strings.EqualFold("Σ", "ς"),
        strings.EqualFold("ǅ", "ǆ"),
        strings.EqualFold("Straße", "straße"),
        strings.EqualFold("Go", "ga"),
        strings.EqualFold("ß", "ss"),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true true false false\n");
}
