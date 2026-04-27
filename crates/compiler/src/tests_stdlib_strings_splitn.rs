use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_strings_split_n_queries() {
    let source = r#"
package main
import "fmt"
import "strings"

func main() {
    zero := strings.SplitN("go,wasm,zig", ",", 0)
    fmt.Println(
        strings.SplitN("go,wasm,zig", ",", 2),
        strings.SplitN("go,wasm,zig", ",", -1),
        zero == nil,
        strings.SplitN("abc", "", 2),
        strings.SplitN("abc", "", 1),
    )
    fmt.Println(zero == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[go wasm,zig] [go wasm zig] true [a bc] [abc]\ntrue\n"
    );
}
