package main

import (
	"fmt"
	"regexp"
)

func main() {
	re := regexp.MustCompile(`(\d+)`)
	commas := regexp.MustCompile(`,+`)

	fmt.Println(re.ReplaceAllString("go1 wasm22 zig333", "<$1> $$"))
	all := commas.Split("go,,wasm,zig", -1)
	limited := commas.Split("go,,wasm,zig", 2)
	fmt.Println(len(all), all[0], all[1], all[2])
	fmt.Println(len(limited), limited[0], limited[1])
}
