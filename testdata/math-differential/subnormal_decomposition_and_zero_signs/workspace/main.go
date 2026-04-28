package main

import (
	"fmt"
	"math"
)

func main() {
	frac, exp := math.Frexp(math.SmallestNonzeroFloat64)
	fmt.Printf("%.1f %d\n", frac, exp)
	fmt.Printf("%.1f\n", math.Logb(math.SmallestNonzeroFloat64))
	fmt.Println(math.Ilogb(math.SmallestNonzeroFloat64))
	fracInf, expInf := math.Frexp(math.Inf(1))
	fmt.Println(fracInf, expInf)
	fracNaN, expNaN := math.Frexp(math.NaN())
	fmt.Println(math.IsNaN(fracNaN), expNaN)
	i, f := math.Modf(math.Copysign(0.0, -1.0))
	fmt.Println(math.Signbit(i), math.Signbit(f))
}
