package main

import "fmt"

var second = readFirst()
var first = 41

func readFirst() int {
	return first + 1
}

func main() {
	fmt.Println(first, second)
}
