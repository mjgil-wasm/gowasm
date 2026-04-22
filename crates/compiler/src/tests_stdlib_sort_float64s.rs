use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sort_float64s_are_sorted_true() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []float64{1.0, 2.5, 3.7}
    fmt.Println(sort.Float64sAreSorted(s))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn sort_float64s_are_sorted_false() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []float64{3.0, 1.0, 2.0}
    fmt.Println(sort.Float64sAreSorted(s))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn sort_search_float64s() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []float64{1.0, 2.5, 3.7, 5.0}
    fmt.Println(sort.SearchFloat64s(s, 2.5))
    fmt.Println(sort.SearchFloat64s(s, 4.0))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n3\n");
}
