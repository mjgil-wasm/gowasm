package main

import "fmt"
import "io/fs"
import "os"
import "strings"

func labelEntries(entries []fs.DirEntry) string {
	var names []string
	for _, entry := range entries {
		name := entry.Name()
		if entry.IsDir() {
			name += "/"
		}
		names = append(names, name)
	}
	return strings.Join(names, ",")
}

func main() {
	data, err := os.ReadFile("config.txt")
	sub, subErr := fs.Sub(os.DirFS("assets"), "nested")
	nested, readErr := fs.ReadFile(sub, "value.txt")
	entries, dirErr := fs.ReadDir(os.DirFS("assets"), ".")
	fmt.Println(strings.TrimSpace(string(data)), err == nil)
	fmt.Println(subErr == nil, strings.TrimSpace(string(nested)), readErr == nil)
	fmt.Println(fs.ValidPath("nested/value.txt"), fs.ValidPath("../bad"))
	fmt.Println(labelEntries(entries))
	_ = dirErr
}
