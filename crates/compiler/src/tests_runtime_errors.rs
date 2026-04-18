use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn runtime_division_by_zero_reports_concrete_operands() {
    let source = r#"
package main

func main() {
    value := 0
    _ = 1 / value
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("program should fail at runtime");
    let text = error.to_string();
    assert!(text.contains("division by zero in function `main`"));
    assert!(text.contains("left int value `1`"));
    assert!(text.contains("right int value `0`"));
}

#[test]
fn runtime_send_on_closed_channel_reports_value_context() {
    let source = r#"
package main

func main() {
    values := make(chan int)
    close(values)
    values <- 7
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("program should fail at runtime");
    let text = error.to_string();
    assert!(text.contains("send on closed channel in function `main`"));
    assert!(text.contains("value int value `7`"));
    assert!(text.contains("target channel value `<chan>`"));
}
