package main

import (
	"cmp"
	"fmt"
	"slices"
	"sort"
)

func main() {
	var ints []int
	fmt.Println(ints == nil)
	slices.Reverse(ints)
	fmt.Println(ints == nil)
	slices.SortFunc(ints, func(a int, b int) int {
		return cmp.Compare(a, b)
	})
	fmt.Println(ints == nil)
	slices.SortStableFunc(ints, func(a int, b int) int {
		return cmp.Compare(a, b)
	})
	fmt.Println(ints == nil)
	sort.Slice(ints, func(i int, j int) bool {
		return ints[i] < ints[j]
	})
	fmt.Println(ints == nil)
	sort.SliceStable(ints, func(i int, j int) bool {
		return ints[i] < ints[j]
	})
	fmt.Println(ints == nil)

	var plain []int
	sort.Ints(plain)
	fmt.Println(plain == nil)

	var words []string
	sort.Strings(words)
	fmt.Println(words == nil)

	var floats []float64
	sort.Float64s(floats)
	fmt.Println(floats == nil)
}
