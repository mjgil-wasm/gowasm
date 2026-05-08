import {
  testCancelWithNoRun,
  testFallbackTermination,
  testFetchCancellation,
  testPackageTestCancellation,
  testSleepCancellation,
} from "./test-worker-cancellation.js";

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
  const context = { assert, createWorker, log, sendAndWait };
  const tests = [
    () => testCancelWithNoRun(context),
    () => testFallbackTermination(context),
    () => testSleepCancellation(context),
    () => testFetchCancellation(context),
    () => testPackageTestCancellation(context),
  ];

  for (const test of tests) {
    try {
      await test();
    } catch (error) {
      failed += 1;
      log(error?.stack || error?.message || String(error));
    }
  }

  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `all worker cancellation tests passed (${passed} assertions)`;
  } else {
    summary.className = "fail";
    summary.textContent =
      `${failed} worker cancellation failure(s), ${passed} assertions passed\n`
      + results.textContent.trim();
  }
  finishCiSummary(summary.textContent, summary.className);
}

void runAll();
