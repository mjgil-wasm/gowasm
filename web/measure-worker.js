import { MODULE_CACHE_DB_NAME } from "./engine-worker-module-cache.js";

const status = document.querySelector("#status");
const metrics = document.querySelector("#metrics");
const rerunButton = document.querySelector("#rerun-button");
const shellFrame = document.querySelector("#shell-frame");
const ciMode = new URLSearchParams(window.location.search).has("ci");

let activeWorker = null;

rerunButton.addEventListener("click", () => {
  void measureWorker();
});

void measureWorker();

function finishCiMetrics(text) {
  if (!ciMode) {
    return;
  }
  void fetch("/__gowasm_ci_complete", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ elementId: "metrics", text }),
  });
}

async function measureWorker() {
  rerunButton.disabled = true;
  metrics.textContent = "";
  status.textContent = "Measuring worker boot, first run, memory, and storage baselines...";
  disposeActiveWorker();

  try {
    await resetModuleCacheDatabase();
    const storageBeforeModuleLoadBytes = await measureStorageUsageBytes();
    const wasmBytes = await measureWasmBytes();
    const worker = new Worker("./engine-worker.js", { type: "module" });
    activeWorker = worker;

    const bootStart = performance.now();
    const ready = await sendAndWait(worker, { kind: "boot" }, 15000);
    if (ready?.kind !== "ready") {
      throw new Error(`expected ready response, got ${JSON.stringify(ready)}`);
    }
    const workerBootMs = roundDurationMs(performance.now() - bootStart);

    const compileDiagnosticsStart = performance.now();
    const compileDiagnosticsResult = await sendAndWait(worker, compileFailureProgram(), 15000);
    if (
      compileDiagnosticsResult?.kind !== "run_result"
        || !Array.isArray(compileDiagnosticsResult.diagnostics)
        || compileDiagnosticsResult.diagnostics.length === 0
    ) {
      throw new Error(
        `expected compile diagnostics run_result, got ${JSON.stringify(compileDiagnosticsResult)}`,
      );
    }
    const compileDiagnosticsMs = roundDurationMs(performance.now() - compileDiagnosticsStart);

    const runStart = performance.now();
    const runResult = await sendAndWait(worker, helloWorldProgram(), 15000);
    if (runResult?.kind !== "run_result") {
      throw new Error(`expected run_result, got ${JSON.stringify(runResult)}`);
    }
    if (String(runResult.stdout ?? "").trim() !== "hello") {
      throw new Error(`hello-world probe returned unexpected stdout: ${JSON.stringify(runResult.stdout)}`);
    }
    const helloRunMs = roundDurationMs(performance.now() - runStart);
    const { bytes: helloMemoryBytes, source: memoryMetricSource } = await measureMemoryUsageBytes();

    const gcChurnStart = performance.now();
    const gcChurnResult = await sendAndWait(worker, gcChurnProgram(), 30000);
    if (gcChurnResult?.kind !== "run_result") {
      throw new Error(`expected gc churn run_result, got ${JSON.stringify(gcChurnResult)}`);
    }
    if (!String(gcChurnResult.stdout ?? "").trim().startsWith("gc-churn-ok")) {
      throw new Error(
        `gc churn probe returned unexpected stdout: ${JSON.stringify(gcChurnResult.stdout)}`,
      );
    }
    const gcChurnRunMs = roundDurationMs(performance.now() - gcChurnStart);
    const { bytes: gcChurnMemoryBytes } = await measureMemoryUsageBytes();

    const moduleLoadStart = performance.now();
    const moduleLoadResult = await sendAndWait(
      worker,
      representativeModuleGraphRequest(),
      15000,
    );
    if (
      moduleLoadResult?.kind !== "module_graph_result"
        || !Array.isArray(moduleLoadResult.modules)
        || moduleLoadResult.modules.length !== 1
    ) {
      throw new Error(`expected module_graph_result, got ${JSON.stringify(moduleLoadResult)}`);
    }
    const moduleLoadMs = roundDurationMs(performance.now() - moduleLoadStart);

    await delay(50);
    const { bytes: moduleLoadMemoryBytes } = await measureMemoryUsageBytes();
    const storageAfterModuleLoadBytes = await measureStorageUsageBytes();
    const moduleCacheStorageDeltaBytes = Math.max(
      0,
      storageAfterModuleLoadBytes - storageBeforeModuleLoadBytes,
    );

    disposeActiveWorker();
    activeWorker = null;
    const { shellReadyMs, shellRunMs } = await measureBrowserShellInteraction();

    status.textContent = "Measurements ready";
    metrics.textContent = [
      "metric_version=3",
      `wasm_bytes=${wasmBytes}`,
      `worker_boot_ms=${workerBootMs}`,
      `compile_diagnostics_ms=${compileDiagnosticsMs}`,
      `hello_run_ms=${helloRunMs}`,
      `worker_boot_and_run_ms=${workerBootMs + helloRunMs}`,
      `gc_churn_run_ms=${gcChurnRunMs}`,
      `module_load_ms=${moduleLoadMs}`,
      `hello_memory_bytes=${helloMemoryBytes}`,
      `gc_churn_memory_bytes=${gcChurnMemoryBytes}`,
      `module_load_memory_bytes=${moduleLoadMemoryBytes}`,
      `module_cache_storage_delta_bytes=${moduleCacheStorageDeltaBytes}`,
      `shell_ready_ms=${shellReadyMs}`,
      `shell_run_ms=${shellRunMs}`,
      `memory_metric_source=${memoryMetricSource}`,
    ].join("\n");
    finishCiMetrics(metrics.textContent);
  } catch (error) {
    status.textContent = "Measurement failed";
    metrics.textContent = `error=${formatError(error)}`;
    finishCiMetrics(metrics.textContent);
  } finally {
    disposeActiveWorker();
    await unloadShellFrame();
    await delay(50);
    try {
      await resetModuleCacheDatabase();
    } catch (cleanupError) {
      console.warn("failed to reset module cache database after measurement", cleanupError);
    }
    rerunButton.disabled = false;
  }
}

