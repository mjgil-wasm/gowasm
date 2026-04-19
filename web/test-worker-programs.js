export function helloWorldProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import "fmt"
func main() { fmt.Println("hello") }
`,
      },
    ],
  };
}

export function formatProgram(source) {
  return {
    kind: "format",
    files: [
      {
        path: "main.go",
        contents: source,
      },
    ],
  };
}

export function lintProgram(source) {
  return {
    kind: "lint",
    files: [
      {
        path: "main.go",
        contents: source,
      },
    ],
  };
}

export function snippetTestProgram(source) {
  return {
    kind: "test_snippet",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: source,
      },
    ],
  };
}

export function packageTestProgram(filter = null) {
  const request = {
    kind: "test_package",
    target_path: "calc.go",
    files: [
      {
        path: "calc.go",
        contents: `package calc

func Add(left int, right int) int {
  return left + right
}
`,
      },
      {
        path: "calc_test.go",
        contents: `package calc

func TestAdd() {
  if Add(2, 3) != 5 {
    panic("expected Add to sum inputs")
  }
}
`,
      },
    ],
  };
  if (filter) {
    request.filter = filter;
  }
  return request;
}

export function packageTestSleepProgram(durationMs, filter = null) {
  const request = {
    kind: "test_package",
    target_path: "calc.go",
    files: [
      {
        path: "calc.go",
        contents: `package calc

import "time"

func Add(left int, right int) int {
  time.Sleep(${durationMs * 1_000_000})
  return left + right
}
`,
      },
      {
        path: "calc_test.go",
        contents: `package calc

func TestAdd() {
  if Add(2, 3) != 5 {
    panic("expected Add to sum inputs")
  }
}
`,
      },
    ],
  };
  if (filter) {
    request.filter = filter;
  }
  return request;
}

export function sleepProgram(durationMs) {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "time"
)
func main() {
  time.Sleep(${durationMs * 1_000_000})
  fmt.Println("done")
}
`,
      },
    ],
  };
}

export function netHttpSlowProgram(url) {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "net/http"
)
func main() {
  resp, err := http.Get(${JSON.stringify(url)})
  if err != nil {
    fmt.Println("fetch-failed", err)
    return
  }
  defer resp.Body.Close()
  buf := make([]byte, 64)
  n, err := resp.Body.Read(buf)
  if err != nil && err.Error() != "EOF" {
    fmt.Println("read-failed", err)
    return
  }
  fmt.Println(resp.StatusCode, string(buf[:n]))
}
`,
      },
    ],
  };
}

export function busyLoopProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import "fmt"
func main() {
  sum := 0
  for i := 0; i < 100000000; i++ {
    sum += i
  }
  fmt.Println(sum)
}
`,
      },
    ],
  };
}

export function runtimeFaultProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main

func explode() {
  value := 0
  _ = 1 / value
}

func main() {
  explode()
}
`,
      },
    ],
  };
}

export function timeCapabilityProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "time"
)
func main() {
  before := time.Now().UnixMilli()
  time.Sleep(5 * time.Millisecond)
  after := time.Now().UnixMilli()
  fmt.Println(before > 0, after >= before)
}
`,
      },
    ],
  };
}

export function netHttpCapabilityProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "net/http"
)
func main() {
  resp, err := http.Get("data:text/plain,hello")
  if err != nil {
    fmt.Println("fetch-failed", err)
    return
  }
  buf := make([]byte, 5)
  n, _ := resp.Body.Read(buf)
  closeErr := resp.Body.Close()
  fmt.Println(resp.StatusCode, string(buf[:n]), closeErr == nil)
}
`,
      },
    ],
  };
}

export function netHttpFailureProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "net/http"
)
func main() {
  resp, err := http.Get("gowasm-test://fetch-failure")
  fmt.Println(resp == nil, err != nil)
}
`,
      },
    ],
  };
}

export function ioFsCapabilityProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import "fmt"
import "io/fs"
import "os"
import "strings"
func main() {
  root := os.DirFS("assets")
  data, err := fs.ReadFile(root, "config.txt")
  info, statErr := fs.Stat(root, "config.txt")
  entries, readDirErr := fs.ReadDir(root, ".")
  matches, globErr := fs.Glob(root, "*.txt")
  sub, subErr := fs.Sub(root, "nested")
  child, childErr := fs.ReadFile(sub, "child.txt")
  walked := make([]string, 0, 4)
  walkErr := fs.WalkDir(root, ".", func(path string, d fs.DirEntry, err error) error {
    if err != nil {
      return err
    }
    walked = append(walked, path)
    return nil
  })

  file, openErr := root.Open("config.txt")
  buf := make([]byte, 2)
  n, readErr := file.Read(buf)
  closeErr := file.Close()
  closedInfo, closedErr := file.Stat()

  fmt.Println(string(data), err == nil, string(child), subErr == nil && childErr == nil)
  fmt.Println(
    info.Size(),
    info.Mode().IsRegular(),
    statErr == nil,
    len(entries),
    entries[0].Name(),
    entries[1].Name(),
    readDirErr == nil,
  )
  fmt.Println(
    len(matches),
    matches[0],
    globErr == nil,
    strings.Join(walked, ","),
    walkErr == nil,
  )
  fmt.Println(
    openErr == nil,
    n,
    readErr == nil,
    closeErr == nil,
    closedInfo == nil,
    closedErr != nil,
  )
}
`,
      },
      {
        path: "assets/config.txt",
        contents: "alpha",
      },
      {
        path: "assets/nested/child.txt",
        contents: "child",
      },
    ],
  };
}

export function osCapabilityProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import (
  "fmt"
  "os"
)
func main() {
  mkdirErr := os.MkdirAll("/tmpdir/sub", 493)
  writeErr := os.WriteFile("/tmpdir/out.txt", []byte("beta"), 420)
  data, err := os.ReadFile("/tmpdir/out.txt")
  entries, readErr := os.ReadDir("/tmpdir")
  missingErr := os.WriteFile("/missing/out.txt", []byte("x"), 420)
  fmt.Println(string(data), err == nil && mkdirErr == nil && writeErr == nil)
  fmt.Println(len(entries), entries[0].Name(), entries[1].Name(), readErr == nil)
  fmt.Println(missingErr != nil)
}
`,
      },
    ],
  };
}
