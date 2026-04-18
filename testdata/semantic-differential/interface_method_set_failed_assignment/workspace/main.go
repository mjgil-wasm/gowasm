package main

type Needs interface {
    Read(value int) int
}

type Wrong struct{}

func (Wrong) Read(text string) int {
    return 1
}

func main() {
    var value Needs = Wrong{}
    _ = value
}
