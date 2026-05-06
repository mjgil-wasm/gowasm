use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn json_marshal_encodes_exported_struct_fields_and_sorts_map_keys() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Person struct {
    Name string
    age int
}

func main() {
    person, err := json.Marshal(&Person{Name: "Ada", age: 37})
    fmt.Println(string(person), err)

    payload, payloadErr := json.Marshal(map[string]int{"b": 2, "a": 1})
    fmt.Println(string(payload), payloadErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"Name\":\"Ada\"} <nil>\n{\"a\":1,\"b\":2} <nil>\n"
    );
}

#[test]
fn json_marshal_respects_common_json_tags() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Name string `json:"name"`
    Hidden string `json:"-"`
    Alias string `json:"alias_name"`
    Notes string `json:",omitempty"`
    Count int `json:"count,omitempty"`
    Labels []string `json:"labels,omitempty"`
    lower string `json:"lower"`
}

func main() {
    compact, compactErr := json.Marshal(Payload{Name: "Ada", Hidden: "secret", Alias: "go"})
    full, fullErr := json.Marshal(Payload{
        Name: "Ada",
        Hidden: "secret",
        Alias: "go",
        Notes: "memo",
        Count: 3,
        Labels: []string{"x"},
        lower: "ignored",
    })
    fmt.Println(string(compact), compactErr)
    fmt.Println(string(full), fullErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"name\":\"Ada\",\"alias_name\":\"go\"} <nil>\n{\"name\":\"Ada\",\"alias_name\":\"go\",\"Notes\":\"memo\",\"count\":3,\"labels\":[\"x\"]} <nil>\n"
    );
}

#[test]
fn json_marshal_supports_string_struct_tags() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Count int `json:"count,string"`
    Ready bool `json:"ready,string"`
    Score float64 `json:"score,string"`
    Name string `json:"name,string"`
}

func main() {
    payload, err := json.Marshal(Payload{
        Count: 3,
        Ready: true,
        Score: 1.5,
        Name: "Ada",
    })
    fmt.Println(string(payload), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"count\":\"3\",\"ready\":\"true\",\"score\":\"1.5\",\"name\":\"\\\"Ada\\\"\"} <nil>\n"
    );
}

#[test]
fn json_marshal_resolves_embedded_field_precedence() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Tagged struct {
    Value string `json:"Value"`
    Shared string
    Name string
}

type Plain struct {
    Value string
    Shared string
    Name string
}

type Payload struct {
    Name string
    *Tagged
    Plain
}

func main() {
    payload, err := json.Marshal(Payload{
        Name: "outer",
        Tagged: &Tagged{
            Value: "tagged",
            Shared: "tagged-shared",
            Name: "embedded-tagged",
        },
        Plain: Plain{
            Value: "plain",
            Shared: "plain-shared",
            Name: "embedded-plain",
        },
    })
    fmt.Println(string(payload), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"Name\":\"outer\",\"Value\":\"tagged\"} <nil>\n"
    );
}

#[test]
fn json_marshal_preserves_nil_vs_empty_maps_and_slices() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

func main() {
    var nilMap map[string]int
    emptyMap := map[string]int{}
    var nilSlice []int
    emptySlice := []int{}

    nilMapBytes, _ := json.Marshal(nilMap)
    emptyMapBytes, _ := json.Marshal(emptyMap)
    nilSliceBytes, _ := json.Marshal(nilSlice)
    emptySliceBytes, _ := json.Marshal(emptySlice)

    fmt.Println(string(nilMapBytes))
    fmt.Println(string(emptyMapBytes))
    fmt.Println(string(nilSliceBytes))
    fmt.Println(string(emptySliceBytes))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "null\n{}\nnull\n[]\n");
}

