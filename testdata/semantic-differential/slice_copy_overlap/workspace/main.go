package main

import "fmt"

func main() {
    values := []int{1, 2, 3, 4}
    fmt.Println(copy(values[1:], values), values)
    fmt.Println(copy(values, values[2:]), values)
}
