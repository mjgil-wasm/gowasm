package main

import (
    "encoding/json"
    "fmt"
)

type Report struct {
    First string `json:"first"`
    Last string `json:"last"`
    Count int `json:"count"`
}

func main() {
    labels := Box[string]{
        Values: []string{"ada", "lin"},
    }
    count, last := CountAndLast(labels.Values)
    payload, err := json.MarshalIndent(Report{
        First: labels.Values[0],
        Last: last,
        Count: count,
    }, "", "  ")
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(string(payload))
}
