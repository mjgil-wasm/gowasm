package main

import "fmt"

type Label string

func (l Label) String() string {
    return "label<" + string(l) + ">"
}

type Problem struct{}

func (Problem) Error() string {
    return "problem<7>"
}

type Labels []string

func main() {
    var err error = Problem{}
    var labels Labels
    labels = Labels([]string{"x", "y"})
    var nilLabels Labels

    fmt.Println(Label("go"))
    fmt.Println(err)
    fmt.Println(fmt.Sprintf("%v %s %q", Label("go"), Label("go"), Label("go")))
    fmt.Printf("%#v\n", labels)
    fmt.Printf("%#v\n", nilLabels)
}
