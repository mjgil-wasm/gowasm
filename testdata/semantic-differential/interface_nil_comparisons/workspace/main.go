package main

import "fmt"

type Describer interface {
	Describe() string
}

type box struct{}

func (*box) Describe() string {
	return "box"
}

func main() {
	var pointer *box
	var value Describer = pointer
	fmt.Println(pointer == nil)
	fmt.Println(value == nil)
}
