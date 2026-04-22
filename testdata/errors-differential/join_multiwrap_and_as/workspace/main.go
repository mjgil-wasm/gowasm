package main

import (
	"errors"
	"fmt"
)

type Problem struct {
	Code int
}

func (p Problem) Error() string {
	return fmt.Sprintf("problem:%d", p.Code)
}

func main() {
	first := errors.New("first")
	second := errors.New("second")
	joined := errors.Join(first, nil, second)
	multi := fmt.Errorf("multi: %w + %w", first, second)
	wrapped := fmt.Errorf("outer: %w", Problem{Code: 7})

	var problem Problem
	var top error

	fmt.Println(joined)
	fmt.Println(errors.Is(joined, first), errors.Is(joined, second), errors.Unwrap(joined) == nil)
	fmt.Println(multi)
	fmt.Println(errors.Is(multi, first), errors.Is(multi, second), errors.Unwrap(multi) == nil)
	fmt.Println(errors.As(wrapped, &problem), problem.Code)
	fmt.Println(errors.As(wrapped, &top), top)
}
