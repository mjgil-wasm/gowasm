package main

import "fmt"

type Counter struct {
	Value int
}

func (counter *Counter) Inc() int {
	counter.Value++
	return counter.Value
}

func values() []Counter {
	return []Counter{Counter{Value: 1}}
}

func main() {
	fmt.Println(values()[0].Inc())
}
