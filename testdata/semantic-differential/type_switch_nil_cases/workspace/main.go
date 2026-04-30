package main

import "fmt"

type Any interface{}
type Named interface{ Name() string }
type Box struct{ value int }

func main() {
    var value Any
    _, anyOk := value.(Any)
    _, namedOk := value.(Named)
    fmt.Println(anyOk, namedOk)

    switch typed := value.(type) {
    case Any:
        fmt.Println("any", typed == nil)
    case nil:
        fmt.Println("nil")
    default:
        fmt.Println("default")
    }

    var ptr *Box
    value = ptr
    switch typed := value.(type) {
    case nil:
        fmt.Println("wrong")
    case *Box:
        fmt.Println(typed == nil)
    default:
        fmt.Println("default")
    }
}
