use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_subslice_append_within_capacity() {
    let source = r#"
package main
import "fmt"

func main() {
    base := make([]int, 2, 4)
    base[0] = 1
    base[1] = 2
    sub := base[:1]
    sub = append(sub, 9)
    fmt.Println(base, sub, len(sub), cap(sub))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 9] [1 9] 2 4\n");
}

#[test]
fn compiles_and_runs_copy_on_overlapping_subslices() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3, 4}
    fmt.Println(copy(values[1:], values), values)
    fmt.Println(copy(values, values[2:]), values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 [1 1 2 3]\n2 [2 3 2 3]\n");
}

#[test]
fn compiles_and_runs_array_assignment_by_value() {
    let source = r#"
package main
import "fmt"

func main() {
    original := [3]int{1, 2, 3}
    copied := original
    copied[0] = 9
    fmt.Println(original, copied)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3] [9 2 3]\n");
}

#[test]
fn compiles_and_runs_pointer_to_slice_and_array_elements() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3}
    ptr := &values[1]
    *ptr = 7

    array := [2]int{4, 5}
    arrayPtr := &array[0]
    *arrayPtr = 9

    fmt.Println(values, array)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 7 3] [9 5]\n");
}
