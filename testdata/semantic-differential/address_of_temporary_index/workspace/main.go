package main

import "fmt"

func values() []int {
	return []int{1, 2}
}

func main() {
	ptr := &values()[0]
	*ptr = 9
	fmt.Println(*ptr)
}
