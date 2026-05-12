import {
  testBrowserShellEndToEnd,
  testBrowserShellRepresentativeSoak,
} from "./test-browser-shell-harness.js";
import { testBrowserShellCancellationRecovery } from "./test-browser-shell-cancellation.js";
import { testBrowserShellDiagnosticNavigation } from "./test-browser-shell-diagnostics.js";
import { testBrowserShellWorkspaceUi } from "./test-browser-shell-workspace-ui.js";

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const frame = document.querySelector("#shell-frame");
const ciMode = new URLSearchParams(window.location.search).has("ci");

let passed = 0;
let failed = 0;

function log(message) {
  results.textContent += message + "\n";
}

function pass(name) {
  passed += 1;
  log(`  PASS  ${name}`);
}

function fail(name, reason) {
  failed += 1;
  log(`  FAIL  ${name}: ${reason}`);
}

function assert(condition, name, detail) {
  if (condition) {
    pass(name);
    return;
  }
  fail(name, detail || "assertion failed");
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
  log("running browser shell integration tests...");

  const tests = [
    {
      name: "browser shell end-to-end harness",
      run: () => testBrowserShellEndToEnd({ assert, frame, log }),
    },
    {
      name: "browser shell cancellation and recovery harness",
      run: () => testBrowserShellCancellationRecovery({ assert, frame, log }),
    },
    {
      name: "browser shell representative soak harness",
      run: () => testBrowserShellRepresentativeSoak({ assert, frame, log }),
    },
    {
      name: "browser shell diagnostic navigation harness",
      run: () => testBrowserShellDiagnosticNavigation({ assert, frame, log }),
    },
    {
      name: "browser shell workspace UI harness",
      run: () => testBrowserShellWorkspaceUi({ assert, frame, log }),
    },
  ];

  for (const test of tests) {
    try {
      await test.run();
    } catch (error) {
      fail(test.name, error?.stack || error?.message || String(error));
    }
  }

  if (failed === 0) {
    summary.textContent = `all browser shell tests passed (${passed} assertions)`;
    summary.className = "pass";
    finishCiSummary(summary.textContent, summary.className);
  } else {
    summary.textContent =
      `${failed} browser shell test failure(s), ${passed} assertion(s) passed\n`
      + results.textContent.trim();
    summary.className = "fail";
    finishCiSummary(summary.textContent, summary.className);
  }
}

runAll();
