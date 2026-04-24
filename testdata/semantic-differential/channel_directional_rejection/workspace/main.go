package main

func main() {
	var recv <-chan int
	var send chan<- int
	recv = send
	_ = recv
}
