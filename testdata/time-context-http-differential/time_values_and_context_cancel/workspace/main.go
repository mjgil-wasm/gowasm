package main

import (
	"context"
	"fmt"
	"time"
)

func main() {
	earlier := time.UnixMicro(1234567)
	later := earlier.Add(1500 * time.Millisecond)
	fmt.Println(earlier.UnixNano(), later.Sub(earlier).Milliseconds(), earlier.Before(later), later.After(earlier))

	parent, cancel := context.WithCancel(context.Background())
	ctx := context.WithValue(parent, "k", "v")
	fmt.Println(ctx.Value("k"))
	cancel()
	<-ctx.Done()
	fmt.Println(ctx.Err() == context.Canceled, ctx.Err().Error())
	deadline, ok := ctx.Deadline()
	fmt.Println(ok, deadline.IsZero())
}
