package main

import "fmt"

func main() {
	value := 1
	defer func() {
		fmt.Println("defer", value)
	}()
	run := func() {
		value++
		fmt.Println("run", value)
	}
	run()
}
