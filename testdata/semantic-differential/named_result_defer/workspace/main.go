package main

import "fmt"

func score(base int) (result int) {
	result = base
	result += 2
	return
}

func main() {
	fmt.Println(score(1))
}
