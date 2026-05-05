package main

import (
	"fmt"
	"path"
)

func main() {
	fmt.Println(path.Base("/a/b/"), path.Clean("a/../../b"), path.Dir("/a/b/"), path.Ext("archive.tar.gz"), path.IsAbs("/a/b/"), path.IsAbs("a/../../b"))
	dir, file := path.Split("../a")
	fmt.Println(dir, file)
	fmt.Println(path.Join("/a", "../b", "c"), path.Join("/a/b", "..", "c"), "["+path.Join("", "", "")+"]")
	escaped, escapedErr := path.Match("a\\*b", "a*b")
	unicode, unicodeErr := path.Match("a?b", "a☺b")
	slashMiss, slashMissErr := path.Match("*", "a/b")
	bad, badErr := path.Match("[", "go")
	fmt.Println(escaped, escapedErr, unicode, unicodeErr, slashMiss, slashMissErr, bad, badErr)
}
