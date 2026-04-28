package main

import "fmt"

func main() {
	var slice []int
	var mapping map[string]int
	var fn func() int
	var ptr *int
	var ch chan int
	var any interface{}
	var err error

	fmt.Println(slice, mapping, fn, ptr, any, ch, err)

	var typed *int
	any = typed
	fmt.Println(typed == nil, any == nil, any)

	slice = []int{}
	mapping = map[string]int{}
	fmt.Println(slice == nil, mapping == nil, slice, mapping)
}
