package main

import (
	"fmt"
	"math"
	"math/bits"
)

func main() {
	fmt.Println(bits.OnesCount(255))
	fmt.Println(bits.LeadingZeros(256))
	fmt.Println(bits.TrailingZeros(12))
	fmt.Println(bits.Len(256))
	fmt.Println(bits.RotateLeft(16, -4))
	fmt.Println(bits.Reverse(6))
	fmt.Println(bits.ReverseBytes(256))
	fmt.Printf("%.1f %.1f\n", math.Round(2.5), math.Round(-2.5))
	fmt.Printf("%.1f\n", math.Remainder(10.0, 3.0))
	fmt.Printf("%.1f\n", math.Float64frombits(math.Float64bits(1.0)))
}
