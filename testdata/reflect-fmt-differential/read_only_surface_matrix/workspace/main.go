package main

import (
	"fmt"
	"reflect"
)

type Labels []string
type Handler func(string, int) (bool, error)

type Box struct {
	Values []string
	Next   *Box
	Any    interface{}
	Ch     chan int
	Fn     Handler
}

func main() {
	var nilPtr *Box
	var nilMap map[string]int
	var nilSlice []string
	var nilCh chan int
	var labels Labels
	labels = append(labels, "x", "y")
	var any interface{} = labels

	handler := Handler(func(name string, count int) (bool, error) {
		return count > 0, nil
	})

	boxType := reflect.TypeOf(Box{})
	fmt.Println(boxType.NumField())
	fmt.Println(boxType.Field(0).Type.String())
	fmt.Println(boxType.Field(1).Type.String())
	fmt.Println(boxType.Field(2).Type.String())
	fmt.Println(boxType.Field(3).Type.String())
	fmt.Println(boxType.Field(4).Type.String())

	handlerType := reflect.TypeOf(handler)
	fmt.Println(handlerType.Kind() == reflect.Func)
	fmt.Println(handlerType.NumIn(), handlerType.In(0).String(), handlerType.In(1).String())
	fmt.Println(handlerType.NumOut(), handlerType.Out(0).String(), handlerType.Out(1).String())

	channelType := reflect.TypeOf(make(chan int, 2))
	fmt.Println(channelType.Kind() == reflect.Chan)
	fmt.Println(channelType.Elem().String())

	fmt.Println(reflect.ValueOf(nilPtr).IsNil())
	fmt.Println(reflect.ValueOf(nilMap).IsNil())
	fmt.Println(reflect.ValueOf(nilSlice).IsNil())
	fmt.Println(reflect.ValueOf(nilCh).IsNil())

	anyValue := reflect.ValueOf(any)
	fmt.Println(anyValue.Kind() == reflect.Slice)
	fmt.Println(anyValue.Type().String())
	fmt.Println(anyValue.Len())
	fmt.Println(anyValue.Index(1).String())

	var more Labels
	more = append(more, "a", "b")
	fieldAny := reflect.ValueOf(Box{Any: more}).Field(2)
	fmt.Println(fieldAny.Kind() == reflect.Interface)
	fmt.Println(fieldAny.Elem().Type().String())
	fmt.Println(fieldAny.Elem().Len())
	fmt.Println(fieldAny.Elem().Index(0).String())
}
