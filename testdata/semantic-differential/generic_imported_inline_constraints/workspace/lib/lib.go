package lib

func Echo[T interface {
	comparable
	int | string
}](value T) T {
	return value
}
