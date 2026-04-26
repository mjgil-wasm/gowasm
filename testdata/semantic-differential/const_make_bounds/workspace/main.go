package main

import "fmt"

const size = 2
const capacity = size + 1

func main() {
    values := make([]int, size, capacity)
    fmt.Println(len(values), cap(values))
}
