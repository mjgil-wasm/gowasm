package main

import "fmt"

func main() {
	var ch1 chan int
	var ch2 chan string
	select {
	case <-ch1:
		fmt.Println("ch1")
	case ch2 <- "hello":
		fmt.Println("ch2")
	default:
		fmt.Println("default")
	}
}
