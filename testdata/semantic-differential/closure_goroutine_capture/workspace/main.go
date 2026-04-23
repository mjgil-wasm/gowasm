package main

import "fmt"

func main() {
	done := make(chan bool, 1)
	value := "worker"
	go func() {
		fmt.Println(value)
		done <- true
	}()
	<-done
}
