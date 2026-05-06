package main

import "fmt"

func main() {
    value := "outer"
    ch := make(chan string, 1)
    ch <- "case"
    select {
    case value := <-ch:
        fmt.Println(value)
    default:
        fmt.Println("default")
    }
    fmt.Println(value)

    count := 99
    closed := make(chan int)
    close(closed)
    select {
    case count, ok := <-closed:
        fmt.Println(count, ok)
    default:
        fmt.Println("default")
    }
    fmt.Println(count)
}
