package main

import "fmt"

type Any interface{}

func main() {
	var value Any = 7
	switch boxed := value; typed := boxed.(type) {
	case int:
		fmt.Println("int", typed, boxed == value)
	default:
		fmt.Println("other")
	}
}
