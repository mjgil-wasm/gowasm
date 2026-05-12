const sourceNode = document.querySelector("#source");
const outputNode = document.querySelector("#output");
const statusNode = document.querySelector("#status");
const runButton = document.querySelector("#run-button");
const testButton = document.querySelector("#test-button");
const formatButton = document.querySelector("#format-button");
const lintButton = document.querySelector("#lint-button");

sourceNode.value = `package main

import "fmt"

func helper() {
\tfmt.Println("worker shell")
}

func main() {
\thelper()
\tfmt.Println("Rust engine next")
}
`;

const worker = new Worker("./engine-worker.js", { type: "module" });

function setStatus(msg, isError = false) {
  statusNode.textContent = msg;
  statusNode.className = isError ? "error" : "ok";
}

function log(msg, color = "#d0d0e0") {
  const span = document.createElement("span");
  span.style.color = color;
  span.textContent = msg + "\n";
  outputNode.appendChild(span);
}

function clearOutput() {
  outputNode.textContent = "";
}

function getWorkspaceFiles() {
  return [{ path: "main.go", contents: sourceNode.value }];
}

function sendRequest(kind, payload = {}) {
  clearOutput();
  worker.postMessage({ kind, ...payload });
}

function handleRunResult(data) {
  setStatus("Run complete");
  if (data.stdout) {
    log(data.stdout, "#d0d0e0");
  }
  if (data.diagnostics?.length) {
    for (const d of data.diagnostics) {
      log(`[${d.severity || "error"}] ${d.message}`, "#ff6666");
    }
  }
}

function handleTestResult(data) {
  const label = data.passed ? "passed" : "failed";
  setStatus(`Test ${label}`, !data.passed);
  if (data.stdout) {
    log(data.stdout, "#d0d0e0");
  }
  if (data.details?.length) {
    for (const t of data.details) {
      const color = t.passed ? "#66ff99" : "#ff6666";
      log(`  ${t.passed ? "✓" : "✗"} ${t.name}`, color);
    }
  }
  if (data.diagnostics?.length) {
    for (const d of data.diagnostics) {
      log(`[${d.severity || "error"}] ${d.message}`, "#ff6666");
    }
  }
}

function handleLintResult(data) {
  if (data.diagnostics?.length) {
    setStatus(`${data.diagnostics.length} lint finding(s)`, true);
    for (const d of data.diagnostics) {
      log(`[${d.severity || "error"}] ${d.message}`, "#ffaa66");
    }
  } else {
    setStatus("Lint clean");
    log("No lint findings.", "#66ff99");
  }
}

function handleFormatResult(data) {
  if (data.diagnostics?.length) {
    setStatus("Format diagnostics received", true);
    for (const d of data.diagnostics) {
      log(`[${d.severity || "error"}] ${d.message}`, "#ff6666");
    }
  } else if (data.files?.length) {
    setStatus("Format applied");
    const main = data.files.find((f) => f.path === "main.go");
    if (main) {
      sourceNode.value = main.contents;
    }
    log("Formatting applied.", "#66ff99");
  } else {
    setStatus("Format complete");
  }
}

worker.addEventListener("message", ({ data }) => {
  switch (data.kind) {
    case "ready":
      setStatus(`Worker ready: ${data.info.engine_name}`);
      break;

    case "run_result":
      handleRunResult(data);
      break;

    case "test_result":
      handleTestResult(data);
      break;

    case "lint_result":
      handleLintResult(data);
      break;

    case "format_result":
      handleFormatResult(data);
      break;

    case "diagnostics":
      setStatus("Diagnostics received", true);
      for (const d of data.diagnostics) {
        log(`[${d.severity || "error"}] ${d.message}`, "#ff6666");
      }
      break;

    case "fatal":
      setStatus(data.message, true);
      log(`FATAL: ${data.message}`, "#ff6666");
      break;

    case "cancelled":
      setStatus("Cancelled");
      log("Execution cancelled.", "#ffaa66");
      break;

    default:
      setStatus("Unknown worker message", true);
      log(JSON.stringify(data, null, 2), "#ff6666");
      break;
  }
});

worker.postMessage({ kind: "boot" });

runButton.addEventListener("click", () => {
  setStatus("Running…");
  sendRequest("run", {
    entry_path: "main.go",
    files: getWorkspaceFiles(),
  });
});

testButton.addEventListener("click", () => {
  setStatus("Testing…");
  sendRequest("test_package", {
    target_path: "main.go",
    files: getWorkspaceFiles(),
  });
});

formatButton.addEventListener("click", () => {
  setStatus("Formatting…");
  sendRequest("format", {
    files: getWorkspaceFiles(),
  });
});

lintButton.addEventListener("click", () => {
  setStatus("Linting…");
  sendRequest("lint", {
    files: getWorkspaceFiles(),
  });
});
