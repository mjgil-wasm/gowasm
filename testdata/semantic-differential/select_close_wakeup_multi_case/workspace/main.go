package main

import "fmt"

func closer(values chan int) {
	close(values)
}

func main() {
	var never chan int
	stalledRecvA := make(chan int)
	stalledSendB := make(chan int)
	stalledRecvC := make(chan int)
	values := make(chan int)

	go closer(values)

	select {
	case value, ok := <-never:
		fmt.Println("nil", value, ok)
	case value, ok := <-stalledRecvA:
		fmt.Println("recvA", value, ok)
	case stalledSendB <- 22:
		fmt.Println("sendB")
	case value, ok := <-stalledRecvC:
		fmt.Println("recvC", value, ok)
	case value, ok := <-values:
		fmt.Println("closed", value, ok)
	}
}
