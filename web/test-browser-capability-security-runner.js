import { runBrowserCapabilitySecurityTests } from "./test-browser-capability-security.js";

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
    await runBrowserCapabilitySecurityTests({ assert, log });
  } catch (error) {
    failed += 1;
    log(error?.stack || error?.message || String(error));
  }

  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `all browser capability security tests passed (${passed} assertions)`;
  } else {
    summary.className = "fail";
    summary.textContent = `${failed} browser capability security failure(s), ${passed} assertions passed\n${results.textContent.trim()}`;
  }
  finishCiSummary(summary.textContent, summary.className);
}

void runAll();
