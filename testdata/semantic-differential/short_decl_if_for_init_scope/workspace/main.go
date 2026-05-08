package main

import "fmt"

func main() {
    flag := 100
    if flag, ok := 7, true; ok {
        fmt.Println(flag)
    }
    fmt.Println(flag)

    index := 99
    sum := 0
    for index, limit := 0, 3; index < limit; index = index + 1 {
        sum = sum + index
    }
    fmt.Println(sum, index)
}
