package main

import (
    "encoding/json"
    "net/http"
    "os"
    "path/filepath"
    "time"
)

type ProfileSummary struct {
    Banner string
    Status string
    Body string
    FetchedAt int64
}

func buildSummary() (string, error) {
    os.Setenv("HOME", "/users/alice")
    configRoot, err := os.UserConfigDir()
    if err != nil {
        return "", err
    }

    bannerPath := filepath.Join(configRoot, "demo", "banner.txt")
    banner, err := os.ReadFile(bannerPath)
    if err != nil {
        return "", err
    }

    resp, err := http.Get("https://example.com/profile")
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    buf := make([]byte, 32)
    n, _ := resp.Body.Read(buf)
    payload, err := json.MarshalIndent(ProfileSummary{
        Banner: string(banner),
        Status: resp.Status,
        Body: string(buf[:n]),
        FetchedAt: time.Now().UnixMilli(),
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
