package main

import "fmt"

func main() {
    report, err := buildCollectionReport()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(report)
}
