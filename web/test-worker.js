import {
  formatProgram,
  helloWorldProgram,
  ioFsCapabilityProgram,
  lintProgram,
  netHttpCapabilityProgram,
  netHttpFailureProgram,
  osCapabilityProgram,
  runtimeFaultProgram,
} from "./test-worker-programs.js";
import {
  testCancelWithNoRun,
  testFallbackTermination,
  testFetchCancellation,
  testPackageTestCancellation,
  testSleepCancellation,
} from "./test-worker-cancellation.js";
import {
  testPersistentModuleGraphPath,
  testStalePersistentModuleEntryInvalidation,
} from "./test-worker-modules.js";
import {
  testExternalPackageTestBoundary,
  testFilteredPackageTestRequest,
  testFailedPackageTestRequestDetails,
  testFailedSnippetTestRequestDetails,
  testPackageTestRequest,
  testPackageTestRequestWithExistingMain,
  testSnippetCompileFailureDiagnostics,
  testSnippetTestRequest,
} from "./test-worker-package-tests.js";
import {
  testDiagnosticExcerptAndSuggestionFormatting,
  testSnippetResultFormatting,
  testModuleStatusRetryFormatting,
  testRuntimeDiagnosticFormatting,
  testStructuredIssueCollection,
  testStructuredIssuePanelRendering,
  testStructuredTestResultFormatting,
} from "./test-browser-shell-output.js";
import {
  testWorkerShellBackendBootAndRequestPath,
  testWorkerShellBackendCancellationTimeoutRestart,
} from "./test-browser-shell-backend.js";
import { testReservedModuleCacheWorkspacePaths } from "./test-browser-workspace.js";
import { testModuleAndToolingEndToEnd } from "./test-worker-tooling.js";

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const ciMode = new URLSearchParams(window.location.search).has("ci");

let passed = 0;
let failed = 0;

function log(message) {
  results.textContent += message + "\n";
}

function pass(name) {
  passed++;
  log(`  PASS  ${name}`);
}

function fail(name, reason) {
  failed++;
  log(`  FAIL  ${name}: ${reason}`);
}

function assert(condition, name, detail) {
  if (condition) {
    pass(name);
  } else {
    fail(name, detail || "assertion failed");
  }
}

function finishCiSummary(text, className) {
  if (!ciMode) {
    return;
  }
  void fetch("/__gowasm_ci_complete", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ elementId: "summary", className, text }),
  });
}

function createWorker() {
  return new Worker("./engine-worker.js", { type: "module" });
}

function workerTestContext() {
  return { log, createWorker, sendAndWait, assert };
}

function testStructuredTestResultFormattingHarness() {
  return testStructuredTestResultFormatting({ assert });
}

function testSnippetResultFormattingHarness() {
  return testSnippetResultFormatting({ assert });
}

function testDiagnosticExcerptAndSuggestionFormattingHarness() {
  return testDiagnosticExcerptAndSuggestionFormatting({ assert });
}

function testRuntimeDiagnosticFormattingHarness() {
  return testRuntimeDiagnosticFormatting({ assert });
}

function testModuleStatusRetryFormattingHarness() {
  return testModuleStatusRetryFormatting({ assert });
}

function testStructuredIssueCollectionHarness() {
  return testStructuredIssueCollection({ assert });
}

function testStructuredIssuePanelRenderingHarness() {
  return testStructuredIssuePanelRendering({ assert });
}

function testWorkerShellBackendBootAndRequestPathHarness() {
  return testWorkerShellBackendBootAndRequestPath({ assert });
}

function testWorkerShellBackendCancellationTimeoutRestartHarness() {
  return testWorkerShellBackendCancellationTimeoutRestart({ assert });
}

function testReservedModuleCacheWorkspacePathsHarness() {
  return testReservedModuleCacheWorkspacePaths({ assert });
}

function testModuleAndToolingEndToEndHarness() {
  return testModuleAndToolingEndToEnd(workerTestContext());
}

function testCancelWithNoRunHarness() {
  return testCancelWithNoRun(workerTestContext());
}

function testFallbackTerminationHarness() {
  return testFallbackTermination(workerTestContext());
}

function testSleepCancellationHarness() {
  return testSleepCancellation(workerTestContext());
}

function testFetchCancellationHarness() {
  return testFetchCancellation(workerTestContext());
}

function testPackageTestCancellationHarness() {
  return testPackageTestCancellation(workerTestContext());
}

function testPackageTestRequestHarness() {
  return testPackageTestRequest(workerTestContext());
}

function testFilteredPackageTestRequestHarness() {
  return testFilteredPackageTestRequest(workerTestContext());
}

function testPackageTestRequestWithExistingMainHarness() {
  return testPackageTestRequestWithExistingMain(workerTestContext());
}

