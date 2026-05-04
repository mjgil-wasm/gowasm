use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_cross_type_nil_matrix() {
    let source = r#"
package main
import "fmt"

func main() {
    var slice []int
    var mapping map[string]int
    var fn func() int
    var ptr *int
    var any interface{}
    var ch chan int

    value, ok := mapping["missing"]
    fmt.Println(slice == nil, len(slice), cap(slice))
    fmt.Println(mapping == nil, len(mapping), value, ok)
    fmt.Println(fn == nil)
    fmt.Println(ptr == nil)
    fmt.Println(any == nil)
    fmt.Println(ch == nil)

    slice = append(slice, 3)
    mapping = map[string]int{"hit": 4}
    fn = func() int { return 5 }
    pointed := 6
    ptr = &pointed
    any = "set"
    ch = make(chan int, 2)

    fmt.Println(slice[0], len(slice), cap(slice))
    fmt.Println(len(mapping), mapping["hit"])
    fmt.Println(fn())
    fmt.Println(*ptr)
    fmt.Println(any)
}
    "#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true 0 0\ntrue 0 0 false\ntrue\ntrue\ntrue\ntrue\n3 1 1\n1 4\n5\n6\nset\n"
    );
}

#[test]
fn compiles_and_runs_nil_formatting_and_typed_nil_interfaces() {
    let source = r#"
package main
import "fmt"

func main() {
    var slice []int
    var mapping map[string]int
    var fn func() int
    var ptr *int
    var ch chan int
    var any interface{}
    var err error

    fmt.Println(slice, mapping, fn, ptr, any, ch, err)

    var typed *int
    any = typed
    fmt.Println(typed == nil, any == nil, any)

    slice = []int{}
    mapping = map[string]int{}
    fmt.Println(slice == nil, mapping == nil, slice, mapping)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[] map[] <nil> <nil> <nil> <nil> <nil>\ntrue false <nil>\nfalse false [] map[]\n"
    );
}
