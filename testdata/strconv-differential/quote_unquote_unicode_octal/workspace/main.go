package main

import (
	"fmt"
	"strconv"
)

func main() {
	fmt.Println(strconv.Quote(string([]byte{7, 10}) + "世"))
	fmt.Println(strconv.QuoteToASCII("世\n"))
	fmt.Println(strconv.Quote(string([]byte{0, 34, 92})))

	text, textErr := strconv.Unquote("\"\\141\\n\"")
	value, multibyte, tail, err := strconv.UnquoteChar("\\377x", '"')
	unicodeValue, unicodeMultibyte, unicodeTail, unicodeErr := strconv.UnquoteChar("\\u03bbx", '"')

	fmt.Println(text, textErr)
	fmt.Println(value, multibyte, tail, err)
	fmt.Println(unicodeValue, unicodeMultibyte, unicodeTail, unicodeErr)
}
