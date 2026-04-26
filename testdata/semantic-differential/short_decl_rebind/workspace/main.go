package main

import "fmt"

func pair() (int, int) {
	return 2, 3
}

func main() {
	x := 1
	x, y := pair()
	fmt.Println(x, y)
}
