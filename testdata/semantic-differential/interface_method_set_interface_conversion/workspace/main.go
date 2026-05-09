package main

import "fmt"
import fs "io/fs"
import "time"

type Reader[T any] interface {
    Value() T
}

type Named interface {
    String() string
}

type Wider interface {
    Named
    Label() string
}

type Label struct {
    value string
}

func (label Label) String() string {
    return "label:" + label.value
}

func (label Label) Label() string {
    return label.value
}

type info struct {
    name string
    dir bool
}

func (i info) Name() string { return i.name }
func (i info) Size() int64 { return int64(len(i.name)) }
func (i info) Mode() fs.FileMode { return 0 }
func (i info) ModTime() time.Time { return time.Unix(1, 2) }
func (i info) IsDir() bool { return i.dir }
func (i info) Sys() interface{} { return "sys" }

type Box struct {
    value int
}

func (box Box) Value() int {
    return box.value
}

func main() {
    var wider Wider = Label{value: "ada"}
    var named Named = wider
    var fileInfo fs.FileInfo = info{name: "ada", dir: false}
    var any interface{} = Box{value: 9}
    reader, ok := any.(Reader[int])
    fmt.Println(named.String(), fileInfo.Name(), ok, reader.Value())
}
