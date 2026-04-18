package main

import (
	"encoding/hex"
	"fmt"
)

func main() {
	encoded := hex.EncodeToString([]byte{0, 15, 16, 255})
	fmt.Println(encoded)

	decoded, err := hex.DecodeString(encoded)
	fmt.Println(len(decoded), decoded[0], decoded[1], decoded[2], decoded[3], err == nil)

	_, oddErr := hex.DecodeString("0")
	fmt.Println(oddErr)

	_, invalidErr := hex.DecodeString("zz")
	fmt.Println(invalidErr)
}
