package main

import (
    "fmt"

    "example.com/app/report"
)

func main() {
    card := report.First()
    fmt.Println(card.Label)
    fmt.Println(card.Message())
}
