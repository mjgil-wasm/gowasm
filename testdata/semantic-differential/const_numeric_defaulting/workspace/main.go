package main

import "fmt"

const answer = 6
const ratio = answer + 0.5

func main() {
    var asFloat float64 = answer
    var asByte byte = 255
    var asRune rune = 'A'
    fmt.Println(answer, asFloat, ratio, asByte, asRune)
}
