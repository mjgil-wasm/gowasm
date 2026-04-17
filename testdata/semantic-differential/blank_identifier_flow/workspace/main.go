package main

import "fmt"

func pair() (int, int) {
	return 4, 5
}

func main() {
	x, _ := pair()
	_, y := pair()
	fmt.Println(x, y)
}
