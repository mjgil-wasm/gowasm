use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn maps_keys() {
    let source = r#"
package main
import "fmt"
import "maps"
import "slices"

func main() {
    m := map[string]int{"a": 1, "b": 2, "c": 3}
    keys := maps.Keys(m)
    slices.SortFunc(keys, func(a string, b string) int {
        if a < b {
            return -1
        }
        if a > b {
            return 1
        }
        return 0
    })
    fmt.Println(keys)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[a b c]\n");
}

#[test]
fn maps_values() {
    let source = r#"
package main
import "fmt"
import "maps"
import "slices"

func main() {
    m := map[string]int{"x": 10, "y": 20, "z": 30}
    vals := maps.Values(m)
    slices.SortFunc(vals, func(a int, b int) int {
        return a - b
    })
    fmt.Println(vals)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[10 20 30]\n");
}

#[test]
fn maps_equal_true() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1, "y": 2}
    b := map[string]int{"x": 1, "y": 2}
    fmt.Println(maps.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn maps_equal_false_different_values() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1, "y": 2}
    b := map[string]int{"x": 1, "y": 3}
    fmt.Println(maps.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn maps_equal_false_different_keys() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1, "y": 2}
    b := map[string]int{"x": 1, "z": 2}
    fmt.Println(maps.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn maps_equal_false_different_lengths() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1}
    b := map[string]int{"x": 1, "y": 2}
    fmt.Println(maps.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn maps_equal_func_custom_comparison() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1, "y": 2}
    b := map[string]int{"x": 10, "y": 20}
    result := maps.EqualFunc(a, b, func(v1 int, v2 int) bool {
        return v2 == v1 * 10
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
fn maps_equal_func_false() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{"x": 1, "y": 2}
    b := map[string]int{"x": 10, "y": 30}
    result := maps.EqualFunc(a, b, func(v1 int, v2 int) bool {
        return v2 == v1 * 10
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
fn maps_clone() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    m := map[string]int{"a": 1, "b": 2}
    c := maps.Clone(m)
    m["a"] = 99
    fmt.Println(c["a"])
    fmt.Println(c["b"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n");
}

#[test]
fn maps_copy() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    dst := map[string]int{"a": 1, "b": 2}
    src := map[string]int{"b": 20, "c": 30}
    maps.Copy(dst, src)
    fmt.Println(dst["a"])
    fmt.Println(dst["b"])
    fmt.Println(dst["c"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n20\n30\n");
}

#[test]
fn maps_delete_func() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    m := map[string]int{"a": 1, "b": 2, "c": 3, "d": 4}
    maps.DeleteFunc(m, func(k string, v int) bool {
        return v > 2
    })
    fmt.Println(m["a"])
    fmt.Println(m["b"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n2\n");
}

#[test]
fn maps_keys_empty() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    m := map[string]int{}
    keys := maps.Keys(m)
    fmt.Println(len(keys))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn maps_equal_empty() {
    let source = r#"
package main
import "fmt"
import "maps"

func main() {
    a := map[string]int{}
    b := map[string]int{}
    fmt.Println(maps.Equal(a, b))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}
