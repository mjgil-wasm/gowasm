package main

import "fmt"

func main() {
	for i, v := range [2]int{4, 5} {
		fmt.Println("array", i, v)
	}
	for i, v := range []int{6, 7} {
		fmt.Println("slice", i, v)
	}
	seen := map[string]int{}
	for k, v := range map[string]int{"a": 1, "b": 2} {
		seen[k] = v
	}
	fmt.Println("map", "a", seen["a"])
	fmt.Println("map", "b", seen["b"])
	for i, r := range "hé" {
		fmt.Println("string", i, r)
	}
	for i := range 3 {
		fmt.Println("int", i)
	}
	ch := make(chan int, 2)
	ch <- 8
	ch <- 9
	close(ch)
	for v := range ch {
		fmt.Println("chan", v)
	}
}
