package main

import "fmt"

func helper() {
	fmt.Println("helper", recover() == nil)
}

func main() {
	defer func() {
		helper()
		fmt.Println("direct", recover())
	}()
	panic("boom")
}