#[test]
fn json_marshal_returns_errors_for_unsupported_values() {
    let source = r#"
package main
import "fmt"
import "math"
import "encoding/json"

func main() {
    badMap, badMapErr := json.Marshal(map[int]string{2: "b", 1: "a"})
    fmt.Println(len(badMap), badMapErr)

    badFloat, badFloatErr := json.Marshal(math.NaN())
    fmt.Println(len(badFloat), badFloatErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0 json: unsupported type: map with non-string keys\n0 json: unsupported value: NaN\n"
    );
}

#[test]
fn json_marshal_indent_formats_output_and_valid_accepts_bytes() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

func main() {
    pretty, err := json.MarshalIndent(map[string]int{"b": 2, "a": 1}, ">", "  ")
    fmt.Println(string(pretty))
    fmt.Println(err)
    fmt.Println(json.Valid(pretty))
    fmt.Println(json.Valid([]byte("{")))
    fmt.Println(json.Valid([]byte{255}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\n>  \"a\": 1,\n>  \"b\": 2\n>}\n<nil>\nfalse\nfalse\nfalse\n"
    );
}

#[test]
fn json_marshal_uses_custom_json_and_text_methods() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct{}

func (Payload) MarshalJSON() ([]byte, error) {
    return []byte(`{"kind":"payload"}`), nil
}

type Label string

func (label Label) MarshalText() ([]byte, error) {
    return []byte("label<" + string(label) + ">"), nil
}

type Both string

func (Both) MarshalJSON() ([]byte, error) {
    return []byte(`"json"`), nil
}

func (Both) MarshalText() ([]byte, error) {
    return []byte("text"), nil
}

type Wrapper struct {
    Label Label
    Both Both
}

func main() {
    payload, payloadErr := json.Marshal(Payload{})
    wrapper, wrapperErr := json.Marshal(Wrapper{Label: Label("go"), Both: Both("x")})
    fmt.Println(string(payload), payloadErr)
    fmt.Println(string(wrapper), wrapperErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"kind\":\"payload\"} <nil>\n{\"Label\":\"label\\u003cgo\\u003e\",\"Both\":\"json\"} <nil>\n"
    );
}

#[test]
fn json_marshal_reports_custom_method_errors_and_invalid_json() {
    let source = r#"
package main
import "fmt"
import "errors"
import "encoding/json"

type Broken struct{}

func (Broken) MarshalJSON() ([]byte, error) {
    return nil, errors.New("boom")
}

type BadSyntax struct{}

func (BadSyntax) MarshalJSON() ([]byte, error) {
    return []byte("{"), nil
}

type BadText struct{}

func (BadText) MarshalText() ([]byte, error) {
    return nil, errors.New("bad text")
}

func main() {
    broken, brokenErr := json.Marshal(Broken{})
    badSyntax, badSyntaxErr := json.Marshal(BadSyntax{})
    badText, badTextErr := json.Marshal(BadText{})
    fmt.Println(len(broken), brokenErr)
    fmt.Println(len(badSyntax), badSyntaxErr)
    fmt.Println(len(badText), badTextErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0 json: error calling MarshalJSON for struct value `{}`: boom\n0 json: error calling MarshalJSON for struct value `{}`: returned invalid JSON\n0 json: error calling MarshalText for struct value `{}`: bad text\n"
    );
}

#[test]
fn json_marshal_keeps_nil_marshaled_pointers_as_null() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Node struct{}

func (*Node) MarshalJSON() ([]byte, error) {
    return []byte(`"unexpected"`), nil
}

func main() {
    var node *Node
    payload, err := json.Marshal(node)
    fmt.Println(string(payload), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "null <nil>\n");
}

#[test]
fn json_unmarshal_decodes_struct_scalars_and_fixed_arrays() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Enabled bool
    Count int
}

type Payload struct {
    Name string
    Count int
    Flags [2]bool
    Meta Meta
}

func main() {
    var payload Payload
    err := json.Unmarshal([]byte(`{"Name":"Ada","Count":3,"Flags":[true,false],"Meta":{"Enabled":true,"Count":9}}`), &payload)
    fmt.Println(payload.Name, payload.Count, payload.Flags[0], payload.Flags[1], payload.Meta.Enabled, payload.Meta.Count, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Ada 3 true false true 9 <nil>\n");
}

#[test]
fn json_unmarshal_respects_common_json_tags() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Name string `json:"name"`
    Hidden string `json:"-"`
    Alias string `json:"alias_name"`
    Notes string `json:",omitempty"`
    Count int `json:"count"`
}

func main() {
    payload := Payload{Hidden: "keep"}
    err := json.Unmarshal([]byte(`{"name":"Ada","Hidden":"skip","-":"skip","alias_name":"go","Notes":"memo","count":3}`), &payload)
    fmt.Println(payload.Name, payload.Hidden, payload.Alias, payload.Notes, payload.Count, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Ada keep go memo 3 <nil>\n");
}

#[test]
fn json_unmarshal_supports_string_struct_tags() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Count int `json:"count,string"`
    Ready bool `json:"ready,string"`
    Score float64 `json:"score,string"`
    Name string `json:"name,string"`
}

func main() {
    payload := Payload{Count: 9, Ready: false, Score: 0.5, Name: "stale"}
    err := json.Unmarshal([]byte(`{"count":"3","ready":"true","score":"1.5","name":"\"Ada\""}`), &payload)
    fmt.Println(payload.Count, payload.Ready, payload.Score, payload.Name, err)

    err = json.Unmarshal([]byte(`{"count":3,"ready":"bad"}`), &payload)
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "3 true 1.5 Ada <nil>\njson: invalid use of ,string struct tag, trying to unmarshal unquoted value into int\n"
    );
}

#[test]
fn json_unmarshal_hardens_numeric_decode_for_int_targets() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Count int
    CountPtr *int
    Items []int
    Scores map[string]int
}

