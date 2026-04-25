use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn cmp_compare_ints() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Compare(1, 2))
    fmt.Println(cmp.Compare(2, 2))
    fmt.Println(cmp.Compare(3, 2))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n0\n1\n");
}

#[test]
fn cmp_compare_strings() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Compare("apple", "banana"))
    fmt.Println(cmp.Compare("hello", "hello"))
    fmt.Println(cmp.Compare("z", "a"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n0\n1\n");
}

#[test]
fn cmp_less_ints() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Less(1, 2))
    fmt.Println(cmp.Less(2, 2))
    fmt.Println(cmp.Less(3, 2))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\nfalse\n");
}

#[test]
fn cmp_less_strings() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Less("a", "b"))
    fmt.Println(cmp.Less("b", "a"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}

#[test]
fn cmp_or_ints() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Or(0, 0, 3, 4))
    fmt.Println(cmp.Or(0, 0, 0))
    fmt.Println(cmp.Or(5))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n0\n5\n");
}

#[test]
fn cmp_or_strings() {
    let source = r#"
package main
import "fmt"
import "cmp"

func main() {
    fmt.Println(cmp.Or("", "", "hello", "world"))
    fmt.Println(cmp.Or("", ""))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n\n");
}

#[test]
fn cmp_compare_with_sort_func() {
    let source = r#"
package main
import "fmt"
import "cmp"
import "slices"

func main() {
    s := []int{5, 3, 1, 4, 2}
    slices.SortFunc(s, func(a int, b int) int {
        return cmp.Compare(a, b)
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4 5]\n");
}
