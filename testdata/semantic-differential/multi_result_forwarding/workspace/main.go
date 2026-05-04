package main

import "fmt"

func pair() (string, int) {
	return "go", 7
}

func wrap() (string, int) {
	return pair()
}

func main() {
	name, count := wrap()
	fmt.Println(name, count)
}
