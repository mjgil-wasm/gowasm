import {
  formatProgram,
  helloWorldProgram,
  lintProgram,
} from "./test-worker-programs.js";
import { negativeSupportCases } from "./test-worker-negative-support-fixtures.js";

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const ciMode = new URLSearchParams(window.location.search).has("ci");

let passed = 0;
let failed = 0;

function log(message) {
  results.textContent += `${message}\n`;
}

function assert(condition, name, detail) {
  if (condition) {
    passed += 1;
    return;
  }
  failed += 1;
  log(`${name}: ${detail}`);
}

function createWorker() {
  return new Worker("./engine-worker.js", { type: "module" });
}

function sendAndWait(worker, message, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`timed out after ${timeoutMs}ms`)), timeoutMs);
    function handler({ data }) {
      clearTimeout(timer);
      worker.removeEventListener("message", handler);
      resolve(data);
    }
    worker.addEventListener("message", handler);
    worker.postMessage(message);
  });
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

async function runAll() {
  const worker = createWorker();
  try {
    const boot = await sendAndWait(worker, { kind: "boot" });
    assert(boot.kind === "ready", "boot", JSON.stringify(boot));

    const runResult = await sendAndWait(worker, helloWorldProgram());
    assert(runResult.kind === "run_result", "run_result kind", JSON.stringify(runResult));
    assert(String(runResult.stdout ?? "").trim() === "hello", "hello stdout", JSON.stringify(runResult.stdout));

    const formatResult = await sendAndWait(
      worker,
      formatProgram(`package main

import "fmt"

func main(){
fmt.Println("hi")
}
`),
    );
    const formatted = formatResult.files?.find((file) => file.path === "main.go")?.contents;
    assert(formatResult.kind === "format_result", "format_result kind", JSON.stringify(formatResult));
    assert(formatted?.includes('func main(){\n\tfmt.Println("hi")\n}'), "format body", JSON.stringify(formatted));

    const lintResult = await sendAndWait(
      worker,
      lintProgram(`package main

import "fmt"

func main() {
println("hi")
}
`),
    );
    assert(lintResult.kind === "lint_result", "lint_result kind", JSON.stringify(lintResult));
    assert(Array.isArray(lintResult.diagnostics) && lintResult.diagnostics.length === 2, "lint diagnostics count", JSON.stringify(lintResult.diagnostics));
    assert(
      lintResult.diagnostics?.some((diagnostic) => diagnostic.message?.includes("never references `fmt`")),
      "lint unused import",
      JSON.stringify(lintResult.diagnostics),
    );

    for (const testCase of negativeSupportCases) {
      const result = await sendAndWait(worker, {
        kind: "run",
        entry_path: testCase.entry_path,
        files: testCase.files,
      });
      const diagnostic = Array.isArray(result.diagnostics) ? result.diagnostics[0] : null;
      assert(result.kind === "run_result", `${testCase.id} kind`, JSON.stringify(result));
      assert(result.stdout === "", `${testCase.id} stdout`, JSON.stringify(result.stdout));
      assert(diagnostic != null, `${testCase.id} diagnostic`, JSON.stringify(result));
      assert(
        diagnostic?.category === testCase.expected_category,
        `${testCase.id} category`,
        JSON.stringify(diagnostic),
      );
      assert(
        diagnostic?.message?.includes(testCase.expected_message_substring),
        `${testCase.id} message`,
        JSON.stringify(diagnostic),
      );
    }
  } finally {
    worker.terminate();
  }

  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `all worker ci smoke tests passed (${passed} assertions)`;
  } else {
    summary.className = "fail";
    summary.textContent = `${failed} worker ci smoke assertion(s) failed, ${passed} passed\n${results.textContent.trim()}`;
  }
  finishCiSummary(summary.textContent, summary.className);
}

void runAll();
