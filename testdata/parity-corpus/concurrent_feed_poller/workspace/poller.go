package main

import (
    "encoding/json"
    "net/http"
    "sort"
    "sync"
)

type FeedResult struct {
    Name string
    Status string
    Body string
}

type FeedReport struct {
    Count int
    Results []FeedResult
}

func runFeedPoller() (string, error) {
    results := make([]FeedResult, 0, 2)
    firstDone := make(chan bool, 1)
    errCh := make(chan error, 2)
    var mu sync.Mutex
    var wg sync.WaitGroup

    wg.Add(2)
    go func() {
        defer wg.Done()
        defer func() {
            firstDone <- true
        }()
        resp, err := http.Get("https://example.com/feed-alpha")
        if err != nil {
            errCh <- err
            return
        }
        defer resp.Body.Close()

        buf := make([]byte, 32)
        n, _ := resp.Body.Read(buf)

        mu.Lock()
        results = append(results, FeedResult{
            Name: "alpha",
            Status: resp.Status,
            Body: string(buf[:n]),
        })
        mu.Unlock()
    }()

    go func() {
        defer wg.Done()
        <-firstDone

        resp, err := http.Get("https://example.com/feed-beta")
        if err != nil {
            errCh <- err
            return
        }
        defer resp.Body.Close()

        buf := make([]byte, 32)
        n, _ := resp.Body.Read(buf)

        mu.Lock()
        results = append(results, FeedResult{
            Name: "beta",
            Status: resp.Status,
            Body: string(buf[:n]),
        })
        mu.Unlock()
    }()

    wg.Wait()
    select {
    case err := <-errCh:
        return "", err
    default:
    }
    sort.Slice(results, func(i int, j int) bool {
        return results[i].Name < results[j].Name
    })

    payload, err := json.MarshalIndent(FeedReport{
        Count: len(results),
        Results: results,
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
