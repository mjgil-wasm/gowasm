package main

import (
	"encoding/json"
	"fmt"
)

type Payload struct{}

func (Payload) MarshalJSON() ([]byte, error) {
	return []byte(`{"kind":"payload"}`), nil
}

type Label string

func (label Label) MarshalText() ([]byte, error) {
	return []byte("label<" + string(label) + ">"), nil
}

type Both string

func (Both) MarshalJSON() ([]byte, error) {
	return []byte(`"json"`), nil
}

func (Both) MarshalText() ([]byte, error) {
	return []byte("text"), nil
}

type Wrapper struct {
	Label Label
	Both  Both
}

type Node struct{}

func (*Node) MarshalJSON() ([]byte, error) {
	return []byte(`"unexpected"`), nil
}

func main() {
	payload, payloadErr := json.Marshal(Payload{})
	wrapper, wrapperErr := json.Marshal(Wrapper{Label: Label("go"), Both: Both("x")})
	var node *Node
	nodeBytes, nodeErr := json.Marshal(node)

	fmt.Println(string(payload), payloadErr)
	fmt.Println(string(wrapper), wrapperErr)
	fmt.Println(string(nodeBytes), nodeErr)
}
