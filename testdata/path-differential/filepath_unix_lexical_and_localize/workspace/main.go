package main

import (
	"fmt"
	"path/filepath"
	"strings"
)

func main() {
	fmt.Println(filepath.Base("/a/b/"), filepath.Clean("a/../../b"), filepath.Dir("/a/b/"), filepath.Ext("archive.tar.gz"), filepath.IsAbs("/a/b/"), filepath.IsAbs("a/../../b"))
	dir, file := filepath.Split("../a")
	fmt.Println(dir, file)
	fmt.Println("["+filepath.Join()+"]", filepath.Join("", "a", "b"), filepath.Join("/a", "../b", "c"))
	same, sameErr := filepath.Rel("a/b", "a/b")
	up, upErr := filepath.Rel("a/b/c", "a/d")
	bad, badErr := filepath.Rel("../a", "b")
	local, localErr := filepath.Localize("a/b")
	dot, dotErr := filepath.Localize(".")
	badLocal, badLocalErr := filepath.Localize("../a")
	fmt.Println(same, sameErr)
	fmt.Println(up, upErr)
	fmt.Println(bad == "", badErr.Error())
	fmt.Println(filepath.IsLocal("a/b"), filepath.IsLocal("./a"), filepath.IsLocal("."), filepath.IsLocal("../a"), filepath.IsLocal("/a"), filepath.IsLocal(""))
	fmt.Println(local, localErr)
	fmt.Println(dot, dotErr)
	fmt.Println(badLocal == "", badLocalErr.Error())
	fmt.Println(filepath.ToSlash("a/b/c"), filepath.FromSlash("a/b/c"), strings.Join(filepath.SplitList("a:b::c"), ","), len(filepath.SplitList("")), "["+filepath.VolumeName("/a/b")+"]")
}
