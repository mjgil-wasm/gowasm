package main

import (
	"fmt"
	"strconv"
)

func main() {
	parseInt := strconv.ParseInt
	parseFloat := strconv.ParseFloat
	atoi := strconv.Atoi

	fmt.Println(strconv.FormatUint(255, 16))

	value, err := strconv.ParseUint("ff", 16, 64)
	fmt.Println(value, err == nil)

	baseZero, baseZeroErr := parseInt("077", 0, 64)
	fmt.Println(baseZero, baseZeroErr)

	floatValue, floatErr := parseFloat("1.5", 16)
	fmt.Println(floatValue, floatErr)

	atoiValue, atoiErr := atoi("63")
	fmt.Println(atoiValue, atoiErr)

	_, parseIntBaseErr := parseInt("10", 1, 64)
	_, parseIntBitsErr := parseInt("10", 10, 65)
	_, parseUintBaseErr := strconv.ParseUint("10", 1, 64)

	fmt.Println(parseIntBaseErr)
	fmt.Println(parseIntBitsErr)
	fmt.Println(parseUintBaseErr)
}
