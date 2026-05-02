package report

import (
    "encoding/json"

    "example.com/app/catalog"
)

type Summary struct {
    Title string `json:"title"`
    Primary string `json:"primary"`
    Count int `json:"count"`
    Ready bool `json:"ready"`
}

func Build() (string, error) {
    items := catalog.Items()
    payload, err := json.MarshalIndent(Summary{
        Title: catalog.Title(),
        Primary: items[0],
        Count: len(items),
        Ready: catalog.Ready(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
