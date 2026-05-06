package main

import "fmt"

type Inner struct {
	Name string
}

func (inner Inner) Label() string {
	return "label:" + inner.Name
}

type Outer struct {
	Inner
}

func main() {
	value := Outer{Inner: Inner{Name: "ada"}}
	fmt.Println(value.Label())
}