async function measureWasmBytes() {
  const wasmUrl = new URL("./generated/gowasm_engine_wasm.wasm", import.meta.url);
  const response = await fetch(wasmUrl, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(
      `failed to fetch ${wasmUrl.pathname}: ${response.status} ${response.statusText}`,
    );
  }
  return (await response.arrayBuffer()).byteLength;
}

function sendAndWait(worker, message, timeoutMs) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    function handleMessage({ data }) {
      clearTimeout(timer);
      worker.removeEventListener("message", handleMessage);
      resolve(data);
    }

    function handleError(event) {
      clearTimeout(timer);
      worker.removeEventListener("error", handleError);
      worker.removeEventListener("message", handleMessage);
      reject(new Error(event.message || "worker error"));
    }

    worker.addEventListener("message", handleMessage);
    worker.addEventListener("error", handleError, { once: true });
    worker.postMessage(message);
  });
}

function disposeActiveWorker() {
  if (!activeWorker) {
    return;
  }
  activeWorker.terminate();
  activeWorker = null;
}

async function measureMemoryUsageBytes() {
  let primaryError = null;
  if (typeof performance.measureUserAgentSpecificMemory === "function") {
    try {
      const result = await performance.measureUserAgentSpecificMemory();
      return {
        bytes: normalizeNonNegativeInteger(result?.bytes, "measureUserAgentSpecificMemory.bytes"),
        source: "measureUserAgentSpecificMemory",
      };
    } catch (error) {
      primaryError = error;
    }
  }

  const legacyHeapBytes = performance.memory?.usedJSHeapSize;
  if (Number.isFinite(legacyHeapBytes) && legacyHeapBytes >= 0) {
    return {
      bytes: normalizeNonNegativeInteger(legacyHeapBytes, "performance.memory.usedJSHeapSize"),
      source: "performance.memory.usedJSHeapSize",
    };
  }

  throw new Error(
    primaryError
      ? `browser does not expose a usable memory measurement API; measureUserAgentSpecificMemory failed: ${formatError(primaryError)}`
      : "browser does not expose a supported memory measurement API; use a Chromium-derived browser",
  );
}

async function measureStorageUsageBytes() {
  if (!navigator.storage?.estimate) {
    throw new Error("browser does not expose navigator.storage.estimate()");
  }
  const estimate = await navigator.storage.estimate();
  return normalizeNonNegativeInteger(estimate?.usage ?? 0, "navigator.storage.estimate().usage");
}

async function resetModuleCacheDatabase() {
  await new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(MODULE_CACHE_DB_NAME);
    request.onsuccess = () => resolve();
    request.onerror = () => {
      reject(request.error ?? new Error("failed to delete module cache database"));
    };
    request.onblocked = () => {
      reject(new Error("module cache database deletion was blocked"));
    };
  });
}

function roundDurationMs(durationMs) {
  return Math.round(durationMs);
}

