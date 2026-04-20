package main

import "fmt"

func main() {
	var run func(int) int
	run = func(n int) int {
		if n == 0 {
			return 0
		}
		return 1 + run(n-1)
	}
	fmt.Println(run(5))
}
