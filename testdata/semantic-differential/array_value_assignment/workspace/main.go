package main

import "fmt"

func main() {
    original := [3]int{1, 2, 3}
    copied := original
    copied[0] = 9
    fmt.Println(original, copied)

    ptr := &original[1]
    *ptr = 7
    fmt.Println(original, copied)
}
