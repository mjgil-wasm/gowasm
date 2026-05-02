package main

import (
	"bytes"
	"fmt"
	"unicode"
)

func main() {
	data := []byte{255, 'A', 192, '(', 'b'}
	fmt.Println(bytes.IndexFunc(data, unicode.IsGraphic))
	fmt.Println(bytes.LastIndexFunc(data, unicode.IsGraphic))
	fmt.Println(bytes.TrimFunc(data, unicode.IsGraphic))
	fmt.Println(bytes.FieldsFunc([]byte{255, ' ', 'a', ' ', 192}, unicode.IsSpace))
	fmt.Println(bytes.Map(unicode.ToUpper, data))

	fmt.Println(bytes.SplitAfterN([]byte("a,b,c"), []byte(","), 2))

	prefix, okPrefix := bytes.CutPrefix([]byte("prefix-body"), []byte("prefix-"))
	suffix, okSuffix := bytes.CutSuffix([]byte("body-suffix"), []byte("-suffix"))
	before, after, okCut := bytes.Cut([]byte("left=right"), []byte("="))
	fmt.Println(prefix, okPrefix)
	fmt.Println(suffix, okSuffix)
	fmt.Println(before, after, okCut)
}
