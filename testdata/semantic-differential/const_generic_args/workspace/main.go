package main

import "fmt"

const answer = 7
const ratio = 1.5

func echo[T any](value T) T {
    return value
}

func main() {
    fmt.Println(echo(answer), echo(ratio))
}
