package main

import (
    "fmt"
    "log"
    "reflect"
)

type Label string

func (l Label) String() string {
    return "label<" + string(l) + ">"
}

type Problem struct{}

func (Problem) Error() string {
    return "problem<7>"
}

type Box struct {
    Name  string
    Count int
}

type Labels []string
type Lookup map[string]int

func main() {
    var err error = Problem{}
    box := Box{Name: "go", Count: 2}
    var labels Labels
    var nilLabels Labels
    var lookup Lookup
    labels = Labels([]string{"x", "y"})
    lookup = map[string]int{"only": 7}
    lookupType := reflect.TypeOf(lookup)

    fmt.Println(Label("go"))
    fmt.Println(err)
    fmt.Println(reflect.TypeOf(labels).String())
    fmt.Println(lookupType.Key().String(), lookupType.Elem().String())
    fmt.Printf("%+v\n", box)
    fmt.Printf("%#v\n", box)
    fmt.Printf("%#v\n", &box)
    fmt.Printf("%#v\n", labels)
    fmt.Printf("%#v\n", nilLabels)
    fmt.Printf("%#v\n", lookup)
    log.Println(&box)
}
