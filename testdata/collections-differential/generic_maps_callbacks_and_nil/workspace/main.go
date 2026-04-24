package main

import (
	"fmt"
	"maps"
)

type Dict[T any] map[string]T

func makeDict[T any](values map[string]T) Dict[T] {
	return values
}

func main() {
	var empty Dict[int]
	var other Dict[int]
	fmt.Println(empty == nil, maps.Equal(empty, maps.Clone(empty)), maps.Equal(empty, other))

	base := makeDict(map[string]int{"a": 1, "b": 2, "c": 3})
	clone := maps.Clone(base)
	maps.Copy(clone, makeDict(map[string]int{"c": 30, "d": 4}))
	maps.DeleteFunc(clone, func(k string, v int) bool {
		return v > 20
	})
	fmt.Println(base["c"], base["d"])
	fmt.Println(clone["a"], clone["b"], clone["c"], clone["d"])
	fmt.Println(maps.EqualFunc(base, makeDict(map[string]int{"a": 10, "b": 20, "c": 30}), func(left int, right int) bool {
		return right == left*10
	}))
}
