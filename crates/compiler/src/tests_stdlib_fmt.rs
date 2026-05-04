use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn fmt_sprintf_string_and_int() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("hello %s, you are %d", "world", 42)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world, you are 42\n");
}

#[test]
fn fmt_sprintf_float() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("pi is %.2f", 3.14159)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "pi is 3.14\n");
}

#[test]
fn fmt_sprintf_bool_and_verb_v() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("%t %v %v", true, 99, "go")
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 99 go\n");
}

#[test]
fn fmt_sprintf_hex_and_octal() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("%x %o %b", 255, 8, 5)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ff 10 101\n");
}

#[test]
fn fmt_sprintf_width_and_padding() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("[%10d]", 42)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[        42]\n");
}

#[test]
fn fmt_sprintf_zero_pad() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("[%05d]", 42)
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[00042]\n");
}

#[test]
fn fmt_printf_writes_to_stdout() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Printf("count: %d\n", 7)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "count: 7\n");
}

#[test]
fn fmt_sprint_concatenates() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprint("hello", " ", "world")
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}

#[test]
fn fmt_sprintln_adds_newline() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintln("hello", "world")
    fmt.Printf("[%s]", s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello world\n]");
}

#[test]
fn fmt_sprintf_percent_literal() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("100%%")
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "100%\n");
}

#[test]
fn fmt_sprintf_left_align() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("[%-10s]", "go")
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[go        ]\n");
}

#[test]
fn fmt_sprintf_quoted_string() {
    let source = r#"
package main
import "fmt"

func main() {
    s := fmt.Sprintf("%q", "hello")
    fmt.Println(s)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\"hello\"\n");
}

#[test]
fn fmt_print_and_sprintf_use_error_and_string_methods() {
    let source = r#"
package main
import "fmt"

type Label string

func (l Label) String() string {
    return "label<" + string(l) + ">"
}

type Problem struct {}

func (Problem) Error() string {
    return "problem<7>"
}

func main() {
    var err error = Problem{}

    fmt.Println(Label("go"))
    fmt.Println(err)
    fmt.Println(fmt.Sprintf("%v %s %q", Label("go"), Label("go"), Label("go")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "label<go>\nproblem<7>\nlabel<go> label<go> \"label<go>\"\n"
    );
}

#[test]
fn fmt_hardens_reflection_sensitive_pointer_and_flagged_output() {
    let source = r#"
package main
import "fmt"

type Box struct {
    Name string
    Count int
}

type Labels []string
type Lookup map[string]int

func main() {
    box := Box{Name: "go", Count: 2}
    var labels Labels
    var nilLabels Labels
    var lookup Lookup
    labels = Labels([]string{"x", "y"})

    fmt.Println(&box)
    fmt.Printf("%+v\n", box)
    fmt.Printf("%+v\n", &box)
    fmt.Printf("%#v\n", box)
    fmt.Printf("%#v\n", labels)
    fmt.Printf("%#v\n", &labels)
    fmt.Printf("%#v\n", nilLabels)
    fmt.Printf("%#v\n", lookup)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "&{go 2}\n",
            "{Name:go Count:2}\n",
            "&{Name:go Count:2}\n",
            "main.Box{Name:\"go\", Count:2}\n",
            "main.Labels{\"x\", \"y\"}\n",
            "&main.Labels{\"x\", \"y\"}\n",
            "main.Labels(nil)\n",
            "main.Lookup(nil)\n",
        )
    );
}

#[test]
fn fmt_and_log_harden_recursive_alias_and_interface_values() {
    let source = r#"
package main
import (
    "fmt"
    "log"
)

type Node struct {
    Name string
    Next *Node
}

type Labels []string

func main() {
    node := &Node{Name: "loop"}
    node.Next = node

    var wrapped interface{} = node
    var labels Labels
    labels = Labels([]string{"x", "y"})
    var aliasWrapped interface{} = labels

    fmt.Println(wrapped)
    fmt.Printf("%+v\n", wrapped)
    fmt.Printf("%#v\n", wrapped)
    fmt.Printf("%#v\n", aliasWrapped)
    log.Println(wrapped)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "&{loop &{...}}\n",
            "&{Name:loop Next:&{...}}\n",
            "&main.Node{Name:\"loop\", Next:&main.Node{...}}\n",
            "main.Labels{\"x\", \"y\"}\n",
            "&{loop &{...}}\n",
        )
    );
}

