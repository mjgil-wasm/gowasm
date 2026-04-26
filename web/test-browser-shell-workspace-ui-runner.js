import { testBrowserShellWorkspaceUi } from "./test-browser-shell-workspace-ui.js";

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const frame = document.querySelector("#shell-frame");
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
  try {
    await testBrowserShellWorkspaceUi({ assert, frame, log });
  } catch (error) {
    failed += 1;
    log(error?.stack || error?.message || String(error));
  }

  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `all browser shell workspace UI tests passed (${passed} assertions)`;
  } else {
    summary.className = "fail";
    summary.textContent = `${failed} browser shell workspace UI failure(s), ${passed} assertions passed\n${results.textContent.trim()}`;
  }
  finishCiSummary(summary.textContent, summary.className);
}

void runAll();
