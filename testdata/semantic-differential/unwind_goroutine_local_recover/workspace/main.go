package main

import "fmt"

func worker(done chan bool) {
	defer func() {
		fmt.Println("worker", recover())
		done <- true
	}()
	panic("boom")
}

func main() {
	done := make(chan bool, 1)
	go worker(done)
	<-done
	fmt.Println("main continues")
}
