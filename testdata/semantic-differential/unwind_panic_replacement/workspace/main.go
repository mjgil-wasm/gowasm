package main

import "fmt"

func main() {
	defer func() {
		fmt.Println("recover", recover())
	}()
	defer func() {
		fmt.Println("second")
		panic("replacement")
	}()
	defer fmt.Println("first")
	panic("initial")
}
