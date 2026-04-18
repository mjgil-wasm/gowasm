package main

import (
	"encoding/json"
	"fmt"
)

type Payload struct {
	Any  interface{}
	List []interface{}
	Meta map[string]interface{}
}

func main() {
	var top interface{}
	topErr := json.Unmarshal([]byte(`{"name":"Ada","count":3,"nested":[true,null]}`), &top)

	payload := Payload{
		Any:  map[string]interface{}{"keep": "x"},
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
