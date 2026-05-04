package main

import (
    "fmt"

    "example.com/app/report"
)

func main() {
    payload, err := report.Build()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(payload)
}
