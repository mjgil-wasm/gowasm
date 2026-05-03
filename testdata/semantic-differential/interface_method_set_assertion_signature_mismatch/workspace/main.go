package main

import "fmt"

type Any interface{}

type Needs interface {
    Read(value int) int
}

type WrongParam struct{}

func (WrongParam) Read(text string) int {
    return 1
}

type WrongResult struct{}

func (WrongResult) Read(value int) string {
    return "bad"
}

func main() {
    var value Any = WrongParam{}
    _, wrongParam := value.(Needs)
    value = WrongResult{}
    _, wrongResult := value.(Needs)
    fmt.Println(wrongParam, wrongResult)
}
