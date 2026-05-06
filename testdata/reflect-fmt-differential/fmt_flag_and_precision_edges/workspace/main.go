package main

import "fmt"

func main() {
    fmt.Println(fmt.Sprintf("%x", -42))
    fmt.Println(fmt.Sprintf("%#x", 42))
    fmt.Println(fmt.Sprintf("%#08x", 42))
    fmt.Println(fmt.Sprintf("%#o", 8))
    fmt.Println(fmt.Sprintf("%O", 8))
    fmt.Println(fmt.Sprintf("%b", -5))
    fmt.Println(fmt.Sprintf("%#08x", -42))
    fmt.Println(fmt.Sprintf("[%#08x][% 6d][%+06d][%.3s][%8.3s]", 42, 42, 42, "gopher", "gopher"))
}
