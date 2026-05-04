use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn slices_contains_found() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3, 4, 5}
    fmt.Println(slices.Contains(s, 3))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn slices_contains_not_found() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3}
    fmt.Println(slices.Contains(s, 9))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn slices_contains_strings() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []string{"apple", "banana", "cherry"}
    fmt.Println(slices.Contains(s, "banana"))
    fmt.Println(slices.Contains(s, "grape"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}

#[test]
fn slices_contains_func() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3, 4, 5}
    result := slices.ContainsFunc(s, func(v int) bool {
        return v > 4
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
fn slices_contains_func_not_found() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3}
    result := slices.ContainsFunc(s, func(v int) bool {
        return v > 10
    })
    fmt.Println(result)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn slices_index_found() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []string{"a", "b", "c", "d"}
    fmt.Println(slices.Index(s, "c"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn slices_index_not_found() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{10, 20, 30}
    fmt.Println(slices.Index(s, 99))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}

#[test]
fn slices_index_func() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3, 4, 5}
    idx := slices.IndexFunc(s, func(v int) bool {
        return v > 3
    })
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn slices_sort_func_ints() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{5, 3, 1, 4, 2}
    slices.SortFunc(s, func(a int, b int) int {
        return a - b
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4 5]\n");
}

#[test]
fn slices_sort_func_descending() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3, 4, 5}
    slices.SortFunc(s, func(a int, b int) int {
        return b - a
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[5 4 3 2 1]\n");
}

#[test]
fn slices_sort_stable_func() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{3, 1, 2, 3, 1, 2}
    slices.SortStableFunc(s, func(a int, b int) int {
        return a - b
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 1 2 2 3 3]\n");
}

#[test]
fn slices_reverse() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 2, 3, 4, 5}
    slices.Reverse(s)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[5 4 3 2 1]\n");
}

#[test]
fn slices_compact() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{1, 1, 2, 2, 2, 3, 3, 4}
    s = slices.Compact(s)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4]\n");
}

#[test]
fn slices_compact_func() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []string{"hello", "HELLO", "world", "WORLD", "!"}
    s = slices.CompactFunc(s, func(a string, b string) bool {
        return len(a) == len(b)
    })
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello !]\n");
}

#[test]
fn slices_equal_true() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    a := []int{1, 2, 3}
    b := []int{1, 2, 3}
    fmt.Println(slices.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn slices_equal_false() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    a := []int{1, 2, 3}
    b := []int{1, 2, 4}
    fmt.Println(slices.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn slices_equal_different_lengths() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    a := []int{1, 2}
    b := []int{1, 2, 3}
    fmt.Println(slices.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn slices_reverse_empty() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{}
    slices.Reverse(s)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[]\n");
}

#[test]
fn slices_compact_empty() {
    let source = r#"
package main
import "fmt"
import "slices"

func main() {
    s := []int{}
    s = slices.Compact(s)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[]\n");
}
