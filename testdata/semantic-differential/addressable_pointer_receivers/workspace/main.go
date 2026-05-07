package main

import "fmt"

type Counter struct {
	Value int
}

func (counter *Counter) Inc(step int) int {
	counter.Value += step
	return counter.Value
}

var global = Counter{Value: 2}
var items = []Counter{Counter{Value: 1}, Counter{Value: 6}}

func capture(counters []Counter) func() int {
	return func() int {
		return counters[0].Inc(1)
	}
}

func main() {
	bump := capture(items)
	fmt.Println(global.Inc(4), global.Value)
	fmt.Println(items[0].Inc(2), items[0].Value)
	fmt.Println(bump(), bump(), items[0].Value)
}
