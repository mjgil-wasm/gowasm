package main

import (
    "encoding/json"
    "fmt"
)

type Meta struct {
    Name  string
    Count int
}

type MetaList []*Meta
type MetaPair [2]*Meta
type Lookup map[string]*Meta

type Payload struct {
    Items  MetaList
    Pair   MetaPair
    Lookup Lookup
}

type Box[T any] struct {
    Value T
}

func main() {
    payload := Payload{
        Items: []*Meta{&Meta{Name: "stale", Count: 9}},
        Pair: [2]*Meta{
            &Meta{Name: "stale", Count: 9},
            &Meta{Name: "keep", Count: 7},
        },
        Lookup: map[string]*Meta{
            "x":    &Meta{Name: "stale", Count: 9},
            "keep": &Meta{Name: "keep", Count: 7},
        },
    }
    err := json.Unmarshal(
        []byte(`{"Items":[{"Name":"Ada"}],"Pair":[{"Name":"Ada"}],"Lookup":{"x":{"Name":"Ada"}}}`),
        &payload,
    )
    fmt.Println(
        payload.Items[0].Name,
        payload.Items[0].Count,
        payload.Pair[0].Name,
        payload.Pair[0].Count,
        payload.Pair[1] == nil,
        payload.Lookup["x"].Name,
        payload.Lookup["x"].Count,
        payload.Lookup["keep"].Name,
        payload.Lookup["keep"].Count,
        err,
    )

    var generic Box[[]*Meta]
    generic.Value = []*Meta{&Meta{Name: "stale", Count: 9}}
    genericErr := json.Unmarshal([]byte(`{"Value":[{"Name":"Ada"}]}`), &generic)
    fmt.Println(generic.Value[0].Name, generic.Value[0].Count, genericErr)
}