#[test]
fn fmt_supports_type_and_pointer_verbs_for_supported_value_model() {
    let source = r#"
package main
import "fmt"

type Box struct {
    Name string
}

type Labels []int

func noop() {}

func main() {
    var nilBox *Box
    box := &Box{Name: "go"}
    var any interface{} = box
    var none interface{}
    var labels Labels
    var nilLabels Labels
    m := map[string]int{"a": 1}
    var nilMap map[string]int
    ch := make(chan int, 1)
    var nilCh chan int
    f := noop
    labels = []int{1, 2}

    fmt.Println(fmt.Sprintf("%T", nilBox))
    fmt.Println(fmt.Sprintf("%T", box))
    fmt.Println(fmt.Sprintf("%T", labels))
    fmt.Println(fmt.Sprintf("%T", any))
    fmt.Println(fmt.Sprintf("%T", none))
    fmt.Println(fmt.Sprintf("%p", nilBox))
    fmt.Println(fmt.Sprintf("%p", box) == fmt.Sprintf("%p", box))
    fmt.Println(fmt.Sprintf("%p", box) != fmt.Sprintf("%p", &Box{Name: "go"}))
    fmt.Println(fmt.Sprintf("%p", nilLabels))
    fmt.Println(fmt.Sprintf("%p", labels) == fmt.Sprintf("%p", labels))
    fmt.Println(fmt.Sprintf("%p", nilMap))
    fmt.Println(fmt.Sprintf("%p", m) == fmt.Sprintf("%p", m))
    fmt.Println(fmt.Sprintf("%p", nilCh))
    fmt.Println(fmt.Sprintf("%p", ch) == fmt.Sprintf("%p", ch))
    fmt.Println(fmt.Sprintf("%p", f) == fmt.Sprintf("%p", f))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "*main.Box\n",
            "*main.Box\n",
            "main.Labels\n",
            "*main.Box\n",
            "<nil>\n",
            "0x0\n",
            "true\n",
            "true\n",
            "0x0\n",
            "true\n",
            "0x0\n",
            "true\n",
            "0x0\n",
            "true\n",
            "true\n",
        )
    );
}

#[test]
fn fmt_hardens_alternate_integer_padding_and_string_precision() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println(fmt.Sprintf("%x", -42))
    fmt.Println(fmt.Sprintf("%#x", 42))
    fmt.Println(fmt.Sprintf("%#08x", 42))
    fmt.Println(fmt.Sprintf("%#o", 8))
    fmt.Println(fmt.Sprintf("%O", 8))
    fmt.Println(fmt.Sprintf("%b", -5))
    fmt.Println(fmt.Sprintf("%#08x", -42))
    fmt.Println(fmt.Sprintf("[%#08x][% 6d][%+06d][%.3s][%8.3s]", 42, 42, 42, "gopher", "gopher"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "-2a\n",
            "0x2a\n",
            "0x0000002a\n",
            "010\n",
            "0o10\n",
            "-101\n",
            "-0x000002a\n",
            "[0x0000002a][    42][+00042][gop][     gop]\n",
        )
    );
}


#[test]
fn fmt_type_verb_renders_stdlib_builtin_types() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"
import "sync"
import "time"

func main() {
    var header http.Header
    var values url.Values
    var mu sync.Mutex
    var wg sync.WaitGroup
    var once sync.Once
    var rw sync.RWMutex
    var t time.Time
    var req http.Request
    var resp http.Response

    fmt.Println(fmt.Sprintf("%T", header))
    fmt.Println(fmt.Sprintf("%T", values))
    fmt.Println(fmt.Sprintf("%T", mu))
    fmt.Println(fmt.Sprintf("%T", wg))
    fmt.Println(fmt.Sprintf("%T", once))
    fmt.Println(fmt.Sprintf("%T", rw))
    fmt.Println(fmt.Sprintf("%T", t))
    fmt.Println(fmt.Sprintf("%T", req))
    fmt.Println(fmt.Sprintf("%T", resp))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "http.Header\n",
            "url.Values\n",
            "sync.Mutex\n",
            "sync.WaitGroup\n",
            "sync.Once\n",
            "sync.RWMutex\n",
            "time.Time\n",
            "http.Request\n",
            "http.Response\n",
        )
    );
}
