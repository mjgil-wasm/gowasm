package main

import "fmt"

type Counter struct {
	Value int
}

func (counter *Counter) IncBy(delta int) {
	counter.Value += delta
}

func main() {
	value := Counter{Value: 1}
	value.IncBy(4)
	fmt.Println(value.Value)
}
