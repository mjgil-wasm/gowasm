export const negativeSupportCases = [
  {
    id: "cgo_import",
    entry_path: "main.go",
    expected_message_substring:
      "unsupported syntax: cgo via `import \"C\"` is outside the supported subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "C"

func main() {}
`,
      },
    ],
  },
  {
    id: "plugin_import",
    entry_path: "main.go",
    expected_message_substring:
      "unsupported syntax: package \`plugin\` is outside the supported subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "plugin"

func main() {}
`,
      },
    ],
  },
  {
    id: "os_exec_import",
    entry_path: "main.go",
    expected_message_substring:
      "unsupported syntax: package \`os/exec\` is outside the supported subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "os/exec"

func main() {}
`,
      },
    ],
  },
  {
    id: "unsafe_import",
    entry_path: "main.go",
    expected_message_substring:
      "unsupported syntax: package \`unsafe\` is outside the supported subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "unsafe"

func main() {}
`,
      },
    ],
  },
  {
    id: "arbitrary_fs_open",
    entry_path: "main.go",
    expected_message_substring:
      "package selector \`os.Open\` is not supported in the current subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "os"

func main() {
    _, _ = os.Open("file.txt")
}
`,
      },
    ],
  },
  {
    id: "reflect_mutation_set_int",
    entry_path: "main.go",
    expected_message_substring:
      "method \`SetInt\` is not part of interface \`reflect.Value\` in the current subset",
    expected_category: "compile_error",
    files: [
      {
        path: "main.go",
        contents: `package main

import "reflect"

func main() {
    value := 1
    reflect.ValueOf(&value).Elem().SetInt(3)
}
`,
      },
    ],
  },
];
