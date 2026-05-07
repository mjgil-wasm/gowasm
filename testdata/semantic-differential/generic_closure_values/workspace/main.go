package main

import "fmt"

type Holder struct {
	Run func(int) int
	Any interface{}
}

func MakeAdder[T any](base T, combine func(T, T) T) func(T) T {
	return func(next T) T {
		return combine(base, next)
	}
}

func main() {
	values := map[string]func(int) int{
		"double": func(v int) int { return v * 2 },
	}
	holder := Holder{
		Run: values["double"],
		Any: MakeAdder[int](40, func(left int, right int) int {
			return left + right
		}),
	}
	fmt.Println(holder.Run(21))
	add := MakeAdder[int](40, func(left int, right int) int {
		return left + right
	})
	fmt.Println(add(2))
	fmt.Println(holder.Any != nil)
}
