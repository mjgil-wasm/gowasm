package main

import "fmt"

func main() {
	runes := []rune("hé世")
	fmt.Println(len(runes))
	fmt.Println(runes[0], runes[1], runes[2])
	fmt.Printf("%q\n", string([]rune{9731, 233, 65}))
	bytes := []byte("hé")
	fmt.Println(len(bytes))
	fmt.Println(bytes[0], bytes[1], bytes[2])
}
