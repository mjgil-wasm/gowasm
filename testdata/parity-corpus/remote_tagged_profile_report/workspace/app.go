package main

import (
    "encoding/json"
    "fmt"
    "net/http"
    "reflect"
)

type RemoteProfile struct {
    Name string `json:"name"`
    Hidden string `json:"-"`
    Alias string `json:"alias_name"`
}

type Report struct {
    Summary string `json:"summary"`
    FieldTag string `json:"field_tag"`
    Alias string `json:"alias,omitempty"`
}

func buildReport() (string, error) {
    resp, err := http.Get("https://example.com/profile.json")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 64)
    n, _ := resp.Body.Read(buf)

    profile := RemoteProfile{Hidden: "keep"}
    if err := json.Unmarshal(buf[:n], &profile); err != nil {
        return "", err
    }

    payload, err := json.MarshalIndent(Report{
        Summary: fmt.Sprintf("%+v", profile),
        FieldTag: string(reflect.TypeOf(profile).Field(0).Tag),
        Alias: reflect.ValueOf(profile).Field(2).String(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
