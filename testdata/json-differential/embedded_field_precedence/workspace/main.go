package main

import (
    "encoding/json"
    "fmt"
)

type Tagged struct {
    Value  string `json:"Value"`
    Shared string
    Name   string
}

type Plain struct {
    Value  string
    Shared string
    Name   string
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
