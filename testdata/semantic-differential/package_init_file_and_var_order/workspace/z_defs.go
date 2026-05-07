package main

import "fmt"

var second = first + 1

func init() {
	fmt.Println("z", second)
}
