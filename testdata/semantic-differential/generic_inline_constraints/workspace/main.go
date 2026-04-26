package main

import "fmt"

type Label string

func (l Label) String() string {
	return string(l)
}

func Show[T interface {
	comparable
	String() string
}](value T) string {
	return value.String()
}

func Echo[T interface{ int | string }](value T) T {
	return value
}

func main() {
	fmt.Println(Show(Label("inline")))
	fmt.Println(Echo(7))
	fmt.Println(Echo("go"))
}
