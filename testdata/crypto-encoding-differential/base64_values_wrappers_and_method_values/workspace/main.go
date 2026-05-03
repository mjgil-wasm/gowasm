package main

import (
	"encoding/base64"
	"fmt"
)

func main() {
	encode := base64.StdEncoding.EncodeToString

	fmt.Println(encode([]byte("Hello")))

	decoded, err := base64.RawURLEncoding.DecodeString("-_8")
	fmt.Println(len(decoded), decoded[0], decoded[1], err == nil)

	urlDecoded, urlErr := base64.URLEncoding.DecodeString("-_8=")
	fmt.Println(len(urlDecoded), urlDecoded[0], urlDecoded[1], urlErr == nil)

	fmt.Println(base64.StdEncoding.EncodeToString([]byte{}))
	emptyDecoded, emptyErr := base64.StdEncoding.DecodeString("")
	fmt.Println(len(emptyDecoded), emptyErr == nil)

	_, invalidErr := base64.StdEncoding.DecodeString("@@==")
	fmt.Println(invalidErr)
}
