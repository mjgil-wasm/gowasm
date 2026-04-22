package main

type Box[T any] struct {
    Values []T
}

func Last[T any](values []T) T {
    return values[len(values)-1]
}

func CountAndLast[T any](values []T) (int, T) {
    return len(values), Last(values)
}
