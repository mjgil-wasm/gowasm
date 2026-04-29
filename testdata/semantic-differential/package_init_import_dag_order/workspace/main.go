package main

import (
	"fmt"
	"example.com/app/left"
	"example.com/app/right"
)

func init() {
	fmt.Println("main")
}

func main() {
	fmt.Println(left.Message(), right.Message())
}
