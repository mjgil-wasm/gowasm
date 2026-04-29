package main

import "fmt"

func main() {
    word := "outer"
    values := []string{"go", "wasm"}
    for index, word := range values {
        fmt.Println(index, word)
    }
    fmt.Println(word)

    item := "after"
    ch := make(chan string, 2)
    ch <- "x"
    ch <- "y"
    close(ch)
    for item := range ch {
        fmt.Println(item)
    }
    fmt.Println(item)
}
