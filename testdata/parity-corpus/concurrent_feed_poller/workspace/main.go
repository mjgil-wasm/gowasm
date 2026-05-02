package main

import "fmt"

func main() {
    report, err := runFeedPoller()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(report)
}
