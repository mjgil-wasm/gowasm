package main

import "fmt"

func main() {
    report, err := buildReport()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(report)
}
