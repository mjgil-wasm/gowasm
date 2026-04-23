package main

import "fmt"

func main() {
outer:
	for i := 0; i < 3; i++ {
		for j := 0; j < 3; j++ {
			switch j {
			case 0:
				fmt.Println("keep", i, j)
				continue
			case 1:
				continue outer
			default:
				fmt.Println("bad")
			}
		}
	}
	fmt.Println("done")
}
