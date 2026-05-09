package main

import (
    "encoding/json"
    "fmt"
)

type Payload struct {
    Count  int            `json:"count,string"`
    Score  float64        `json:"score,string"`
    Items  []int
    Scores map[string]int
}

func main() {
    payload := Payload{
        Count:  7,
        Score:  2.5,
        Items:  []int{9},
        Scores: map[string]int{"good": 5},
    }

    err := json.Unmarshal([]byte(`{"count":"1.5"}`), &payload)
    fmt.Println(err, payload.Count)

    err = json.Unmarshal([]byte(`{"Items":[1,9223372036854775808]}`), &payload)
    fmt.Println(err, len(payload.Items), payload.Items[0])

    err = json.Unmarshal([]byte(`{"Scores":{"bad":9223372036854775808}}`), &payload)
    fmt.Println(err, payload.Scores["good"], payload.Scores["bad"])

    err = json.Unmarshal([]byte(`{"score":"1e309"}`), &payload)
    fmt.Println(err, payload.Score)
}
