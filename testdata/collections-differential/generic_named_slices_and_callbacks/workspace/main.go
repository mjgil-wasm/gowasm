package main

import (
	"cmp"
	"fmt"
	"slices"
	"sort"
)

type Bag[T any] []T

type Item struct {
	Group int
	Label string
}

func makeBag[T any](values []T) Bag[T] {
	return values
}

func main() {
	values := makeBag([]Item{
		Item{Group: 2, Label: "z"},
		Item{Group: 1, Label: "b"},
		Item{Group: 1, Label: "a"},
		Item{Group: 2, Label: "y"},
	})
	alias := values[:]
	slices.SortStableFunc(values, func(a Item, b Item) int {
		if byGroup := cmp.Compare(a.Group, b.Group); byGroup != 0 {
			return byGroup
		}
		return cmp.Compare(a.Label, b.Label)
	})
	fmt.Println(values, alias)

	sort.Slice(values, func(i int, j int) bool {
		return values[i].Label > values[j].Label
	})
	fmt.Println(values, alias)

	labels := makeBag([]string{"go", "go", "vm", "vm", "wasm"})
	labelAlias := labels[:]
	labels = slices.Compact(labels)
	fmt.Printf("%T %v %v %d %d\n", labels, labels, labelAlias[:len(labels)], len(labels), cap(labels))
}
