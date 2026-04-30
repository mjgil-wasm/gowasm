package main

import "fmt"

func left() int { return 1 }
func right() string { return "go" }
func nextLeft() int { return 2 }
func nextRight() string { return "wasm" }

func main() {
    number, word := left(), right()
    fmt.Println(number, word)
    number, word = nextLeft(), nextRight()
    fmt.Println(number, word)
}
