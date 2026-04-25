use super::compile_source;
use gowasm_vm::Vm;

fn run(source: &str) -> String {
    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

#[test]
fn collections_assign_between_named_and_unnamed_forms() {
    let source = r#"
package main
import "fmt"

type Numbers []int
type Lookup map[string]int

func takeSlice(values []int) int { return len(values) }
func takeMap(values map[string]int) int { return values["x"] }

func main() {
    plainSource := []int{1, 2, 3}
    var namedSlice Numbers = plainSource
    var plainSlice []int = namedSlice

    plainMapSource := map[string]int{"x": 4}
    var namedMap Lookup = plainMapSource
    var plainMap map[string]int = namedMap

    fmt.Println(len(plainSlice), plainMap["x"], takeSlice(namedSlice), takeMap(namedMap))
}
"#;

    assert_eq!(run(source), "3 4 3 4\n");
}

#[test]
fn named_basic_types_do_not_assign_without_conversion() {
    let source = r#"
package main

type MyInt int
type YourInt int

func main() {
    var left MyInt = MyInt(1)
    var raw int = left
    var other YourInt = left
    _, _ = raw, other
}
"#;

    compile_source(source).expect_err("named basic types should require conversion");
}

#[test]
fn predeclared_rune_and_byte_follow_int_alias_assignability() {
    let source = r#"
package main
import "fmt"

func takeRune(value rune) rune { return value }

func main() {
    var codepoint rune = 65
    var raw int = codepoint
    var again rune = raw
    var octet byte = raw
    fmt.Println(raw, again, octet, takeRune(raw))
}
"#;

    assert_eq!(run(source), "65 65 65 65\n");
}

#[test]
fn function_shapes_assign_between_named_and_unnamed_forms() {
    let source = r#"
package main
import "fmt"

type Formatter func(int) string

func render(v int) string { return fmt.Sprintf("%d", v) }
func use(fn func(int) string) string { return fn(7) }

func main() {
    var named Formatter = render
    var plain func(int) string = named
    fmt.Println(use(named), plain(8))
}
"#;

    assert_eq!(run(source), "7 8\n");
}

#[test]
fn pointer_and_channel_shapes_assign_when_one_side_is_unnamed() {
    let source = r#"
package main
import "fmt"

type Counter int
type CounterPtr *Counter
type Sink chan<- int

func main() {
    value := Counter(3)
    var namedPtr CounterPtr = &value
    var plainPtr *Counter = namedPtr

    ch := make(chan int, 1)
    var sink Sink = ch
    var plainSink chan<- int = sink
    plainSink <- 4

    fmt.Println(*plainPtr, <-ch)
}
"#;

    assert_eq!(run(source), "3 4\n");
}

#[test]
fn named_nilable_types_accept_nil() {
    let source = r#"
package main
import "fmt"

type Numbers []int
type Lookup map[string]int
type Counter int
type CounterPtr *Counter
type Stream chan int

func main() {
    var values Numbers = nil
    var mapping Lookup = nil
    var ptr CounterPtr = nil
    var stream Stream = nil

    fmt.Println(values == nil, mapping == nil, ptr == nil, stream == nil)
}
"#;

    assert_eq!(run(source), "true true true true\n");
}

#[test]
fn assignments_arguments_returns_and_range_use_the_same_rules() {
    let source = r#"
package main
import "fmt"

type Label string
type Labels []Label

func last(values []Label) Label {
    var current Label
    for _, current = range values {
    }
    return current
}

func main() {
    var values Labels = []Label{"a", "b"}
    fmt.Println(last(values))
}
"#;

    assert_eq!(run(source), "b\n");
}
