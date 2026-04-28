package main

import "fmt"

func pair() (int, string) { return 7, "go" }

func main() {
    values := map[string]string{"hit": "ok"}
    word := "outer"
    word, found := values["hit"]
    fmt.Println(word, found)

    var any interface{} = 9
    count := 0
    count, ok := any.(int)
    fmt.Println(count, ok)

    ch := make(chan int, 1)
    ch <- 5
    close(ch)
    count, recvOk := <-ch
    fmt.Println(count, recvOk)

    count, label := pair()
    fmt.Println(count, label)
}
