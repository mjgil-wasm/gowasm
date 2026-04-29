package main

import "fmt"

func main() {
    var nilSlice []int
    empty := []int{}

    fmt.Println(nilSlice == nil, len(nilSlice), cap(nilSlice))
    fmt.Println(empty == nil, len(empty), cap(empty))

    nilSlice = append(nilSlice, 7)
    fmt.Println(nilSlice, len(nilSlice), cap(nilSlice))
}
