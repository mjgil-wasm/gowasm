package main

import (
	"fmt"
	"net/url"
)

func main() {
	parsed, _ := url.Parse("https://alice%40example.com:p%40ss%3A%2F%3F@example.com/path")
	redacted := parsed.Redacted
	username := parsed.User.Username
	password := parsed.User.Password

	fmt.Println(username())
	pass, ok := password()
	fmt.Println(pass, ok)
	fmt.Println(parsed.String())
	fmt.Println(redacted())

	values := make(url.Values)
	values.Add("z", "last")
	values.Add("a", "1")
	values.Add("a", "two words")
	values.Set("space key", "x+y")

	encode := values.Encode
	has := values.Has
	fmt.Println(encode())
	fmt.Println(values.Get("a"), has("space key"), has("missing"))
	values.Del("z")
	fmt.Println(encode())
}
