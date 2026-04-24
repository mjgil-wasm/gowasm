package main

import (
	"fmt"
	"net/url"
)

func main() {
	parsed, err := url.Parse("https://example.com/a%2Fb?q=1#frag%2Fpart")
	fmt.Println(err == nil, parsed.Path, parsed.RawPath, parsed.RawQuery, parsed.Fragment, parsed.RawFragment)
	fmt.Println(parsed.EscapedPath(), parsed.EscapedFragment(), parsed.RequestURI())

	fmt.Println(url.QueryEscape("space key/x+y?=z&"))
	decoded, err := url.QueryUnescape("space+key%2Fx%2By%3F%3Dz%26")
	fmt.Println(decoded, err == nil)

	fmt.Println(url.PathEscape("my/cool+blog&about,stuff"))
	decoded, err = url.PathUnescape("my%2Fcool+blog&about%2Cstuff")
	fmt.Println(decoded, err == nil)

	_, err = url.Parse("bad\nurl")
	fmt.Println(err)
	_, err = url.PathUnescape("%zz")
	fmt.Println(err)
	_, err = url.QueryUnescape("%zz")
	fmt.Println(err)
}
