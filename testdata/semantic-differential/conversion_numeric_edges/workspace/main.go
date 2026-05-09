package main

import "fmt"

type Octet byte

func main() {
	x := 257
	y := -1
	z := 258.75
	fmt.Println(byte(x), byte(y), byte(z))
	fmt.Printf("%q %q\n", string(rune(9731)), string([]byte{226, 152, 131}))
	fmt.Println(Octet(x))
}
