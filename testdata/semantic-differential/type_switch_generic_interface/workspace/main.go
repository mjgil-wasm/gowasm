package main

import "fmt"

type Any interface{}

type Reader[T any] interface {
    Value() T
}

type Box[T any] struct {
    value T
}

func (box Box[T]) Value() T {
    return box.value
}

func main() {
    var value Any = Box[int]{value: 9}
    switch reader := value.(type) {
    case Reader[int]:
        fmt.Println(reader.Value())
    default:
        fmt.Println("other")
    }
}
