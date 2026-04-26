package main

import "fmt"

func main() {
	values := make(chan int, 2)
	values <- 8
	values <- 9
	close(values)

	for value := range values {
		fmt.Println(value)
	}

	value, ok := <-values
	fmt.Println(value, ok)
}