func main() {
    ptrValue := 11
    payload := Payload{
        Count: 7,
        CountPtr: &ptrValue,
        Items: []int{9},
        Scores: map[string]int{"keep": 5},
    }

    err := json.Unmarshal([]byte(`{"Count":1.5}`), &payload)
    fmt.Println(err, payload.Count)

    err = json.Unmarshal([]byte(`{"CountPtr":9223372036854775808}`), &payload)
    fmt.Println(err, *payload.CountPtr)

    err = json.Unmarshal([]byte(`{"Items":[1,9223372036854775808]}`), &payload)
    fmt.Println(err, len(payload.Items), payload.Items[0])

    err = json.Unmarshal([]byte(`{"Scores":{"bad":9223372036854775808}}`), &payload)
    fmt.Println(err, payload.Scores["keep"], payload.Scores["bad"])
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "json: cannot unmarshal number 1.5 into Go struct field Payload.Count of type int 7\njson: cannot unmarshal number 9223372036854775808 into Go struct field Payload.CountPtr of type int 11\njson: cannot unmarshal number 9223372036854775808 into Go struct field Payload.Items of type int 2 1\njson: cannot unmarshal number 9223372036854775808 into Go struct field Payload.Scores of type int 5 0\n"
    );
}

#[test]
fn json_unmarshal_hardens_quoted_numeric_decode() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Count int `json:"count,string"`
    Score float64 `json:"score,string"`
}

func main() {
    payload := Payload{Count: 7, Score: 2.5}

    err := json.Unmarshal([]byte(`{"count":"1.5"}`), &payload)
    fmt.Println(err, payload.Count)

    err = json.Unmarshal([]byte(`{"count":"9223372036854775808"}`), &payload)
    fmt.Println(err, payload.Count)

    err = json.Unmarshal([]byte(`{"score":"1e309"}`), &payload)
    fmt.Println(err, payload.Score)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "json: cannot unmarshal number 1.5 into Go struct field Payload.count of type int 7\njson: cannot unmarshal number 9223372036854775808 into Go struct field Payload.count of type int 7\njson: cannot unmarshal number 1e309 into Go struct field Payload.score of type float64 2.5\n"
    );
}

#[test]
fn json_unmarshal_matches_struct_fields_case_insensitively() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Name string `json:"name"`
    Count int `json:"count"`
    Alias string
}

