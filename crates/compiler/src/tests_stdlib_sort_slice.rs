use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sort_slice_ints() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{3, 1, 4, 1, 5, 9, 2, 6}
    sort.Slice(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 1 2 3 4 5 6 9]\n");
}

#[test]
fn sort_slice_strings() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []string{"banana", "apple", "cherry"}
    sort.Slice(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[apple banana cherry]\n");
}

#[test]
fn sort_slice_descending() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{3, 1, 4, 1, 5}
    sort.Slice(s, func(i int, j int) bool {
        return s[i] > s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[5 4 3 1 1]\n");
}

#[test]
fn sort_slice_empty() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{}
    sort.Slice(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[]\n");
}

#[test]
fn sort_slice_single() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{42}
    sort.Slice(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[42]\n");
}

#[test]
fn sort_slice_is_sorted_true() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{1, 2, 3, 4, 5}
    result := sort.SliceIsSorted(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn sort_slice_is_sorted_false() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{3, 1, 2}
    result := sort.SliceIsSorted(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}
