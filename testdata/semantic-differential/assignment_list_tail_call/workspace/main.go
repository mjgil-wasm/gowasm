package main

import "fmt"

func prefix() string { return "left" }
func nextPrefix() string { return "right" }
func pair() (int, string) { return 7, "go" }
func nextPair() (int, string) { return 9, "wasm" }

func main() {
    label, number, word := prefix(), pair()
    fmt.Println(label, number, word)
    label, number, word = nextPrefix(), nextPair()
    fmt.Println(label, number, word)
}
