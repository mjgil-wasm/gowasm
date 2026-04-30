import { packageTestProgram, snippetTestProgram } from "./test-worker-programs.js";

export async function testPackageTestRequest({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- package test request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, packageTestProgram());
    assert(
      result.kind === "test_result",
      "package test request produces test_result",
      `got: ${result.kind}`,
    );
    assert(
      result.runner === "package",
      "package test result reports package runner kind",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.passed === true,
      "package test result passes for a successful same-package test",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "calc.go",
      "package test result reports the target path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      JSON.stringify(result.details?.completed_tests) === JSON.stringify(["TestAdd"]),
      "package test result reports the completed tests",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.stdout?.includes("PASS TestAdd"),
      "package test output includes the executed test name",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testFilteredPackageTestRequest({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- filtered package test request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "test_package",
      target_path: "calc.go",
      filter: "TestSub",
      files: [
        {
          path: "calc.go",
          contents: `package calc

func Add(left int, right int) int {
  return left + right
}

func Sub(left int, right int) int {
  return left - right
}
`,
        },
        {
          path: "calc_test.go",
          contents: `package calc

import "fmt"

func TestAdd() {
  fmt.Println("add-ran")
}

func TestSub() {
  fmt.Println("sub-ran")
}
`,
        },
      ],
    });
    assert(
      result.kind === "test_result" && result.passed === true,
      "filtered package test request produces a passing test_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      JSON.stringify(result.details?.planned_tests) === JSON.stringify(["TestSub"]),
      "filtered package test reports only the selected planned test",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      !result.stdout?.includes("TestAdd") && result.stdout?.includes("PASS TestSub"),
      "filtered package test stdout only includes the selected test",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testPackageTestRequestWithExistingMain({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- package test request with existing main ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "test_package",
      target_path: "main.go",
      files: [
        {
          path: "main.go",
          contents: `package main

func main() {
  panic("original main should not run during package tests")
}
`,
        },
        {
          path: "main_test.go",
          contents: `package main

import "fmt"

func TestExample() {
  fmt.Println("package-main-test")
}
`,
        },
      ],
    });
    assert(
      result.kind === "test_result",
      "package-main test request produces test_result",
      `got: ${result.kind}`,
    );
    assert(
      result.runner === "package",
      "package-main test result reports package runner kind",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.passed === true,
      "package-main test request passes without executing original main",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "main.go",
      "package-main test details report the target path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      JSON.stringify(result.details?.completed_tests) === JSON.stringify(["TestExample"]),
      "package-main test details report the completed test",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.stdout?.includes("PASS TestExample"),
      "package-main test output includes the executed test name",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testExternalPackageTestBoundary({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- external package test boundary ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "test_package",
      target_path: "calc_external_test.go",
      files: [
        {
          path: "calc.go",
          contents: `package calc

func Add() {}
`,
        },
        {
          path: "calc_external_test.go",
          contents: `package calc_test

func TestExternal() {}
`,
        },
      ],
    });
    assert(
      result.kind === "test_result" && result.passed === false,
      "external package test request produces a failed test_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.diagnostics?.[0]?.message?.includes("external test packages ending in `_test` are not yet supported"),
      "external package test request reports the explicit `_test` boundary",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testFailedPackageTestRequestDetails({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- failed package test request details ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
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

func TestAdd() {}

func TestFail() {
  panic("boom")
}

func TestAfter() {}
`,
        },
      ],
    });
    assert(
      result.kind === "test_result" && result.passed === false,
      "failed package test request produces a failed test_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "calc.go",
      "failed package test details report the target path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      JSON.stringify(result.details?.completed_tests) === JSON.stringify(["TestAdd"]),
      "failed package test details report completed tests",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.details?.active_test === "TestFail",
      "failed package test details report the active failing test",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.stdout?.includes("RUN TestFail"),
      "failed package test stdout keeps the in-flight test marker",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testSnippetTestRequest({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- snippet test request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(
      worker,
      snippetTestProgram(`import "fmt"

fmt.Println("snippet-pass")
`),
    );
    assert(
      result.kind === "test_result",
      "snippet test request produces test_result",
      `got: ${result.kind}`,
    );
    assert(
      result.runner === "snippet",
      "snippet test result reports snippet runner kind",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.passed === true,
      "snippet test passes for a successful sample",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "main.go",
      "snippet test result reports the entry path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      JSON.stringify(result.details?.completed_tests) === JSON.stringify(["main.go"]),
      "snippet test result reports the completed snippet entry",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.stdout?.trim() === "snippet-pass",
      "snippet test output preserves program stdout",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testFailedSnippetTestRequestDetails({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- failed snippet test request details ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "test_snippet",
      entry_path: "main.go",
      files: [
        {
          path: "main.go",
          contents: `panic("boom")
`,
        },
      ],
    });
    assert(
      result.kind === "test_result" && result.passed === false,
      "failed snippet test request produces a failed test_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "main.go",
      "failed snippet test details report the entry path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.details?.active_test === "main.go",
      "failed snippet test details report the active entry",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 1,
      "failed snippet test keeps the runtime diagnostics payload",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.diagnostics?.[0]?.runtime?.stack_trace?.[0]?.source_location?.line === 1,
      "failed snippet runtime diagnostics point back at the original snippet line",
      `got: ${JSON.stringify(result.diagnostics?.[0])}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testSnippetCompileFailureDiagnostics({
  log,
  createWorker,
  sendAndWait,
  assert,
}) {
  log("\n--- snippet compile failure diagnostics ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "test_snippet",
      entry_path: "main.go",
      files: [
        {
          path: "main.go",
          contents: `import "example.com/missing/tool"

println("hello")
`,
        },
      ],
    });
    assert(
      result.kind === "test_result" && result.passed === false,
      "snippet compile failure produces a failed test_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.details?.subject_path === "main.go",
      "snippet compile failure keeps the original entry path",
      `got: ${JSON.stringify(result.details)}`,
    );
    assert(
      result.diagnostics?.[0]?.file_path === "main.go"
        && result.diagnostics?.[0]?.position?.line === 1,
      "snippet compile failure diagnostics remap to the original snippet source",
      `got: ${JSON.stringify(result.diagnostics?.[0])}`,
    );
    assert(
      result.diagnostics?.[0]?.message?.includes("--> main.go:1:")
        && result.diagnostics?.[0]?.message?.includes(
          '1 | import "example.com/missing/tool"',
        ),
      "snippet compile failure diagnostics render the original snippet excerpt",
      `got: ${JSON.stringify(result.diagnostics?.[0]?.message)}`,
    );
  } finally {
    worker.terminate();
  }
}
