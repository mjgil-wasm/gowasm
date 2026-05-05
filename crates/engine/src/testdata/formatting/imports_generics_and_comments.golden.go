package main

import (
	"fmt"
)

func choose[T any](value T) T {
	// keep
	// comment
	return value
}

func main() {
	result := choose[int](1)
	fmt.Println(result)
}
