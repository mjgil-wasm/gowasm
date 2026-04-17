package main

import "fmt"

type Box struct {
    Name string
}

type Labels []int

func noop() {}

func main() {
    var nilBox *Box
    box := &Box{Name: "go"}
    var any interface{} = box
    var none interface{}
    var labels Labels
    var nilLabels Labels
    m := map[string]int{"a": 1}
    var nilMap map[string]int
    ch := make(chan int, 1)
    var nilCh chan int
    f := noop
    labels = []int{1, 2}

    fmt.Println(fmt.Sprintf("%T", nilBox))
    fmt.Println(fmt.Sprintf("%T", box))
    fmt.Println(fmt.Sprintf("%T", labels))
    fmt.Println(fmt.Sprintf("%T", any))
    fmt.Println(fmt.Sprintf("%T", none))
    fmt.Println(fmt.Sprintf("%p", nilBox))
    fmt.Println(fmt.Sprintf("%p", box) == fmt.Sprintf("%p", box))
    fmt.Println(fmt.Sprintf("%p", box) != fmt.Sprintf("%p", &Box{Name: "go"}))
    fmt.Println(fmt.Sprintf("%p", nilLabels))
    fmt.Println(fmt.Sprintf("%p", labels) == fmt.Sprintf("%p", labels))
    fmt.Println(fmt.Sprintf("%p", nilMap))
    fmt.Println(fmt.Sprintf("%p", m) == fmt.Sprintf("%p", m))
    fmt.Println(fmt.Sprintf("%p", nilCh))
    fmt.Println(fmt.Sprintf("%p", ch) == fmt.Sprintf("%p", ch))
    fmt.Println(fmt.Sprintf("%p", f) == fmt.Sprintf("%p", f))
}
