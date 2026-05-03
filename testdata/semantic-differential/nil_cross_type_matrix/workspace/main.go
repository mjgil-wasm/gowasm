package main

import "fmt"

func main() {
	var slice []int
	var mapping map[string]int
	var fn func() int
	var ptr *int
	var any interface{}
	var ch chan int

	value, ok := mapping["missing"]
	fmt.Println(slice == nil, len(slice), cap(slice))
	fmt.Println(mapping == nil, len(mapping), value, ok)
	fmt.Println(fn == nil)
	fmt.Println(ptr == nil)
	fmt.Println(any == nil)
	fmt.Println(ch == nil)

	slice = append(slice, 3)
	mapping = map[string]int{"hit": 4}
	fn = func() int { return 5 }
	pointed := 6
	ptr = &pointed
	any = "set"
	ch = make(chan int, 2)

	fmt.Println(slice[0], len(slice), cap(slice))
	fmt.Println(len(mapping), mapping["hit"])
	fmt.Println(fn())
	fmt.Println(*ptr)
	fmt.Println(any)
}
