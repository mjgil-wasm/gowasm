use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn instantiated_generic_aliases_work_with_collection_builtins() {
    let source = r#"
package main
import "fmt"

type List[T any] []T
type Dict[T any] map[string]T

func main() {
    values := make(List[int], 1, 3)
    values[0] = 4
    values = append(values, 7)
    fmt.Println(len(values), cap(values), values[0], values[1])
    clear(values)
    fmt.Println(values[0], values[1])

    dict := make(Dict[int])
    dict["x"] = 9
    dict["y"] = 10
    delete(dict, "y")
    fmt.Println(len(dict), dict["x"], dict["y"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 3 4 7\n0 0\n1 9 0\n");
}

#[test]
fn instantiated_generic_aliases_work_with_channel_builtins() {
    let source = r#"
package main
import "fmt"

type Pipe[T any] chan T

func main() {
    values := make(Pipe[int], 1)
    fmt.Println(values == nil)
    close(values)
    fmt.Println(values == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\nfalse\n");
}

#[test]
fn generic_map_parameters_share_runtime_mutations() {
    let source = r#"
package main
import "fmt"

func bump[K comparable](values map[K]int, key K) {
    values[key] = values[key] + 1
}

func main() {
    values := map[string]int{"go": 1}
    alias := values
    bump(alias, "go")
    fmt.Println(values["go"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2\n");
}

#[test]
fn instantiated_generic_fields_support_stdlib_alias_methods() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"

type Box[T any] struct {
    value T
}

func main() {
    var params Box[url.Values]
    params.value = make(url.Values)
    params.value.Add("q", "go wasm")
    params.value.Add("q", "codex")

    var header Box[http.Header]
    header.value = make(http.Header)
    header.value.Set("accept", "text/plain")
    header.value.Add("Accept", "application/json")

    fmt.Println(params.value.Encode())
    fmt.Println(header.value.Get("Accept"), len(header.value.Values("Accept")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "q=go+wasm&q=codex\ntext/plain 2\n");
}
