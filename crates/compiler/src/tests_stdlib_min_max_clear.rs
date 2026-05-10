use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn min_two_ints() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(min(3, 1))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn max_two_ints() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(max(3, 1))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn min_multiple_ints() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(min(5, 3, 1, 4, 2))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn max_multiple_ints() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(max(5, 3, 1, 4, 2))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n");
}

#[test]
fn min_strings() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(min("banana", "apple", "cherry"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "apple\n");
}

#[test]
fn max_strings() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(max("banana", "apple", "cherry"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "cherry\n");
}

#[test]
fn min_single_value() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(min(42))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn min_max_in_expression() {
    let source = r#"
package main
import "fmt"

func main() {
    x := min(10, 20) + max(3, 5)
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15\n");
}

#[test]
fn clear_map() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{"a": 1, "b": 2, "c": 3}
    fmt.Println(len(m))
    clear(m)
    fmt.Println(len(m))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n0\n");
}

#[test]
fn clear_slice() {
    let source = r#"
package main
import "fmt"

func main() {
    s := []int{1, 2, 3}
    clear(s)
    fmt.Println(s)
    fmt.Println(len(s))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[0 0 0]\n3\n");
}

#[test]
fn clear_empty_map() {
    let source = r#"
package main
import "fmt"

func main() {
    m := map[string]int{}
    clear(m)
    fmt.Println(len(m))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn clear_map_aliases_share_mutations() {
    let source = r#"
package main
import "fmt"

func wipe(values map[string]int) {
    clear(values)
}

func main() {
    values := map[string]int{"a": 1, "b": 2}
    alias := values
    wipe(alias)
    fmt.Println(len(values), values == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 false\n");
}
