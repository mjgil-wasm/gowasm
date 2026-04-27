package main

import (
	"fmt"
	"math"
)

func main() {
	fmt.Println(math.IsNaN(math.Max(math.NaN(), 1.0)))
	fmt.Println(math.IsNaN(math.Min(1.0, math.NaN())))
	fmt.Println(math.Signbit(math.Max(math.Copysign(0.0, -1.0), 0.0)))
	fmt.Println(math.Signbit(math.Min(math.Copysign(0.0, -1.0), 0.0)))
	fmt.Println(math.Inf(1), math.Inf(-1), math.NaN())
}
