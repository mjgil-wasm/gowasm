package main

import "fmt"

type Reader interface {
    Read() string
}

type NamedReader interface {
    Reader
    Name() string
}

type inner struct {
    label string
}

func (value inner) Read() string {
    return "read:" + value.label
}

type outer struct {
    inner
}

func (outer) Name() string {
    return "outer"
}

func main() {
    var value NamedReader = outer{inner: inner{label: "ada"}}
    fmt.Println(value.Read(), value.Name())
}
