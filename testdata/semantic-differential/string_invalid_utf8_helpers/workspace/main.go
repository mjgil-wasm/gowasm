package main

import (
	"fmt"
	"unicode/utf8"
)

func main() {
	invalid := []byte{0xff, 'x'}
	fmt.Println(utf8.Valid(invalid))
	fmt.Println(utf8.RuneCount(invalid))
	truncated := []byte{0xe2, 0x98}
	fmt.Println(utf8.Valid(truncated))
	fmt.Println(utf8.RuneCount(truncated))
}
