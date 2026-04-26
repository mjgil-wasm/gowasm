package main

import (
    "encoding/json"
    fs "io/fs"
    "os"
    "sort"
)

type ManifestFile struct {
    Name string
    Body string
}

type Manifest struct {
    Root string
    Files []ManifestFile
}

func buildManifest() (string, error) {
    dirfs := os.DirFS("/notes")
    matches, err := fs.Glob(dirfs, "*.txt")
    if err != nil {
        return "", err
    }

    sort.Strings(matches)
    files := make([]ManifestFile, 0, len(matches))
    for _, name := range matches {
        body, err := fs.ReadFile(dirfs, name)
        if err != nil {
            return "", err
        }
        files = append(files, ManifestFile{
            Name: name,
            Body: string(body),
        })
    }

    payload, err := json.MarshalIndent(Manifest{
        Root: "/notes",
        Files: files,
    }, "", "  ")
    if err != nil {
        return "", err
    }
    return string(payload), nil
}
