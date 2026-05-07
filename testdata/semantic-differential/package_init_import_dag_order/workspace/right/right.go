package right

import (
	"fmt"
	"example.com/app/shared"
)

func init() {
	fmt.Println("right")
}

func Message() string {
	return "right-" + shared.Message()
}
