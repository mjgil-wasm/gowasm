use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_mixed_gc_stress_language_shapes() {
    let source = r#"
package main
import "fmt"

type Node struct {
    value int
    next *Node
}

type Reader interface {
    Read() int
}

type NodeReader struct {
    node *Node
    bias int
}

func (r NodeReader) Read() int {
    return r.node.value + r.bias
}

func makeClosure(base int, node *Node) func() int {
    values := map[string]int{"next": node.next.value}
    return func() int {
        return base + values["next"]
    }
}

func main() {
    first := &Node{value: 2}
    second := &Node{value: 5}
    first.next = second
    second.next = first

    ch := make(chan int, 2)
    ch <- makeClosure(first.value, first)()
    ch <- second.next.value

    var reader Reader = NodeReader{node: second, bias: <-ch}
    fmt.Println(reader.Read(), <-ch)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "12 2\n");
}

#[test]
fn compiles_and_runs_gc_stress_across_deferred_closure_unwind() {
    let source = r#"
package main
import "fmt"

type Node struct {
    value int
    next *Node
}

func main() {
    first := &Node{value: 40}
    second := &Node{value: 2}
    first.next = second

    defer func() {
        fmt.Println(first.value + first.next.value)
    }()

    scratch := []*Node{}
    for i := 0; i < 6; i++ {
        scratch = append(scratch, &Node{value: i})
    }
    _ = scratch
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_gc_allocation_threshold(1);
    vm.run_program(&program).expect("program should run");

    assert_eq!(vm.stdout(), "42\n");
    assert!(vm.gc_stats().total_collections > 0);
}
