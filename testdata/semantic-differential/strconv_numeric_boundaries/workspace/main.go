package main

import (
	"fmt"
	"strconv"
)

func main() {
	value, err := strconv.Atoi("9223372036854775807")
	_, overflowErr := strconv.Atoi("9223372036854775808")
	huge, hugeErr := strconv.ParseFloat("3.5e38", 32)
	fmt.Println(value, err == nil, overflowErr != nil)
	fmt.Printf("%g %t\n", huge, hugeErr != nil)
	fmt.Println(strconv.FormatFloat(1.23456789, 'g', -1, 32))
	fmt.Println(strconv.FormatFloat(1.23456789, 'g', -1, 64))
}
