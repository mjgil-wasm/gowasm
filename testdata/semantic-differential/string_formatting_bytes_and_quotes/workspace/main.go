package main

import "fmt"

func main() {
	s := "hé"
	fmt.Printf("%q\n", s)
	fmt.Printf("%x\n", s)
	fmt.Printf("%q\n", string([]rune{9731}))
}
