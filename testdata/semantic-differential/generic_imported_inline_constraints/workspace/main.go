package main

import (
	"example.com/app/lib"
	"fmt"
)

func main() {
	fmt.Println(lib.Echo[string]("imported"))
}
