package main

import (
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha256"
	"crypto/sha512"
	"fmt"
)

func main() {
	md5Hello := md5.Sum([]byte("hello"))
	md5Empty := md5.Sum([]byte{})
	sha1Hello := sha1.Sum([]byte("hello"))
	sha1Empty := sha1.Sum([]byte{})
	sha256Hello := sha256.Sum256([]byte("hello"))
	sha256Empty := sha256.Sum256([]byte{})
	sha512Hello := sha512.Sum512([]byte("hello"))
	sha512Empty := sha512.Sum512([]byte{})

	fmt.Println(md5Hello[0], md5Hello[1], md5Hello[2], md5Hello[3], md5Hello[12], md5Hello[13], md5Hello[14], md5Hello[15])
	fmt.Println(md5Empty[0], md5Empty[1], md5Empty[2], md5Empty[3], md5Empty[12], md5Empty[13], md5Empty[14], md5Empty[15])
	fmt.Println(sha1Hello[0], sha1Hello[1], sha1Hello[2], sha1Hello[3], sha1Hello[16], sha1Hello[17], sha1Hello[18], sha1Hello[19])
	fmt.Println(sha1Empty[0], sha1Empty[1], sha1Empty[2], sha1Empty[3], sha1Empty[16], sha1Empty[17], sha1Empty[18], sha1Empty[19])
	fmt.Println(sha256Hello[0], sha256Hello[1], sha256Hello[2], sha256Hello[3], sha256Hello[28], sha256Hello[29], sha256Hello[30], sha256Hello[31])
	fmt.Println(sha256Empty[0], sha256Empty[1], sha256Empty[2], sha256Empty[3], sha256Empty[28], sha256Empty[29], sha256Empty[30], sha256Empty[31])
	fmt.Println(sha512Hello[0], sha512Hello[1], sha512Hello[2], sha512Hello[3], sha512Hello[60], sha512Hello[61], sha512Hello[62], sha512Hello[63])
	fmt.Println(sha512Empty[0], sha512Empty[1], sha512Empty[2], sha512Empty[3], sha512Empty[60], sha512Empty[61], sha512Empty[62], sha512Empty[63])
	fmt.Println(md5.BlockSize, md5.Size)
	fmt.Println(sha1.BlockSize, sha1.Size)
	fmt.Println(sha256.BlockSize, sha256.Size)
	fmt.Println(sha512.BlockSize, sha512.Size)
}
