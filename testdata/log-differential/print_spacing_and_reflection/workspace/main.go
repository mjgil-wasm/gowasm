package main

import "log"

type Labels []string

func main() {
	log.SetFlags(0)
	log.SetPrefix("")
	labels := Labels([]string{"x", "y"})
	single := Labels([]string{"z"})
	log.Print("go", "wasm")
	log.Print("value:", 7, labels)
	log.Printf("%T %#v %v", single, single, []int{1, 2})
}
