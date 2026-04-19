package main

func pair() (int, string) { return 1, "x" }

func main() {
    x := 0
    y := ""
    x, y := pair()
    println(x, y)
}