function testExternalPackageTestBoundaryHarness() {
  return testExternalPackageTestBoundary(workerTestContext());
}

function testFailedPackageTestRequestDetailsHarness() {
  return testFailedPackageTestRequestDetails(workerTestContext());
}

function testSnippetTestRequestHarness() {
  return testSnippetTestRequest(workerTestContext());
}

function testFailedSnippetTestRequestDetailsHarness() {
  return testFailedSnippetTestRequestDetails(workerTestContext());
}

function testSnippetCompileFailureDiagnosticsHarness() {
  return testSnippetCompileFailureDiagnostics(workerTestContext());
}

function sendAndWait(worker, message, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    function handler({ data }) {
      clearTimeout(timer);
      worker.removeEventListener("message", handler);
      resolve(data);
    }
    worker.addEventListener("message", handler);
    worker.postMessage(message);
  });
}

function waitForMessage(worker, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`timed out waiting for message after ${timeoutMs}ms`));
    }, timeoutMs);

    function handler({ data }) {
      clearTimeout(timer);
      worker.removeEventListener("message", handler);
      resolve(data);
    }
    worker.addEventListener("message", handler);
  });
}

async function testPersistentModuleGraphPathWithHarness() {
  await testPersistentModuleGraphPath({ assert, createWorker, log, sendAndWait });
}

async function testStalePersistentModuleEntryInvalidationWithHarness() {
  await testStalePersistentModuleEntryInvalidation({
    assert,
    createWorker,
    log,
    sendAndWait,
  });
}

