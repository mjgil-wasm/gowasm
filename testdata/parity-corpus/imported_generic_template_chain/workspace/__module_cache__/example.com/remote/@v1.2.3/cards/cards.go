package cards

type Box[T any] struct {
    Label T
}

func (box Box[T]) Speak() string {
    return "remote-template"
}
