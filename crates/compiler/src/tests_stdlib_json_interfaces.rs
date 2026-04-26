use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn json_supports_empty_interface_targets_and_fields() {
    let source = r#"
package main
import "fmt"
import "encoding/json"

type Payload struct {
    Any interface{}
    List []interface{}
    Meta map[string]interface{}
}

func main() {
    var top interface{}
    topErr := json.Unmarshal([]byte(`{"name":"Ada","count":3,"nested":[true,null]}`), &top)

    payload := Payload{
        Any: map[string]interface{}{"keep": "x"},
        List: []interface{}{"stale"},
        Meta: map[string]interface{}{"stale": false},
    }
    payloadErr := json.Unmarshal([]byte(`{"Any":{"score":1.5},"List":[1,true,null,"go",{"count":2}],"Meta":{"label":"ok","active":true,"missing":null}}`), &payload)

    topBytes, topMarshalErr := json.Marshal(top)
    payloadBytes, payloadMarshalErr := json.Marshal(payload)

    fmt.Println(string(topBytes), topErr, topMarshalErr)
    fmt.Println(string(payloadBytes), payloadErr, payloadMarshalErr)
    _, numberOk := payload.List[0].(float64)
    flag, flagOk := payload.List[1].(bool)
    text, textOk := payload.List[3].(string)
    label, labelOk := payload.Meta["label"].(string)

    fmt.Println(numberOk, flagOk, flag, textOk, text, labelOk, label, payload.List[2] == nil, payload.Meta["missing"] == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "{\"count\":3,\"name\":\"Ada\",\"nested\":[true,null]} <nil> <nil>\n{\"Any\":{\"score\":1.5},\"List\":[1,true,null,\"go\",{\"count\":2}],\"Meta\":{\"active\":true,\"label\":\"ok\",\"missing\":null,\"stale\":false}} <nil> <nil>\ntrue true true true go true ok true true\n"
    );
}
