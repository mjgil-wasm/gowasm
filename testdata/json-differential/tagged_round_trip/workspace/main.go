package main

import (
    "encoding/json"
    "fmt"
)

type Payload struct {
    Name   string `json:"name"`
    Hidden string `json:"-"`
    Alias  string `json:"alias_name"`
    Count  int    `json:"count,omitempty"`
}

func main() {
    payload := Payload{Hidden: "keep"}
    err := json.Unmarshal([]byte(`{"NAME":"wrong","name":"Ada","alias_name":"go"}`), &payload)
    compact, compactErr := json.Marshal(payload)
    fmt.Println(payload.Name, payload.Hidden, payload.Alias, payload.Count, err)
    fmt.Println(string(compact), compactErr)
}
