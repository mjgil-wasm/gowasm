package main

import (
    "encoding/json"
    "net/http"
)

type RemoteItem struct {
    Name string `json:"name"`
    Count int `json:"count"`
}

type RemoteSnapshot struct {
    Title string `json:"title"`
    Items []RemoteItem `json:"items"`
    Scores map[string]int `json:"scores"`
}

type SnapshotReport struct {
    Title string `json:"title"`
    First string `json:"first"`
    SecondCount int `json:"second_count"`
    Alpha int `json:"alpha"`
    Beta int `json:"beta"`
    ItemCount int `json:"item_count"`
}

func buildCollectionReport() (string, error) {
    resp, err := http.Get("https://example.com/collections.json")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 160)
    n, _ := resp.Body.Read(buf)

    var snapshot RemoteSnapshot
    if err := json.Unmarshal(buf[:n], &snapshot); err != nil {
        return "", err
    }

    payload, err := json.MarshalIndent(SnapshotReport{
        Title: snapshot.Title,
        First: snapshot.Items[0].Name,
        SecondCount: snapshot.Items[1].Count,
        Alpha: snapshot.Scores["alpha"],
        Beta: snapshot.Scores["beta"],
        ItemCount: len(snapshot.Items),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
