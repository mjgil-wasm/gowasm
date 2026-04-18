package main

import "fmt"
import fs "io/fs"
import "time"

type info struct {
    name string
    dir  bool
}

func (i info) Name() string       { return i.name }
func (i info) Size() int64        { return int64(len(i.name)) }
func (i info) Mode() fs.FileMode  { return 0 }
func (i info) ModTime() time.Time { return time.Unix(1, 2) }
func (i info) IsDir() bool        { return i.dir }
func (i info) Sys() interface{}   { return "sys" }

func main() {
    var value interface{} = info{name: "cfg", dir: false}
    fileInfo, ok := value.(fs.FileInfo)
    fmt.Println(fileInfo.Name(), ok)
    switch typed := value.(type) {
    case fs.FileInfo:
        fmt.Println(typed.IsDir())
    default:
        fmt.Println("other")
    }
}
