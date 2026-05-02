package main

import "fmt"

func main() {
	label := "outer"
	switch value := 1; value {
	case 1:
		label := "inner"
		fmt.Println(label, value)
	}
	switch value := 0; value {
	case 2:
		fmt.Println("two")
	default:
		fmt.Println("default", value)
		fallthrough
	case 3:
		after := value + 3
		fmt.Println("after", after)
	}
	fmt.Println(label)
}
