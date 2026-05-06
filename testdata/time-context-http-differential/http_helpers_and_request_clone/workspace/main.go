package main

import (
	"fmt"
	"net/http"
	"time"
)

func main() {
	parsed, err := http.ParseTime("Sun, 06 Nov 1994 08:49:37 GMT")
	fmt.Println(err == nil, parsed.Format(time.DateTime))

	req, _ := http.NewRequest(http.MethodPost, "https://example.com/api?q=1", nil)
	req.Header.Set("accept", "text/plain")
	req.Header.Add("ACCEPT", "application/json")
	clone := req.Clone(req.Context())
	clone.Header.Add("Accept", "text/html")

	fmt.Println(req.Method, req.URL.String())
	fmt.Println(req.Header.Get("Accept"))
	fmt.Println(len(req.Header.Values("Accept")), len(clone.Header.Values("Accept")))
	fmt.Println(http.CanonicalHeaderKey("x-test"), http.StatusText(http.StatusTeapot))
}
