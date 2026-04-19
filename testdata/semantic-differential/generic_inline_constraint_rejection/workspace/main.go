package main

func Echo[T interface{ int | string }](value T) T {
	return value
}

func main() {
	_ = Echo(true)
}
