package main

import "fmt"

type Node[T interface{ int | string }] struct {
	value T
	next  *Node[T]
}

func main() {
	var node Node[int]
	node.value = 11
	fmt.Println(node.value, node.next == nil)
}
