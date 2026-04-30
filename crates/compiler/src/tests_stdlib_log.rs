use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn log_println_prints_with_newline() {
    let source = r#"
package main
import "log"

func main() {
    log.Println("hello", "world")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}

#[test]
fn log_print_uses_sprint_spacing_and_appends_newline() {
    let source = r#"
package main
import "log"

func main() {
    log.Print("hello", "world")
    log.Print("value:", 7)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "helloworld\nvalue:7\n");
}

#[test]
fn log_printf_formats_output() {
    let source = r#"
package main
import "log"

func main() {
    log.Printf("value: %d, name: %s", 42, "test")
    log.Printf("next line\n")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "value: 42, name: test\nnext line\n");
}

#[test]
fn log_prefix_and_zero_flags_control_output() {
    let source = r#"
package main
import (
    "fmt"
    "log"
)

func main() {
    log.SetFlags(0)
    log.SetPrefix("LOG: ")
    log.Print("hello", "world")
    fmt.Printf("%q %d\n", log.Prefix(), log.Flags())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "LOG: helloworld\n\"LOG: \" 0\n");
}

#[test]
fn log_set_flags_rejects_non_zero_flags() {
    let source = r#"
package main
import "log"

func main() {
    log.SetFlags(1)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("log: non-zero flags are outside the supported slice"));
}

#[test]
fn log_fatal_prints_and_exits_with_code_1() {
    let source = r#"
package main
import "log"

func main() {
    log.Fatal("something went wrong")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let result = vm.run_program(&program).expect_err("program should exit");
    assert!(matches!(
        result.root_cause(),
        gowasm_vm::VmError::ProgramExit { code: 1 }
    ));
    assert_eq!(vm.stdout(), "something went wrong\n");
}

#[test]
fn log_fatalf_formats_and_exits_with_code_1() {
    let source = r#"
package main
import "log"

func main() {
    log.Fatalf("error: %s (code %d)", "failed", 99)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let result = vm.run_program(&program).expect_err("program should exit");
    assert!(matches!(
        result.root_cause(),
        gowasm_vm::VmError::ProgramExit { code: 1 }
    ));
    assert_eq!(vm.stdout(), "error: failed (code 99)\n");
}

#[test]
fn log_fatal_does_not_execute_subsequent_statements() {
    let source = r#"
package main
import "log"

func main() {
    log.Fatal("stop")
    log.Println("unreachable")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let _ = vm.run_program(&program);
    assert_eq!(vm.stdout(), "stop\n");
}
