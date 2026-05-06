package shared

import "fmt"

func init() {
	fmt.Println("shared")
}

func Message() string {
	return "ready"
}
