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

type Box struct {
    Value Labels
    Any   interface{}
}

func main() {
    var notifier Notifier
    var labels Labels
    var boxLabels Labels
    zero := Box{}
    box := Box{
        Any:   Event{Name: "go"},
    }
    boxLabels = []string{"x", "y"}
    box.Value = boxLabels

    fmt.Println(reflect.TypeOf(notifier) == nil)
    fmt.Println(reflect.ValueOf(notifier).IsValid())

    labelsType := reflect.TypeOf(labels)
    fmt.Println(labelsType.Kind() == reflect.Slice)
    fmt.Println(labelsType.Elem().String())

    boxType := reflect.TypeOf(box)
    fmt.Println(boxType.Name())
    fmt.Println(boxType.NumField())
    fmt.Println(boxType.Field(0).Type.String())
    fmt.Println(boxType.Field(1).Type.String())

    zeroValue := reflect.ValueOf(zero)
    fmt.Println(zeroValue.Field(1).Kind() == reflect.Interface)
    fmt.Println(zeroValue.Field(1).IsNil())
    fmt.Println(zeroValue.Field(1).Elem().IsValid())

    boxValue := reflect.ValueOf(box)
    fmt.Println(boxValue.Field(0).Type().String())
    fmt.Println(boxValue.Field(1).Kind() == reflect.Interface)
    fmt.Println(boxValue.Field(1).Elem().Type().String())
    fmt.Println(boxValue.Field(1).Elem().Field(0).String())
}
