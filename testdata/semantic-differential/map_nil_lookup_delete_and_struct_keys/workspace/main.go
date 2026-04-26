package main

import "fmt"

type Key struct {
	ID int
}

func main() {
	var zero map[string]int
	value, ok := zero["missing"]
	delete(zero, "missing")
	fmt.Println(zero == nil, len(zero), value, ok)

	values := map[Key]int{Key{ID: 1}: 7}
	_, missing := values[Key{ID: 2}]
	delete(values, Key{ID: 3})
	values[Key{ID: 2}] = 11
	fmt.Println(values[Key{ID: 1}], values[Key{ID: 2}], missing, len(values))
}
