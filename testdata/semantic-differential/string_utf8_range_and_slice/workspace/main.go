package main

import "fmt"

func main() {
	s := "Go☃界"
	fmt.Println(len(s), s[2], s[3], s[4], s[5], s[6], s[7])
	fmt.Printf("%q\n", s[2:5])
	for index, value := range "世a界" {
		fmt.Println(index, value)
	}
}
