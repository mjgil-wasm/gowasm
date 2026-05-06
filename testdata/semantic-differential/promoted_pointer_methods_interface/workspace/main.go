package main

import "fmt"

type Named interface {
	Name() string
}

type Inner struct{}

func (*Inner) Name() string {
	return "inner"
}

type Outer struct {
	Inner
}

func show(named Named) string {
	return named.Name()
}

func main() {
	value := Outer{}
	fmt.Println(show(&value), value.Name(), (&value).Name())
}
