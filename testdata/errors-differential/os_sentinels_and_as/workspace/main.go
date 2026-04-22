package main

import (
	"errors"
	"fmt"
	"os"
)

func main() {
	_, missingErr := os.ReadFile("missing")
	wrapped := fmt.Errorf("wrap: %w", missingErr)
	syscallErr := os.NewSyscallError("open", missingErr)

	var target error

	fmt.Println(
		errors.Is(missingErr, os.ErrNotExist),
		errors.Is(wrapped, os.ErrNotExist),
		errors.Is(syscallErr, os.ErrNotExist),
	)
	fmt.Println(
		errors.Unwrap(missingErr) == os.ErrNotExist,
		errors.Unwrap(syscallErr) == missingErr,
	)
	fmt.Println(errors.As(wrapped, &target), errors.Is(target, os.ErrNotExist))
}
