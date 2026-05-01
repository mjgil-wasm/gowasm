package main

import "fmt"

func main() {
	var zero chan int
	fmt.Println(zero == nil)

	live := make(chan int, 1)
	fmt.Println(live == nil)

	var recv <-chan int = live
	var send chan<- int = live
	send <- 7
	fmt.Println(<-recv)
}
