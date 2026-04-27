use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_if_else_branches() {
    let source = r#"
package main
import "fmt"

func main() {
    if 2 + 3 == 5 {
        fmt.Println("then")
    } else {
        fmt.Println("else")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "then\n");
}

#[test]
fn compiles_and_runs_else_if_chains() {
    let source = r#"
package main
import "fmt"

func main() {
    if false {
        fmt.Println("first")
    } else if true {
        fmt.Println("second")
    } else {
        fmt.Println("third")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "second\n");
}

#[test]
fn allows_block_scoped_shadowing_inside_if_branches() {
    let source = r#"
package main
import "fmt"

func main() {
    value := "outer"
    if true {
        value := "inner"
        fmt.Println(value)
    }
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner\nouter\n");
}

#[test]
fn compiles_and_runs_if_with_init_statement() {
    let source = r#"
package main
import "fmt"

func main() {
    if x := 10; x > 5 {
        fmt.Println("big", x)
    } else {
        fmt.Println("small", x)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "big 10\n");
}

#[test]
fn compiles_and_runs_if_init_with_function_call() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    if n, err := strconv.Atoi("42"); err == nil {
        fmt.Println(n)
    } else {
        fmt.Println("error")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42\n");
}

#[test]
fn compiles_and_runs_if_init_scoping() {
    let source = r#"
package main
import "fmt"

func main() {
    x := "outer"
    if x := "inner"; x == "inner" {
        fmt.Println(x)
    }
    fmt.Println(x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner\nouter\n");
}

#[test]
fn compiles_and_runs_if_init_else_if_chain() {
    let source = r#"
package main
import "fmt"

func main() {
    if x := 3; x > 10 {
        fmt.Println("big")
    } else if x > 2 {
        fmt.Println("medium", x)
    } else {
        fmt.Println("small")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "medium 3\n");
}

#[test]
fn compiles_and_runs_condition_for_loops() {
    let source = r#"
package main
import "fmt"

func main() {
    running := true
    for running {
        fmt.Println("tick")
        running = false
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "tick\n");
}

#[test]
fn compiles_and_runs_classic_for_clauses() {
    let source = r#"
package main
import "fmt"

func main() {
    sum := 0
    for i := 0; i < 4; i++ {
        sum = sum + i
    }
    fmt.Println(sum)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn compiles_and_runs_infinite_for_loops_with_return_exit() {
    let source = r#"
package main
import "fmt"

func main() {
    for {
        fmt.Println("tick")
        return
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "tick\n");
}

#[test]
fn continue_in_classic_for_loops_runs_the_post_clause() {
    let source = r#"
package main
import "fmt"

func main() {
    for i := 0; i < 3; i++ {
        if i == 1 {
            continue
        }
        fmt.Println(i)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n2\n");
}

#[test]
fn compiles_and_runs_break_and_continue_in_loops() {
    let source = r#"
package main
import "fmt"

func main() {
    count := 0
    for true {
        count = count + 1
        if count == 1 {
            continue
        }
        fmt.Println(count)
        break
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn rejects_break_outside_loops() {
    let error = compile_source(
        r#"
package main

func main() {
    break
}
"#,
    )
    .expect_err("break outside loops should fail");

    assert!(error
        .to_string()
        .contains("`break` can only be used inside a `for` loop or `switch`"));
}

#[test]
fn rejects_continue_outside_loops() {
    let error = compile_source(
        r#"
package main

func main() {
    continue
}
"#,
    )
    .expect_err("continue outside loops should fail");

    assert!(error
        .to_string()
        .contains("`continue` can only be used inside a `for` loop"));
}
