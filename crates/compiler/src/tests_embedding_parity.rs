use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_nested_promoted_pointer_methods_through_embedded_chains() {
    let source = r#"
package main
import "fmt"

type Named interface { Name() string }

type Inner struct {}

func (*Inner) Name() string { return "inner" }

type Middle struct { *Inner }
type Outer struct { Middle }

func show(n Named) string { return n.Name() }

func main() {
    value := Outer{Middle: Middle{Inner: &Inner{}}}
    fmt.Println(show(&value), value.Name(), (&value).Name())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner inner inner\n");
}

#[test]
fn compiles_and_runs_pointer_promoted_methods_from_value_embedded_fields() {
    let source = r#"
package main
import "fmt"

type Named interface { Name() string }

type Inner struct {}

func (*Inner) Name() string { return "inner" }

type Outer struct { Inner }

func show(n Named) string { return n.Name() }

func main() {
    value := Outer{}
    fmt.Println(show(&value), value.Name(), (&value).Name())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner inner inner\n");
}

#[test]
fn rejects_value_receivers_that_only_gain_pointer_methods_via_embedded_values() {
    let source = r#"
package main

type Named interface { Name() string }

type Inner struct {}

func (*Inner) Name() string { return "inner" }

type Outer struct { Inner }

func main() {
    var named Named = Outer{}
    _ = named
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("type `Outer` does not satisfy interface `Named`"));
}
