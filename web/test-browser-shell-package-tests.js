import { resetModuleCacheDatabase } from "./test-worker-modules.js";
import {
  click,
  clickAndWaitFor,
  control,
  loadShellFrame,
  setInputValue,
  setPlainValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellMainPackageTestingImportDiagnostic({
  assert,
  frame,
  log,
}) {
  log("\n--- browser shell main package test harness ---");

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await setWorkspaceFileContents(doc, "main.go", mainSource());
    await addWorkspaceFile(doc, "main_test.go", mainTestSource());
    await setPlainValue(doc, "package-target-input", "main.go");

    await clickAndWaitFor(
      doc,
      "test-package-button",
      () =>
        control(doc, "status").textContent.includes("Package tests passed")
          && control(doc, "output").textContent.includes("PASS TestAdd"),
      "main package test request",
    );

    const output = control(doc, "output").textContent;
    assert(
      output.includes("Package tests passed."),
      "browser shell main package test reports the package-test success summary",
      shellSnapshot(doc),
    );
    assert(
      output.includes("Target: main.go"),
      "browser shell main package test keeps the selected package target in the result",
      shellSnapshot(doc),
    );
    assert(
      output.includes("RUN TestAdd") && output.includes("PASS TestAdd"),
      "browser shell main package test runs the same-package testing.T test through the package runner",
      shellSnapshot(doc),
    );
    assert(
      !output.includes("tests failed")
        && !output.includes("import `testing` could not be resolved")
        && !output.includes("expected `[`"),
      "browser shell main package test no longer surfaces the earlier parser or testing import blockers",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

async function addWorkspaceFile(doc, path, contents) {
  await setInputValue(doc, "file-path-input", path);
  click(doc, "add-file-button");
  await waitFor(
    () => Array.from(control(doc, "file-list").options).some((option) => option.value === path),
    `workspace file ${path} added`,
    doc,
  );
  await setWorkspaceFileContents(doc, path, contents);
}

function mainSource() {
  return `package main

import "fmt"

// Add takes two integers and returns their sum.
func Add(a, b int) int {
\treturn a + b
}

func main() {
\tresult := Add(5, 7)
\tfmt.Printf("5 + 7 = %d\\n", result)
}
`;
}

function mainTestSource() {
  return `package main

import "testing"

func TestAdd(t *testing.T) {
\t// Table-driven tests are the standard way to write tests in Go
\ttests := []struct {
\t\tname     string
\t\ta        int
\t\tb        int
\t\texpected int
\t}{
\t\t{"positive numbers", 2, 3, 5},
\t\t{"negative numbers", -2, -4, -6},
\t\t{"mixed numbers", -1, 5, 4},
\t\t{"zeroes", 0, 0, 0},
\t}

\tfor _, tc := range tests {
\t\tt.Run(tc.name, func(t *testing.T) {
\t\t\tresult := Add(tc.a, tc.b)
\t\t\tif result != tc.expected {
\t\t\t\tt.Errorf("Add(%d, %d) = %d; want %d", tc.a, tc.b, result, tc.expected)
\t\t\t}
\t\t})
\t}
}
`;
}
