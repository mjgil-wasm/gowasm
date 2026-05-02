package main

import "fmt"

func main() {
    summary, err := buildSummary()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(summary)
}
