package left

import (
	"fmt"
	"example.com/app/shared"
)

func init() {
	fmt.Println("left")
}

func Message() string {
	return "left-" + shared.Message()
}
