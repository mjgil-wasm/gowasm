package report

import (
    "encoding/json"

    "example.com/app/names"
    "example.com/remote/greeter"
)

type Summary struct {
    First string `json:"first"`
    Second string `json:"second"`
    Count int `json:"count"`
}

func Build() (string, error) {
    labels := names.List()
    payload, err := json.MarshalIndent(Summary{
        First: greeter.Message(labels[0]),
        Second: greeter.Message(labels[1]),
        Count: len(labels),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
