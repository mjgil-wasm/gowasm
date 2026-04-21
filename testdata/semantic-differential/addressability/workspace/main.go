package main

import "fmt"

type Counter struct {
	Value int
}

func bump(target *int) {
	*target += 2
}

func main() {
	value := Counter{Value: 1}
	bump(&value.Value)
	fmt.Println(value.Value)
}
