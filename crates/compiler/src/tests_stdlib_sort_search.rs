use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sort_search_found() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    a := []int{1, 3, 6, 10, 15, 21, 28, 36, 45, 55}
    x := 6
    i := sort.Search(len(a), func(i int) bool {
        return a[i] >= x
    })
    fmt.Println(i)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn sort_search_not_found() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    a := []int{1, 3, 6, 10, 15}
    x := 20
    i := sort.Search(len(a), func(i int) bool {
        return a[i] >= x
    })
    fmt.Println(i)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn sort_search_first_element() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    a := []int{1, 3, 6, 10, 15}
    x := 1
    i := sort.Search(len(a), func(i int) bool {
        return a[i] >= x
    })
    fmt.Println(i)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn sort_search_empty() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    i := sort.Search(0, func(i int) bool {
        return true
    })
    fmt.Println(i)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn sort_search_strings() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    a := []string{"apple", "banana", "cherry", "date", "fig"}
    target := "cherry"
    i := sort.Search(len(a), func(i int) bool {
        return a[i] >= target
    })
    fmt.Println(i, a[i])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 cherry\n");
}

#[test]
fn sort_slice_stable_ints() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{3, 1, 4, 1, 5, 9, 2, 6}
    sort.SliceStable(s, func(i int, j int) bool {
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
fn sort_slice_stable_preserves_order() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{3, 1, 2, 3, 1, 2, 3}
    sort.SliceStable(s, func(i int, j int) bool {
        return s[i] < s[j]
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 1 2 2 3 3 3]\n");
}

#[test]
fn sort_slice_stable_empty() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{}
    sort.SliceStable(s, func(i int, j int) bool {
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
fn sort_slice_stable_single() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    s := []int{42}
    sort.SliceStable(s, func(i int, j int) bool {
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
