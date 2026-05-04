package main

import "fmt"

func main() {
	var never chan int
	recvA := make(chan int, 1)
	sendB := make(chan int, 1)
	recvC := make(chan int, 1)
	sendD := make(chan int, 1)

	recvA <- 11
	recvC <- 33
	sum := 0

	for i := 0; i < 4; i++ {
		select {
		case <-never:
		case value := <-recvA:
			sum += value
		case sendB <- 22:
		case value := <-recvC:
			sum += value
		case sendD <- 44:
		default:
			fmt.Println("default")
		}
	}

	fmt.Println(sum)
	fmt.Println(<-sendB)
	fmt.Println(<-sendD)
}