func main() {
    var payload Payload
    err := json.Unmarshal([]byte(`{"NAME":"wrong","name":"Ada","COUNT":7,"count":3,"alias":"go"}`), &payload)
    fmt.Println(payload.Name, payload.Count, payload.Alias, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Ada 3 go <nil>\n");
}

#[test]
fn json_unmarshal_resolves_embedded_field_precedence() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Tagged struct {
    Value string `json:"Value"`
    Shared string
    Name string
}

type Plain struct {
    Value string
    Shared string
    Name string
}

type Payload struct {
    Name string
    *Tagged
    Plain
}

func main() {
    var payload Payload
    err := json.Unmarshal([]byte(`{"Name":"outer","Value":"decoded","Shared":"ignored"}`), &payload)
    fmt.Println(
        payload.Name,
        payload.Tagged != nil,
        payload.Tagged.Value,
        payload.Tagged.Name == "",
        payload.Plain.Value == "",
        payload.Tagged.Shared == "",
        payload.Plain.Shared == "",
        err,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "outer true decoded true true true true <nil>\n"
    );
}

#[test]
fn json_unmarshal_decodes_typed_slices_and_string_key_maps() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Item struct {
    Name string
    Count int
}

type Payload struct {
    Items []Item
    Scores map[string]int
}

func main() {
    payload := Payload{
        Items: []Item{Item{Name: "stale", Count: 9}},
        Scores: map[string]int{"keep": 7},
    }
    err := json.Unmarshal([]byte(`{"Items":[{"Name":"Ada","Count":3},{"Name":"Bob","Count":5}],"Scores":{"alpha":1,"beta":2}}`), &payload)
    fmt.Println(len(payload.Items), payload.Items[0].Name, payload.Items[1].Count, len(payload.Scores), payload.Scores["alpha"], payload.Scores["beta"], payload.Scores["keep"], err)

    err = json.Unmarshal([]byte(`{"Items":null,"Scores":null}`), &payload)
    fmt.Println(payload.Items == nil, payload.Scores == nil, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 Ada 5 3 1 2 7 <nil>\ntrue true <nil>\n");
}

#[test]
fn json_unmarshal_allocates_nil_pointer_targets() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Meta struct {
    Count int
}

type Payload struct {
    Name string
    Meta *Meta
}

func main() {
    var payload *Payload
    err := json.Unmarshal([]byte(`{"Name":"Ada","Meta":{"Count":3}}`), &payload)
    fmt.Println(payload != nil, payload.Name, payload.Meta != nil, payload.Meta.Count, err)

    err = json.Unmarshal([]byte(`{"Meta":null}`), &payload)
    fmt.Println(payload != nil, payload.Meta == nil, err)

    err = json.Unmarshal([]byte(`null`), &payload)
    fmt.Println(payload == nil, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true Ada true 3 <nil>\ntrue true <nil>\ntrue <nil>\n"
    );
}

#[test]
fn json_unmarshal_reports_invalid_targets_and_invalid_json() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Name string
}

func main() {
    var ready chan int
    err := json.Unmarshal([]byte(`1`), &ready)
    fmt.Println(err)

    count := 0
    err = json.Unmarshal([]byte(`1`), count)
    fmt.Println(err)

    var scores map[int]int
    err = json.Unmarshal([]byte(`{"1": 1}`), &scores)
    fmt.Println(err)

    var payload Payload
    err = json.Unmarshal([]byte(`{`), &payload)
    fmt.Println(err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "json: unsupported unmarshal target: channel\njson: Unmarshal expects a non-nil pointer target\njson: unsupported unmarshal target: map with non-string keys\njson: invalid JSON\n"
    );
}

#[test]
fn json_tag_option_errors_are_deterministic() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type BadOptions struct {
    Count int `json:"count,string,string"`
}

type UnsupportedString struct {
    Labels []string `json:"labels,string"`
}

type UnknownOption struct {
    Name string `json:"name,inline"`
}

func main() {
    badOptions, badOptionsErr := json.Marshal(BadOptions{Count: 3})
    unsupportedString, unsupportedStringErr := json.Marshal(UnsupportedString{Labels: []string{"x"}})
    var unknownOption UnknownOption
    unknownOptionErr := json.Unmarshal([]byte(`{"name":"Ada"}`), &unknownOption)
    fmt.Println(len(badOptions), badOptionsErr)
    fmt.Println(len(unsupportedString), unsupportedStringErr)
    fmt.Println(unknownOptionErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0 json: malformed struct tag for field \"Count\": duplicate json tag option \"string\"\n0 json: malformed struct tag for field \"Labels\": ,string is only supported on bool, int, float64, and string fields\njson: malformed struct tag for field \"Name\": unsupported json tag option \"inline\"\n"
    );
}
