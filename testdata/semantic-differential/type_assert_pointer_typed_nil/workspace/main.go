package main

import "fmt"

type Any interface{}
type Box struct{ value int }

func main() {
    var ptr *Box
    var value Any = ptr
    typed, ok := value.(*Box)
    fmt.Println(ok, typed == nil)
}
