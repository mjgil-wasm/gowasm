use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_nil_pointer_comparisons() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
}

func main() {
    var point *Point
    fmt.Println(point == nil, point != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\n");
}

#[test]
fn compiles_and_runs_address_of_and_deref_reads() {
    let source = r#"
package main
import "fmt"

func show(ptr *int) {
    fmt.Println(*ptr)
}

func main() {
    value := 7
    show(&value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_deref_assignment() {
    let source = r#"
package main
import "fmt"

func set(ptr *int) {
    *ptr = 9
}

func main() {
    value := 1
    set(&value)
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_returned_local_pointers() {
    let source = r#"
package main
import "fmt"

func makePtr() *int {
    value := 7
    return &value
}

func main() {
    ptr := makePtr()
    fmt.Println(*ptr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_returned_parameter_pointers() {
    let source = r#"
package main
import "fmt"

func makePtr(value int) *int {
    return &value
}

func main() {
    ptr := makePtr(11)
    fmt.Println(*ptr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "11\n");
}

#[test]
fn compiles_and_runs_returned_closures_over_local_pointers() {
    let source = r#"
package main
import "fmt"

func makeReader() func() int {
    value := 5
    ptr := &value
    return func() int {
        *ptr = *ptr + 1
        return *ptr
    }
}

func main() {
    run := makeReader()
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n7\n");
}

#[test]
fn compiles_and_runs_returned_closures_over_parameter_pointers() {
    let source = r#"
package main
import "fmt"

func makeReader(value int) func() int {
    ptr := &value
    return func() int {
        *ptr = *ptr + 1
        return *ptr
    }
}

func main() {
    run := makeReader(2)
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n4\n");
}

#[test]
fn compiles_and_runs_returned_closures_over_field_pointers() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
}

func makeReader() func() int {
    point := Point{x: 4}
    ptr := &point.x
    return func() int {
        *ptr = *ptr + 2
        return *ptr
    }
}

func main() {
    run := makeReader()
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n8\n");
}

#[test]
fn compiles_and_runs_returned_closures_over_index_pointers() {
    let source = r#"
package main
import "fmt"

func makeReader() func() int {
    values := []int{1, 2}
    ptr := &values[1]
    return func() int {
        *ptr = *ptr + 3
        return *ptr
    }
}

func main() {
    run := makeReader()
    fmt.Println(run())
    fmt.Println(run())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5\n8\n");
}

#[test]
fn compiles_and_runs_field_pointer_reads_and_writes() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
}

func main() {
    point := Point{x: 1}
    ptr := &point.x
    *ptr = 9
    fmt.Println(point.x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_projected_field_pointer_reads_and_writes() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
}

func main() {
    point := Point{x: 1}
    pointPtr := &point
    fieldPtr := &(*pointPtr).x
    *fieldPtr = 9
    fmt.Println(point.x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_index_pointer_reads_and_writes() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2}
    ptr := &values[1]
    *ptr = 9
    fmt.Println(values[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_projected_index_pointer_reads_and_writes() {
    let source = r#"
package main
import "fmt"

func main() {
    values := []int{1, 2}
    valuesPtr := &values
    itemPtr := &(*valuesPtr)[1]
    *itemPtr = 9
    fmt.Println(values[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_implicit_pointer_selector_reads_and_writes() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    point := Point{x: 1, y: 2}
    ptr := &point
    fmt.Println(ptr.x, ptr.y)
    ptr.x = 9
    fmt.Println(point.x)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n9\n");
}

#[test]
fn compiles_and_runs_implicit_pointer_value_method_calls() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func (point Point) sum() int {
    return point.x + point.y
}

func main() {
    point := Point{x: 2, y: 5}
    ptr := &point
    total := ptr.sum()
    fmt.Println(total, ptr.sum())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 7\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_pointer_values() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    counter := Counter{value: 1}
    ptr := &counter
    fmt.Println(ptr.inc(), counter.value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 2\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_addressable_values() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    counter := Counter{value: 1}
    fmt.Println(counter.inc(), counter.value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 2\n");
}

#[test]
fn compiles_and_runs_deref_selector_and_index_assignments() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
}

func main() {
    point := Point{x: 1}
    pointPtr := &point
    (*pointPtr).x = 9

    values := []int{1, 2}
    valuesPtr := &values
    (*valuesPtr)[1] = 7

    fmt.Println(point.x, values[1])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 7\n");
}

#[test]
fn compiles_and_runs_chained_selector_reads_through_embedded_pointer_fields() {
    let source = r#"
package main
import "fmt"

type Tagged struct {
    Value string
    Count int
}

type Payload struct {
    *Tagged
}

func main() {
    payload := Payload{Tagged: &Tagged{Value: "go", Count: 7}}
    fmt.Println(payload.Tagged != nil, payload.Tagged.Value, payload.Tagged.Count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true go 7\n");
}

#[test]
fn direct_fields_beat_promoted_embedded_fields_with_the_same_name() {
    let source = r#"
package main
import "fmt"

type Tagged struct {
    Name string
}

type Plain struct {
    Name string
}

type Payload struct {
    Name string
    *Tagged
    Plain
}

func main() {
    payload := Payload{
        Name: "outer",
        Tagged: &Tagged{Name: "tagged"},
        Plain: Plain{Name: "plain"},
    }
    fmt.Println(payload.Name, payload.Tagged.Name, payload.Plain.Name)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "outer tagged plain\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_addressable_selector_receivers() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

type Holder struct {
    counter Counter
}

func main() {
    holder := Holder{counter: Counter{value: 3}}
    fmt.Println(holder.counter.inc(), holder.counter.value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "4 4\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_addressable_index_receivers() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 2
    return counter.value
}

func main() {
    counters := []Counter{Counter{value: 1}, Counter{value: 5}}
    fmt.Println(counters[1].inc(), counters[1].value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 7\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_global_values() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 4
    return counter.value
}

var global = Counter{value: 2}
var items = []Counter{Counter{value: 1}, Counter{value: 6}}

func main() {
    fmt.Println(global.inc(), global.value)
    fmt.Println(items[0].inc(), items[0].value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6 6\n5 5\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_methods_on_closure_captured_index_receivers() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    counters := []Counter{Counter{value: 2}}
    bump := func() int {
        return counters[0].inc()
    }
    fmt.Println(bump(), bump(), counters[0].value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 4 4\n");
}

#[test]
fn rejects_address_of_map_index() {
    let source = r#"
package main

func main() {
    values := map[string]int{"x": 1}
    _ = &values["x"]
}
"#;

    let error = compile_source(source).expect_err("map index address-of should fail");
    assert!(
        error
            .to_string()
            .contains("cannot take the address of a map or string index"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_address_of_string_index() {
    let source = r#"
package main

func main() {
    text := "go"
    _ = &text[0]
}
"#;

    let error = compile_source(source).expect_err("string index address-of should fail");
    assert!(
        error
            .to_string()
            .contains("cannot take the address of a map or string index"),
        "unexpected error: {error}"
    );
}

#[test]
fn compiles_and_runs_address_of_index_on_temporary_slice_result() {
    let source = r#"
package main
import "fmt"

func values() []int {
    return []int{1, 2}
}

func main() {
    ptr := &values()[0]
    *ptr = 9
    fmt.Println(*ptr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_method_calls_on_temporary_index_receivers() {
    let source = r#"
package main
import "fmt"

type Counter struct {
    value int
}

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func values() []Counter {
    return []Counter{Counter{value: 1}}
}

func main() {
    fmt.Println(values()[0].inc())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}
