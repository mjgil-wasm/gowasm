package main

import (
	"fmt"
	"regexp"
)

func main() {
	re := regexp.MustCompile(`(\pL+)-(\d+)`)
	finder := re.FindStringSubmatch
	replacer := regexp.MustCompile(`(\d+)`).ReplaceAllString

	matched, err := regexp.MatchString(`^\pL+-\d+$`, "東京-34")
	fmt.Println(matched, err == nil)
	fmt.Println(finder("xx cafe-12 yy"))
	fmt.Println(finder("xx 東京-34 yy"))
	fmt.Println(replacer("go1 wasm22 zig333", "<$1>"))
}
