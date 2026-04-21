package main

import "fmt"

func send(values chan int, done chan bool) {
	values <- 7
	done <- true
}

func main() {
	handoff := make(chan int)
	done := make(chan bool)
	go send(handoff, done)
	value := <-handoff
	<-done
	fmt.Println(value)

	closed := make(chan int)
	gate := make(chan bool)
	go func() {
		value, ok := <-closed
		fmt.Println(value, ok)
		gate <- true
	}()
	close(closed)
	<-gate
}