function normalizeNonNegativeInteger(value, label) {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`${label} must be a non-negative finite number, got ${value}`);
  }
  return Math.round(value);
}

function formatError(error) {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  return String(error);
}

function representativeModuleGraphRequest() {
  const modulePath = "example.com/perf/module";
  const version = "v1.0.0";
  return {
    kind: "load_module_graph",
    modules: [
      {
        module_path: modulePath,
        version,
        fetch_url: `data:application/json,${encodeURIComponent(
          JSON.stringify({
            module: {
              module_path: modulePath,
              version,
            },
            files: [
              {
                path: "go.mod",
                contents: `module ${modulePath}\n\ngo 1.21\n`,
              },
              {
                path: "perf/perf.go",
                contents: `package perf

func Label() string {
\treturn "perf"
}
`,
              },
            ],
          }),
        )}`,
      },
    ],
  };
}

function helloWorldProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import "fmt"
func main() { fmt.Println("hello") }
`,
      },
    ],
  };
}

function compileFailureProgram() {
  return {
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
  };
}

function gcChurnProgram() {
  return {
    kind: "run",
    entry_path: "main.go",
    files: [
      {
        path: "main.go",
        contents: `package main
import "fmt"

type Node struct {
  value int
  next *Node
}

type Reader interface {
  Read() int
}

type NodeReader struct {
  node *Node
  bias int
}

func (r NodeReader) Read() int {
  return r.node.value + r.bias
}

func makeClosure(base int, node *Node) func() int {
  values := map[string]int{"next": node.next.value}
  return func() int {
    return base + values["next"]
  }
}

func main() {
  first := &Node{value: 2}
  second := &Node{value: 5}
  first.next = second
  second.next = first

  ch := make(chan int, 2)
  ch <- makeClosure(first.value, first)()
  ch <- second.next.value

  var reader Reader = NodeReader{node: second, bias: <-ch}
  fmt.Println("gc-churn-ok", reader.Read(), <-ch)

}
`,
      },
    ],
  };
}

async function measureBrowserShellInteraction() {
  const doc = await loadShellFrame();

  const readyStart = performance.now();
  await waitForShellReady(doc);
  const shellReadyMs = roundDurationMs(performance.now() - readyStart);

  const runStart = performance.now();
  const runButton = control(doc, "run-button");
  runButton.click();
  await waitFor(
    () =>
      control(doc, "status").textContent === "Worker responded"
        && control(doc, "output").textContent.includes("Rust engine next"),
    20000,
    doc,
    "browser shell run response",
  );
  const shellRunMs = roundDurationMs(performance.now() - runStart);

  return {
    shellReadyMs,
    shellRunMs,
  };
}

async function loadShellFrame() {
  shellFrame.src = `./index.html?measure-worker=${Date.now()}`;
  await waitForFrameLoad(shellFrame, 20000);
  return shellFrame.contentDocument;
}

async function unloadShellFrame() {
  shellFrame.src = "about:blank";
  await waitForFrameLoad(shellFrame, 5000);
}

async function waitForShellReady(doc) {
  await waitFor(
    () =>
      control(doc, "status").textContent.startsWith("Worker ready:")
        && !control(doc, "run-button").disabled,
    20000,
    doc,
    "browser shell ready state",
  );
}

function control(doc, id) {
  const element = doc.getElementById(id);
  if (!element) {
    throw new Error(`missing browser shell control #${id}`);
  }
  return element;
}

async function waitForFrameLoad(frame, timeoutMs) {
  await new Promise((resolve, reject) => {
    const timer = self.setTimeout(() => {
      frame.removeEventListener("load", onLoad);
      reject(new Error("timed out waiting for browser shell frame load"));
    }, timeoutMs);

    function onLoad() {
      self.clearTimeout(timer);
      resolve();
    }

    frame.addEventListener("load", onLoad, { once: true });
  });
}

async function waitFor(predicate, timeoutMs, doc, description) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (predicate()) {
      return;
    }
    await delay(25);
  }
  throw new Error(`${description} timed out\n${shellSnapshot(doc)}`);
}

function shellSnapshot(doc) {
  return JSON.stringify(
    {
      status: control(doc, "status").textContent,
      output: control(doc, "output").textContent,
      selected_file: control(doc, "file-list").value,
      entry_path: control(doc, "entry-path-input").value,
      package_target: control(doc, "package-target-input").value,
    },
    null,
    2,
  );
}

function delay(durationMs) {
  return new Promise((resolve) => setTimeout(resolve, durationMs));
}
