package main

import (
    "fmt"
    "log"
    "reflect"
)

type Config struct {
    Name   string `json:"name"`
    hidden int    `json:"hidden"`
}

func main() {
    cfg := Config{Name: "Ada", hidden: 9}
    typ := reflect.TypeOf(cfg)
    exported := typ.Field(0)
    hidden := typ.Field(1)

    fmt.Println(exported.Name, exported.Tag.Get("json"), exported.PkgPath == "")
    fmt.Println(hidden.Name, hidden.Tag.Get("json"), hidden.PkgPath)
    log.Printf("%#v\n", cfg)
}
