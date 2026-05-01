use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn json_unmarshal_resets_named_alias_slice_pointer_elements() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Name string
    Count int
}

type MetaList []*Meta

type Payload struct {
    Items MetaList
}

func main() {
    payload := Payload{
        Items: []*Meta{&Meta{Name: "stale", Count: 9}},
    }
    err := json.Unmarshal([]byte(`{"Items":[{"Name":"Ada"}]}`), &payload)
    fmt.Println(len(payload.Items), payload.Items[0] != nil, payload.Items[0].Name, payload.Items[0].Count, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 true Ada 9 <nil>\n");
}

#[test]
fn json_unmarshal_resets_generic_wrapper_pointer_elements() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Name string
    Count int
}

type Box[T any] struct {
    Value T
}

func main() {
    var payload Box[[]*Meta]
    payload.Value = []*Meta{&Meta{Name: "stale", Count: 9}}
    err := json.Unmarshal([]byte(`{"Value":[{"Name":"Ada"}]}`), &payload)
    fmt.Println(len(payload.Value), payload.Value[0] != nil, payload.Value[0].Name, payload.Value[0].Count, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 true Ada 9 <nil>\n");
}

#[test]
fn json_unmarshal_zeroes_named_alias_pointer_arrays() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Name string
    Count int
}

type MetaPair [2]*Meta

type Payload struct {
    Items MetaPair
}

func main() {
    payload := Payload{
        Items: [2]*Meta{
            &Meta{Name: "stale", Count: 9},
            &Meta{Name: "keep", Count: 7},
        },
    }
    err := json.Unmarshal([]byte(`{"Items":[{"Name":"Ada"}]}`), &payload)
    fmt.Println(payload.Items[0] != nil, payload.Items[0].Name, payload.Items[0].Count, payload.Items[1] == nil, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true Ada 9 true <nil>\n");
}

#[test]
fn json_unmarshal_replaces_named_alias_map_pointer_values() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Name string
    Count int
}

type Lookup map[string]*Meta

type Payload struct {
    Items Lookup
}

func main() {
    payload := Payload{
        Items: map[string]*Meta{
            "x": &Meta{Name: "stale", Count: 9},
            "keep": &Meta{Name: "keep", Count: 7},
        },
    }
    err := json.Unmarshal([]byte(`{"Items":{"x":{"Name":"Ada"}}}`), &payload)
    fmt.Println(payload.Items["x"] != nil, payload.Items["x"].Name, payload.Items["x"].Count, payload.Items["keep"].Name, payload.Items["keep"].Count, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true Ada 0 keep 7 <nil>\n");
}
