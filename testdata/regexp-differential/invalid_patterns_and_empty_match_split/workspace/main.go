package main

import (
	"fmt"
	"regexp"
)

func main() {
	matched, err := regexp.MatchString("[invalid", "x")
	fmt.Println(matched, err != nil)

	_, compileErr := regexp.Compile("[invalid")
	fmt.Println(compileErr != nil)

	empty := regexp.MustCompile("")
	fmt.Println(empty.Split("ab", 0) == nil)
	all := empty.Split("ab", -1)
	limited := empty.Split("ab", 4)
	fmt.Println(len(all), all[0], all[1])
	fmt.Println(len(limited), limited[0], limited[1])

	star := regexp.MustCompile("a*")
	parts := star.Split("abaabaccadaaae", 5)
	fmt.Println(len(parts), parts[0], parts[1], parts[2], parts[3], parts[4])
}
