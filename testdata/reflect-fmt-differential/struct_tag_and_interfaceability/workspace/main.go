package main

import (
    "fmt"
    "reflect"
)

type Labels []string

type Payload struct {
    Name string `json:"name,omitempty"`
    hidden int
    Labels `json:"value"`
}

func main() {
    payload := Payload{
        Name: "go",
        hidden: 9,
        Labels: []string{"x", "y"},
    }
    typ := reflect.TypeOf(payload)
    value := reflect.ValueOf(payload)
    nameField := typ.Field(0)
    valueField := typ.Field(2)
    missing, missingOk := nameField.Tag.Lookup("missing")

    fmt.Println(nameField.Name, nameField.Tag.Get("json"), nameField.PkgPath == "")
    fmt.Println(missing == "", missingOk)
    fmt.Println(valueField.Name, valueField.Tag.Get("json"), valueField.Anonymous)
    fmt.Println(value.NumField())
    fmt.Println(value.Field(0).CanInterface())
    fmt.Println(value.Field(1).CanInterface())
    fmt.Println(value.Field(2).Len(), value.Field(2).Index(1).String())
    fmt.Println(valueField.Tag.Get("json") == "value")
}
