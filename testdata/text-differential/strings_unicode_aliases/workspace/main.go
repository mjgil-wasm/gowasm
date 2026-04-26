package main

import (
	"fmt"
	"strings"
	"unicode"
)

type eqFoldFn func(string, string) bool

func main() {
	var fold eqFoldFn = strings.EqualFold
	fmt.Println(
		fold("Go", "go"),
		fold("Σ", "ς"),
		strings.SplitN("go-wasm-zig", "-", 2),
		strings.SplitAfterN("go-wasm-zig", "-", 2),
		strings.ToTitle("héLlö"),
	)

	fmt.Println(
		unicode.IsSpace(8233),
		unicode.IsGraphic(955),
		unicode.ToUpper(955),
		unicode.SimpleFold('K'),
		unicode.ToTitle('ǆ'),
	)
}
