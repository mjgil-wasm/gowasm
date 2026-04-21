use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_range_loops_over_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{4, 5, 6}
    for index, value := range values {
        fmt.Println(index, value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 4\n1 5\n2 6\n");
}

#[test]
fn compiles_and_runs_range_loops_over_maps() {
    let source = r#"
package main
import "fmt"

func main() {
    values := map[string]int{"go": 1, "wasm": 2}
    for key, value := range values {
        fmt.Println(key, value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "go 1\nwasm 2\n");
}

#[test]
fn compiles_and_runs_range_loops_with_break_and_continue() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2, 3, 4}
    for index := range values {
        if index == 1 {
            continue
        }
        fmt.Println(index)
        if index == 2 {
            break
        }
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n2\n");
}

#[test]
fn compiles_and_runs_range_loops_over_strings() {
    let source = r#"
package main
import "fmt"

func main() {
    for index, value := range "hé" {
        fmt.Println(index, value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 104\n1 233\n");
}

#[test]
fn compiles_and_runs_range_loops_report_byte_indexes_for_multibyte_strings() {
    let source = r#"
package main
import "fmt"

func main() {
    for index, value := range "世a界" {
        fmt.Println(index, value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 19990\n3 97\n4 30028\n");
}

#[test]
fn compiles_and_runs_bindingless_range_loops() {
    let source = r#"
package main
import "fmt"

func main() {
    count := 0
    for range []int{4, 5, 6} {
        count++
    }
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn compiles_and_runs_assignment_form_range_loops_over_slices() {
    let source = r#"
package main
import "fmt"

func main() {
    var index int
    var value int
    for index, value = range []int{4, 5, 6} {
        fmt.Println(index, value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 4\n1 5\n2 6\n");
}

#[test]
fn compiles_and_runs_range_loops_over_channels() {
    let source = r#"
package main
import "fmt"

func send(values chan<- int) {
    values <- 4
    values <- 5
    close(values)
}

func main() {
    values := make(chan int, 2)
    go send(values)
    for value := range values {
        fmt.Println(value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4\n5\n");
}

#[test]
fn compiles_and_runs_assignment_form_range_loops_over_channels() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    var value int
    for value = range values {
        fmt.Println(value)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n8\n");
}

#[test]
fn rejects_assignment_form_range_loops_with_unknown_targets() {
    let source = r#"
package main

func main() {
    for index, value = range []int{4, 5, 6} {
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("assignment target `index` is not defined in the current function scope"));
}

#[test]
fn rejects_assignment_form_range_loops_with_type_mismatches() {
    let source = r#"
package main

func main() {
    var value string
    for value = range []int{4, 5, 6} {
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("range value of type `int` is not assignable to `string`"));
}

#[test]
fn rejects_two_binding_assignment_form_channel_range_loops() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    var index int
    var value int
    for index, value = range values {
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("channel `range` currently supports only one binding"));
}

#[test]
fn rejects_range_loops_over_non_iterable_types() {
    let source = r#"
package main

func main() {
    x := true
    for value := range x {
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("cannot range over non-iterable type `bool`"));
}

#[test]
fn compiles_and_runs_range_over_int() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := range 5 {
        fmt.Println(i)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n2\n3\n4\n");
}

#[test]
fn compiles_and_runs_range_over_int_variable() {
    let source = r#"
package main
import "fmt"

func main() {
    n := 3
    for i := range n {
        fmt.Println(i)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n2\n");
}

#[test]
fn compiles_and_runs_bindingless_range_over_int() {
    let source = r#"
package main
import "fmt"

func main() {
    count := 0
    for range 4 {
        count++
    }
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4\n");
}

#[test]
fn compiles_and_runs_range_over_int_with_break() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := range 10 {
        if i == 3 {
            break
        }
        fmt.Println(i)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n1\n2\n");
}

#[test]
fn compiles_and_runs_range_over_zero_int() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := range 0 {
        fmt.Println(i)
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "done\n");
}

#[test]
fn compiles_and_runs_range_loops_over_receive_only_channels() {
    let source = r#"
package main
import "fmt"

func drain(values <-chan int) {
    for value := range values {
        fmt.Println(value)
    }
}

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    drain(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n8\n");
}

#[test]
fn compiles_and_runs_assignment_form_range_loops_over_receive_only_channels() {
    let source = r#"
package main
import "fmt"

func drain(values <-chan int) {
    var value int
    for value = range values {
        fmt.Println(value)
    }
}

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    drain(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n8\n");
}

#[test]
fn compiles_and_runs_bindingless_range_loops_over_receive_only_channels() {
    let source = r#"
package main
import "fmt"

func drain(values <-chan int) {
    count := 0
    for range values {
        count++
    }
    fmt.Println(count)
}

func main() {
    values := make(chan int, 2)
    values <- 7
    values <- 8
    close(values)
    drain(values)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn compiles_and_runs_channel_range_loops_with_break_and_continue() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int, 4)
    values <- 1
    values <- 2
    values <- 3
    values <- 4
    close(values)
    for value := range values {
        if value == 2 {
            continue
        }
        fmt.Println(value)
        if value == 3 {
            break
        }
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n3\n");
}

#[test]
fn compiles_and_runs_closed_channel_range_loops_without_entering_body() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    for value := range values {
        fmt.Println(value)
    }
    fmt.Println("done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "done\n");
}

#[test]
fn compiles_and_runs_bindingless_closed_channel_range_loops_without_entering_body() {
    let source = r#"
package main
import "fmt"

func main() {
    values := make(chan int)
    close(values)
    count := 0
    for range values {
        count++
    }
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn rejects_two_binding_channel_range_loops() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    for index, value := range values {
    }
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("channel `range` currently supports only one binding"));
}
