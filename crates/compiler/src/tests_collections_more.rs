use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn slice_append_and_len() {
    let source = r#"
package main
import "fmt"

func main() {
    s := []int{1, 2}
    s = append(s, 3)
    s = append(s, 4, 5)
    fmt.Println(len(s), s[0], s[4])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 1 5\n");
}

#[test]
fn map_operations() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{"a": 1, "b": 2}
    m["c"] = 3
    fmt.Println(len(m))
    fmt.Println(m["a"], m["b"], m["c"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n1 2 3\n");
}

#[test]
fn map_delete() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{"x": 10, "y": 20}
    delete(m, "x")
    fmt.Println(len(m))
    fmt.Println(m["y"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n20\n");
}

#[test]
fn map_comma_ok_lookup() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{"key": 42}
    v1, ok1 := m["key"]
    v2, ok2 := m["missing"]
    fmt.Println(v1, ok1)
    fmt.Println(v2, ok2)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true\n0 false\n");
}

#[test]
fn slice_range_with_index_and_value() {
    let source = r#"
package main
import "fmt"

func main() {
    items := []string{"a", "b", "c"}
    for i, v := range items {
        fmt.Println(i, v)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 a\n1 b\n2 c\n");
}

#[test]
fn array_fixed_size() {
    let source = r#"
package main
import "fmt"

func main() {
    arr := [3]int{10, 20, 30}
    fmt.Println(len(arr))
    arr[1] = 25
    fmt.Println(arr[0], arr[1], arr[2])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n10 25 30\n");
}

#[test]
fn nested_slice_of_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    matrix := [][]int{
        []int{1, 2},
        []int{3, 4},
    }
    fmt.Println(matrix[0][0], matrix[0][1])
    fmt.Println(matrix[1][0], matrix[1][1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n3 4\n");
}

#[test]
fn empty_slice_and_nil_slice() {
    let source = r#"
package main
import "fmt"

func main() {
    empty := []int{}
    var nilSlice []int
    fmt.Println(len(empty))
    fmt.Println(nilSlice == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\ntrue\n");
}

#[test]
fn string_concatenation_and_len() {
    let source = r#"
package main
import "fmt"

func main() {
    s := "hello"
    s = s + " world"
    fmt.Println(s)
    fmt.Println(len(s))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n11\n");
}

#[test]
fn map_range_iteration() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{"x": 1}
    for k, v := range m {
        fmt.Println(k, v)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "x 1\n");
}