// ---------------------------------------------------------------------------
// Test: basic run completes
// ---------------------------------------------------------------------------
async function testBasicRun() {
  log("\n--- basic run ---");
  const worker = createWorker();
  try {
    const boot = await sendAndWait(worker, { kind: "boot" });
    assert(boot.kind === "ready", "boot produces ready message");

    const result = await sendAndWait(worker, helloWorldProgram());
    assert(result.kind === "run_result", "run produces run_result");
    assert(
      result.stdout?.trim() === "hello",
      "stdout contains expected output",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: format request uses the shared worker boundary
// ---------------------------------------------------------------------------
async function testFormatRequest() {
  log("\n--- format request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(
      worker,
      formatProgram(
        'package main\n\nimport "fmt"\n\n'
          + "func choose[T any](value T) T {\n// keep\n// comment\nreturn value\n}\n\n"
          + 'func main() {\n'
          + 'data := `{\n"items": [\n1,\n2\n]\n}`\n'
          + "value := choose[int](1)\n"
          + 'println("hi")\n'
          + "switch true {\n"
          + "case true:\n"
          + 'println("nested")\n'
          + "}\n"
          + 'fmt.Println(value)\n'
          + "}\n",
      ),
    );
    assert(
      result.kind === "format_result",
      "format request produces format_result",
      `got: ${result.kind}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 0,
      "format request returns no diagnostics for valid source",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    const formatted = result.files?.find((file) => file.path === "main.go")?.contents;
    assert(
      formatted?.includes('\tprintln("hi")'),
      "format request reindents the function body",
      `got: ${JSON.stringify(formatted)}`,
    );
    assert(
      formatted?.includes("func choose[T any](value T) T {"),
      "format request preserves supported generic declarations through the product path",
      `got: ${JSON.stringify(formatted)}`,
    );
    assert(
      formatted?.includes("\t// keep\n\t// comment"),
      "format request preserves supported line comments while reindenting the surrounding generic declaration",
      `got: ${JSON.stringify(formatted)}`,
    );
    assert(
      formatted?.includes('\tdata := `{\n"items": [\n1,\n2\n]\n}`'),
      "format request preserves multiline raw string bodies",
      `got: ${JSON.stringify(formatted)}`,
    );
    assert(
      formatted?.includes("\tcase true:") && formatted?.includes('\t\tprintln("nested")'),
      "format request reindents switch cases",
      `got: ${JSON.stringify(formatted)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: lint request uses the shared worker boundary
// ---------------------------------------------------------------------------
async function testLintRequest() {
  log("\n--- lint request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(
      worker,
      lintProgram(`package main

import "fmt"

func main() {
println("hi")
}
`),
    );
    assert(
      result.kind === "lint_result",
      "lint request produces lint_result",
      `got: ${result.kind}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 2,
      "lint request returns formatter and unused-import diagnostics",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.diagnostics?.every((diagnostic) => diagnostic.severity === "warning"),
      "lint request reports warning severity",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.diagnostics?.every(
        (diagnostic) =>
          diagnostic.file_path === "main.go"
          && typeof diagnostic.position?.line === "number"
          && typeof diagnostic.position?.column === "number"
          && typeof diagnostic.source_span?.start?.line === "number"
          && typeof diagnostic.source_excerpt?.line === "number"
          && typeof diagnostic.suggested_action === "string",
      ),
      "lint request reports source positions, spans, excerpts, and suggestions for lint warnings",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.diagnostics?.some((diagnostic) => diagnostic.message?.includes("run Format")),
      "lint request points the user at the formatter action",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.diagnostics?.some((diagnostic) =>
        diagnostic.message?.includes("never references `fmt`"),
      ),
      "lint request reports unused imports",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    const formattingDiagnostic = result.diagnostics?.find((diagnostic) =>
      diagnostic.message?.includes("run Format"),
    );
    assert(
      formattingDiagnostic?.position?.line === 6 && formattingDiagnostic?.position?.column === 1,
      "lint request reports formatter drift at the first differing source position",
      `got: ${JSON.stringify(formattingDiagnostic)}`,
    );
    const unusedImportDiagnostic = result.diagnostics?.find((diagnostic) =>
      diagnostic.message?.includes("never references `fmt`"),
    );
    assert(
      unusedImportDiagnostic?.position?.line === 3
        && unusedImportDiagnostic?.position?.column === 8,
      "lint request reports unused imports at the import path position",
      `got: ${JSON.stringify(unusedImportDiagnostic)}`,
    );
    assert(
      formattingDiagnostic?.suggested_action?.includes("Run Format"),
      "lint formatter warnings include an explicit suggested action",
      `got: ${JSON.stringify(formattingDiagnostic)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: lint suppression directives use the shared worker boundary
// ---------------------------------------------------------------------------
async function testLintSuppressionRequest() {
  log("\n--- lint suppression request ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(
      worker,
      lintProgram(`package main

//gowasm:ignore format-drift
//gowasm:ignore duplicate-import
//gowasm:ignore unused-import
import (
"fmt"
"fmt"
)

func main() {
println("hi")
}
`),
    );
    assert(
      result.kind === "lint_result",
      "lint suppression request still produces lint_result",
      `got: ${result.kind}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 0,
      "lint suppression request removes the named lint rules only",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: diagnostics for invalid program
// ---------------------------------------------------------------------------
async function testDiagnosticsForInvalidProgram() {
  log("\n--- diagnostics for invalid program ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "run",
      entry_path: "main.go",
      files: [
        {
          path: "main.go",
          contents: `package main
func main() {
  undeclaredFunction()
}
`,
        },
      ],
    });
    assert(
      result.kind === "run_result" &&
        Array.isArray(result.diagnostics) &&
        result.diagnostics.length === 1,
      "invalid program produces compile diagnostics through run_result",
      `got: ${JSON.stringify(result)}`,
    );
    const diagnostic = result.diagnostics?.[0];
    assert(
      diagnostic?.source_span?.start?.line === 3
        && diagnostic?.source_excerpt?.line === 3
        && diagnostic?.suggested_action?.includes("compile again"),
      "invalid program diagnostics include source span, excerpt, and suggested action",
      `got: ${JSON.stringify(diagnostic)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: structured runtime diagnostics for failed runs
// ---------------------------------------------------------------------------
async function testRuntimeDiagnosticsForFailedRun() {
  log("\n--- structured runtime diagnostics ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, runtimeFaultProgram());
    assert(
      result.kind === "run_result",
      "runtime fault still produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 1,
      "runtime fault returns one diagnostic",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    const diagnostic = result.diagnostics?.[0];
    assert(
      diagnostic?.runtime?.root_message?.includes("division by zero"),
      "runtime diagnostic includes structured root message",
      `got: ${JSON.stringify(diagnostic)}`,
    );
    assert(
      diagnostic?.runtime?.stack_trace?.[0]?.function === "explode",
      "runtime diagnostic includes the faulting frame",
      `got: ${JSON.stringify(diagnostic?.runtime)}`,
    );
    assert(
      diagnostic?.runtime?.stack_trace?.[1]?.function === "main",
      "runtime diagnostic includes the caller frame",
      `got: ${JSON.stringify(diagnostic?.runtime)}`,
    );
    assert(
      diagnostic?.source_span?.start?.line === 6
        && diagnostic?.source_excerpt?.line === 6,
      "runtime diagnostic includes source excerpt metadata for the top frame",
      `got: ${JSON.stringify(diagnostic)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: structured runtime diagnostics for browser budget exhaustion
// ---------------------------------------------------------------------------
async function testBudgetDiagnosticsForFailedRun() {
  log("\n--- budget exhaustion diagnostics ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, {
      kind: "run",
      entry_path: "main.go",
      files: [
        {
          path: "main.go",
          contents: `package main
func main() {
  for {
  }
}
`,
        },
      ],
    }, 15000);
    assert(
      result.kind === "run_result",
      "budget exhaustion still produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 1,
      "budget exhaustion returns one diagnostic",
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    const diagnostic = result.diagnostics?.[0];
    assert(
      diagnostic?.runtime?.root_message?.includes("instruction budget exhausted"),
      "budget diagnostic includes structured root message",
      `got: ${JSON.stringify(diagnostic)}`,
    );
    assert(
      diagnostic?.runtime?.stack_trace?.[0]?.function === "main",
      "budget diagnostic includes the active frame",
      `got: ${JSON.stringify(diagnostic?.runtime)}`,
    );
    assert(
      diagnostic?.suggested_action?.includes("instruction budget"),
      "budget diagnostics include a suggested action",
      `got: ${JSON.stringify(diagnostic)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: browser-backed net/http capability path
// ---------------------------------------------------------------------------
async function testNetHttpCapabilityPath() {
  log("\n--- net/http capability path ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, netHttpCapabilityProgram(), 15000);
    assert(
      result.kind === "run_result",
      "net/http capability run produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      result.stdout === "200 hello true\n",
      "net/http capability run produces expected output",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: browser-backed net/http failure path
// ---------------------------------------------------------------------------
async function testNetHttpFailurePath() {
  log("\n--- net/http failure path ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, netHttpFailureProgram(), 15000);
    assert(
      result.kind === "run_result",
      "net/http failure run produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      result.stdout === "true true\n",
      "net/http failure run reports a nil response plus non-nil error",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: browser-backed io/fs capability path
// ---------------------------------------------------------------------------
async function testIoFsCapabilityPath() {
  log("\n--- io/fs capability path ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, ioFsCapabilityProgram(), 15000);
    assert(
      result.kind === "run_result",
      "io/fs capability run produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      result.stdout
        === "alpha true child true\n"
        + "5 true true 2 config.txt nested true\n"
        + "1 config.txt true .,config.txt,nested,nested/child.txt true\n"
        + "true 2 true true true true\n",
      "io/fs capability run produces expected output",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Test: browser-backed os capability path
// ---------------------------------------------------------------------------
async function testOsCapabilityPath() {
  log("\n--- os capability path ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(worker, osCapabilityProgram(), 15000);
    assert(
      result.kind === "run_result",
      "os capability run produces run_result",
      `got: ${result.kind}`,
    );
    assert(
      result.stdout === "beta true\n2 out.txt sub true\ntrue\n",
      "os capability run produces expected output",
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}

// ---------------------------------------------------------------------------
// Run all tests
// ---------------------------------------------------------------------------
async function runAll() {
  log("starting worker integration tests...");

  const tests = [
    testBasicRun,
    testCancelWithNoRunHarness,
    testFallbackTerminationHarness,
    testSleepCancellationHarness,
    testFetchCancellationHarness,
    testPackageTestCancellationHarness,
    testFormatRequest,
    testLintRequest,
    testLintSuppressionRequest,
    testStructuredTestResultFormattingHarness,
    testSnippetResultFormattingHarness,
    testDiagnosticExcerptAndSuggestionFormattingHarness,
    testRuntimeDiagnosticFormattingHarness,
    testModuleStatusRetryFormattingHarness,
    testStructuredIssueCollectionHarness,
    testStructuredIssuePanelRenderingHarness,
    testWorkerShellBackendBootAndRequestPathHarness,
    testWorkerShellBackendCancellationTimeoutRestartHarness,
    testReservedModuleCacheWorkspacePathsHarness,
    testPackageTestRequestHarness,
    testFilteredPackageTestRequestHarness,
    testPackageTestRequestWithExistingMainHarness,
    testExternalPackageTestBoundaryHarness,
    testFailedPackageTestRequestDetailsHarness,
    testSnippetTestRequestHarness,
    testFailedSnippetTestRequestDetailsHarness,
    testSnippetCompileFailureDiagnosticsHarness,
    testDiagnosticsForInvalidProgram,
    testRuntimeDiagnosticsForFailedRun,
    testBudgetDiagnosticsForFailedRun,
    testNetHttpCapabilityPath,
    testNetHttpFailurePath,
    testIoFsCapabilityPath,
    testOsCapabilityPath,
    testModuleAndToolingEndToEndHarness,
    testPersistentModuleGraphPathWithHarness,
    testStalePersistentModuleEntryInvalidationWithHarness,
  ];

  for (const test of tests) {
    try {
      await test();
    } catch (error) {
      fail(test.name, error.message);
    }
  }

  log("\n" + "=".repeat(50));
  const total = passed + failed;
  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `All ${total} assertions passed.`;
    finishCiSummary(summary.textContent, summary.className);
  } else {
    summary.className = "fail";
    summary.textContent = `${failed} of ${total} assertions failed.\n${results.textContent.trim()}`;
    finishCiSummary(summary.textContent, summary.className);
  }
  log(`${passed} passed, ${failed} failed`);
}

runAll();
