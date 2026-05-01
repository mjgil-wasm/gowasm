package main

import (
    "fmt"
    "reflect"
)

type Labels []string

type Notifier interface {
    Notify() string
}

type Event struct {
    Name string
}

func (e Event) Notify() string {
    return e.Name
}

type Box[T any] struct {
    Value T
    Any   interface{}
}

func main() {
    var notifier Notifier
    var labels Labels
    var zero Box[Labels]
    var box Box[Labels]
    var boxLabels Labels
    boxLabels = []string{"x", "y"}
    box.Value = boxLabels
    box.Any = Event{Name: "go"}

    zeroValue := reflect.ValueOf(zero)
    boxValue := reflect.ValueOf(box)

    fmt.Println(reflect.TypeOf(notifier) == nil)
    fmt.Println(reflect.ValueOf(notifier).IsValid())
    fmt.Println(reflect.TypeOf(labels).String())
    fmt.Println(zeroValue.Field(1).Kind() == reflect.Interface)
    fmt.Println(zeroValue.Field(1).IsNil())
    fmt.Println(zeroValue.Field(1).Elem().IsValid())
    fmt.Println(boxValue.Field(0).Type().String())
    fmt.Println(boxValue.Field(1).Elem().Type().String())
    fmt.Println(boxValue.Field(1).Elem().Field(0).String())
    fmt.Printf("%+v\n", box)
}
