package main

func only() int { return 1 }

func main() {
    first, second := only()
    println(first, second)
}
