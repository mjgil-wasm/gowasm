package main

import "fmt"

func main() {
    values := map[string]int{"go": 7}
    var any interface{} = 11
    ch := make(chan string, 1)
    ch <- "ready"
    close(ch)

    label, number, found := "map", values["go"]
    fmt.Println(label, number, found)
    label, number, found = "map", values["missing"]
    fmt.Println(label, number, found)

    label, asserted, ok := "type", any.(int)
    fmt.Println(label, asserted, ok)

    label, received, recvOk := "chan", <-ch
    fmt.Println(label, received, recvOk)
}
