package main

import (
	"bytes"
	"fmt"
)

func main() {
	data := []byte{255, 'A', 195, '(', 'b'}
	fmt.Println(bytes.Fields([]byte{255, ' ', 'a', ' ', 192}))
	fmt.Println(bytes.Trim([]byte{255, 'a', 192}, "�"))
	fmt.Println(bytes.TrimLeft([]byte{255, 'a', 192}, "�"))
	fmt.Println(bytes.TrimRight([]byte{255, 'a', 192}, "�"))
	fmt.Println(bytes.IndexAny(data, "A�b"))
	fmt.Println(bytes.LastIndexAny(data, "(�"))
	fmt.Println(bytes.IndexRune(data, 65533))
	fmt.Println(bytes.Count(data, []byte{}))
	fmt.Println(bytes.Replace(data, []byte{}, []byte{'-'}, 2))
	fmt.Println(bytes.ReplaceAll(data, []byte{}, []byte{'-'}))
	fmt.Println(bytes.Split(data, []byte{}))
	fmt.Println(bytes.SplitN(data, []byte{}, 2))
	fmt.Println(bytes.SplitAfter(data, []byte{}))
	fmt.Println(bytes.TrimSpace([]byte{255, ' ', 'a', ' ', 192}))
	fmt.Println(bytes.EqualFold([]byte{255, 'A', 192, 'b'}, []byte{255, 'a', 192, 'B'}))
}
