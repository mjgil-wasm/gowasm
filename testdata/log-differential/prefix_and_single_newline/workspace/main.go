package main

import "log"

func main() {
	log.SetFlags(0)
	log.SetPrefix("LOG: ")
	log.Println("ready", 7)
	log.Printf("done:%s\n", "ok")
	log.Print("tail")
}
