package main

import "fmt"

func main() {
    base := []int{1, 2, 3, 4}
    head := base[:0]
    mid := base[1:3]
    tail := base[4:]

    fmt.Println(len(head), cap(head))
    fmt.Println(len(mid), cap(mid), mid[0], mid[1])
    fmt.Println(len(tail), cap(tail))
}
