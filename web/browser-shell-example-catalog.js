export const PACKAGED_EXAMPLES = [
  {
    id: "generics-channels",
    title: "Generics and channels",
    description:
      "Uses a generic collector over a channel to show typed helpers and channel iteration inside the supported runtime slice.",
    concepts: ["generics", "channels"],
    entryPath: "main.go",
    packageTarget: "",
    selectedFilePath: "main.go",
    verify: {
      action: "run",
      outputIncludes: ["2,4,6"],
    },
    files: [
      {
        path: "go.mod",
        contents: "module example.com/genericschannels\n\ngo 1.21\n",
      },
      {
        path: "main.go",
        contents: `package main

import (
\t"fmt"
\t"strings"
)

func collect[T any](in <-chan T, render func(T) string) string {
\tparts := []string{}
\tfor value := range in {
\t\tparts = append(parts, render(value))
\t}
\treturn strings.Join(parts, ",")
}

func main() {
\tnums := make(chan int, 3)
\tfor _, value := range []int{2, 4, 6} {
\t\tnums <- value
\t}
\tclose(nums)
\tfmt.Println(collect(nums, func(value int) string { return fmt.Sprint(value) }))
}
`,
      },
    ],
  },
  {
    id: "json-workspace-fs",
    title: "JSON and workspace files",
    description:
      "Reads a checked-in workspace JSON file through the browser-backed filesystem and decodes it with encoding/json.",
    concepts: ["json", "fs"],
    entryPath: "main.go",
    packageTarget: "",
    selectedFilePath: "main.go",
    verify: {
      action: "run",
      outputIncludes: ['true {"count":2,"message":"Ada Lin"}'],
    },
    files: [
      {
        path: "go.mod",
        contents: "module example.com/jsonfs\n\ngo 1.21\n",
      },
      {
        path: "people.json",
        contents: `{"count":2,"message":"Ada Lin"}`,
      },
      {
        path: "main.go",
        contents: `package main

import (
\t"fmt"
\t"encoding/json"
\t"os"
)

func main() {
\tdata, err := os.ReadFile("people.json")
\tif err != nil {
\t\tpanic(err)
\t}
\tfmt.Println(json.Valid(data), string(data))
}
`,
      },
    ],
  },
  {
    id: "http-data-url",
    title: "HTTP data URL client",
    description:
      "Uses the browser-backed net/http client against a deterministic data URL so the example stays hermetic inside the browser gate.",
    concepts: ["http"],
    entryPath: "main.go",
    packageTarget: "",
    selectedFilePath: "main.go",
    verify: {
      action: "run",
      outputIncludes: ["200"],
    },
    files: [
      {
        path: "go.mod",
        contents: "module example.com/httpdataurl\n\ngo 1.21\n",
      },
      {
        path: "main.go",
        contents: `package main

import (
\t"fmt"
\t"net/http"
)

func main() {
\tresp, err := http.Get("data:text/plain,hello-http")
\tif err != nil {
\t\tpanic(err)
\t}
\tdefer resp.Body.Close()
\tfmt.Println(resp.StatusCode)
}
`,
      },
    ],
  },
  {
    id: "package-tests",
    title: "Package tests",
    description:
      "Shows the supported same-package Test* runner slice with an explicit package target under a non-main package.",
    concepts: ["tests"],
    entryPath: "main.go",
    packageTarget: "calc/calc.go",
    selectedFilePath: "calc/calc.go",
    verify: {
      action: "test_package",
      outputIncludes: ["Completed: TestSum"],
    },
    files: [
      {
        path: "go.mod",
        contents: "module example.com/packagetests\n\ngo 1.21\n",
      },
      {
        path: "main.go",
        contents: `package main

import (
\t"fmt"
\t"example.com/packagetests/calc"
)

func main() {
\tfmt.Println(calc.Sum(2, 3))
}
`,
      },
      {
        path: "calc/calc.go",
        contents: `package calc

func Sum(left int, right int) int {
\treturn left + right
}
`,
      },
      {
        path: "calc/calc_test.go",
        contents: `package calc

func TestSum() {
\tif Sum(2, 3) != 5 {
\t\tpanic("sum failed")
\t}
}
`,
      },
    ],
  },
  {
    id: "formatting",
    title: "Formatter cleanup",
    description:
      "Starts from intentionally uneven Go formatting and lets the browser shell apply the conservative supported formatter.",
    concepts: ["formatting"],
    entryPath: "main.go",
    packageTarget: "",
    selectedFilePath: "main.go",
    verify: {
      action: "format",
    },
    files: [
      {
        path: "go.mod",
        contents: "module example.com/formatting\n\ngo 1.21\n",
      },
      {
        path: "main.go",
        contents: `package main

import (
"fmt"
)

func pair[T any](left T,right T)string{
return fmt.Sprint(left, ":", right)
}

func main(){
fmt.Println(pair("format","me"))
}
`,
      },
    ],
  },
];

export const PACKAGED_EXAMPLE_IDS = PACKAGED_EXAMPLES.map((example) => example.id);

export function getPackagedExample(exampleId) {
  return PACKAGED_EXAMPLES.find((example) => example.id === String(exampleId ?? "")) ?? null;
}

export function formatPackagedExampleSummary(example) {
  const lines = [
    `${example.title}`,
    example.description,
    `Concepts: ${example.concepts.join(", ")}`,
  ];
  if (example.entryPath) {
    lines.push(`Entry path: ${example.entryPath}`);
  }
  if (example.packageTarget) {
    lines.push(`Package target: ${example.packageTarget}`);
  }
  lines.push(`Verification action: ${example.verify.action}`);
  return lines.join("\n");
}
