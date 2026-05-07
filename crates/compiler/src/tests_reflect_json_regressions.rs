use super::compile_source;
use gowasm_vm::Vm;

struct RegressionCase {
    name: &'static str,
    source: &'static str,
    expected_stdout: &'static str,
}

#[test]
fn reflect_and_tag_regression_corpus_cases_run() {
    for case in regression_cases() {
        run_case(case);
    }
}

fn regression_cases() -> Vec<RegressionCase> {
    vec![
        RegressionCase {
            name: "tagged struct round trip stays aligned across reflect and fmt",
            source: r#"
package main

import (
    "encoding/json"
    "fmt"
    "reflect"
)

type Payload struct {
    Name string `json:"name"`
    Hidden string `json:"-"`
    Alias string `json:"alias_name"`
    Count int `json:"count,omitempty"`
}

func main() {
    payload := Payload{Hidden: "keep"}
    err := json.Unmarshal([]byte(`{"name":"Ada","Hidden":"skip","alias_name":"go"}`), &payload)
    typ := reflect.TypeOf(payload)
    value := reflect.ValueOf(&payload).Elem()
    compact, compactErr := json.Marshal(payload)

    fmt.Println(err)
    fmt.Println(typ.Kind() == reflect.Struct)
    fmt.Println(typ.Field(0).Name, typ.Field(0).Tag)
    fmt.Println(typ.Field(2).Name, typ.Field(2).Tag)
    fmt.Println(
        value.Field(0).String(),
        value.Field(1).String(),
        value.Field(2).String(),
        value.Field(3).Int(),
    )
    fmt.Printf("%+v\n", payload)
    fmt.Println(string(compact), compactErr)
}
"#,
            expected_stdout: concat!(
                "<nil>\n",
                "true\n",
                "Name json:\"name\"\n",
                "Alias json:\"alias_name\"\n",
                "Ada keep go 0\n",
                "{Name:Ada Hidden:keep Alias:go Count:0}\n",
                "{\"name\":\"Ada\",\"alias_name\":\"go\"} <nil>\n",
            ),
        },
        RegressionCase {
            name: "omitempty and composite formatting share the same reflected shape",
            source: r#"
package main

import (
    "encoding/json"
    "fmt"
    "reflect"
)

type Snapshot struct {
    Name string `json:"name"`
    Labels []string `json:"labels,omitempty"`
    Count int `json:"count,omitempty"`
}

func main() {
    snapshot := Snapshot{Name: "Ada", Labels: []string{"x", "y"}}
    typ := reflect.TypeOf(snapshot)
    value := reflect.ValueOf(snapshot)
    payload, payloadErr := json.Marshal(snapshot)

    fmt.Printf("%+v\n", snapshot)
    fmt.Println(typ.Field(1).Name, typ.Field(1).Tag)
    fmt.Println(value.Field(1).Len(), value.Field(1).Index(0).String())
    fmt.Println(string(payload), payloadErr)
}
"#,
            expected_stdout: concat!(
                "{Name:Ada Labels:[x y] Count:0}\n",
                "Labels json:\"labels,omitempty\"\n",
                "2 x\n",
                "{\"name\":\"Ada\",\"labels\":[\"x\",\"y\"]} <nil>\n",
            ),
        },
    ]
}

fn run_case(case: RegressionCase) {
    let program = compile_source(case.source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        case.expected_stdout,
        "corpus case `{}` produced unexpected stdout",
        case.name
    );
}
