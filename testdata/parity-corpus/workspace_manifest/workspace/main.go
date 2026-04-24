package main

import "fmt"

func main() {
    manifest, err := buildManifest()
    if err != nil {
        fmt.Println("error:", err)
        return
    }
    fmt.Println(manifest)
}
