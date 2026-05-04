package main

import "fmt"

func main() {
    value := "outer"
    {
        value, inner := "inner", 7
        fmt.Println(value, inner)
    }
    fmt.Println(value)
}
