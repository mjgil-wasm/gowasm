package main

import (
    "fmt"

    "example.com/app/report"
)

type speaker interface {
    Speak() string
}

func main() {
    box := report.First()
    var value speaker = box
    fmt.Println(box.Label)
    fmt.Println(box.Speak())
    fmt.Println(value.Speak())
}
