package main

import (
	"fmt"
	"net/url"
)

func main() {
	base, _ := url.Parse("https://alice:secret@example.com/a/b/c?q=1#base")

	ref, _ := url.Parse("../d/e?x=2")
	fmt.Println(base.ResolveReference(ref).String())

	absolute, _ := url.Parse("//cdn.example.com//asset/./v1")
	fmt.Println(base.ResolveReference(absolute).String())

	fragment, _ := url.Parse("#next")
	fmt.Println(base.ResolveReference(fragment).String())

	parsed, err := base.Parse("../../g h")
	fmt.Println(err == nil, parsed.String())

	joined := base.JoinPath("c d", "e%2Ff")
	fmt.Println(joined.String())

	result, err := url.JoinPath("https://example.com/base/", "x y", "/z/")
	fmt.Println(result, err == nil)

	opaqueBase, _ := url.Parse("mailto:alice@example.com?")
	opaqueRef, _ := url.Parse("")
	fmt.Println(opaqueBase.ResolveReference(opaqueRef).String())

	_, err = url.JoinPath("bad\nurl", "x")
	fmt.Println(err)
}
