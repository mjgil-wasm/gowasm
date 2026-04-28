import {
  normalizeModuleRelativePath,
  validateWasmBufferWindow,
  validateWorkerRequestEnvelope,
} from "./browser-capability-security.js";
import { importProjectArchiveUrl } from "./browser-archive.js";
import { formatDiagnostics } from "./browser-shell-output.js";
import { renderDiagnosticSourcePanel } from "./browser-shell-source-links.js";
import {
  lookupModuleCache,
  MODULE_CACHE_SCHEMA_VERSION,
} from "./engine-worker-module-cache.js";
import { createArchiveDataUrl } from "./test-browser-archive-fixtures.js";
import {
  seedModuleCacheRecord,
} from "./test-browser-shell-cache-fixtures.js";
import { resetModuleCacheDatabase } from "./test-worker-modules.js";

export async function runBrowserCapabilitySecurityTests({ assert }) {
  await testArchivePathTraversal(assert);
  testModulePathNormalization(assert);
  testWasmBufferBounds(assert);
  testWorkerEnvelopeValidation(assert);
  await testMalformedWorkerMessages(assert);
  await testModuleCachePoisoning(assert);
  testDiagnosticRenderingEscapes(assert);
}

async function testArchivePathTraversal(assert) {
  let message = "";
  try {
    await importProjectArchiveUrl(
      createArchiveDataUrl([{ path: "../escape.go", contents: "package main\n" }]),
    );
  } catch (error) {
    message = error?.message || String(error);
  }
  assert(
    message.includes("path traversal entry"),
    "archive import rejects traversal entries",
    message || "expected archive traversal rejection",
  );
}

function testModulePathNormalization(assert) {
  assert(
    normalizeModuleRelativePath("./pkg/hello.go") === "pkg/hello.go",
    "module path normalization preserves safe relative paths",
    "safe module path did not normalize as expected",
  );

  let rejection = "";
  try {
    normalizeModuleRelativePath("../escape.go");
  } catch (error) {
    rejection = error?.message || String(error);
  }
  assert(
    rejection.includes("path traversal"),
    "module path normalization rejects traversal",
    rejection || "expected traversal rejection",
  );
}

function testWasmBufferBounds(assert) {
  validateWasmBufferWindow({ buffer: new ArrayBuffer(16) }, 2, 4, "response buffer");
  assert(true, "wasm buffer bounds allow valid windows", "");

  let rejection = "";
  try {
    validateWasmBufferWindow({ buffer: new ArrayBuffer(8) }, 4, 8, "response buffer");
  } catch (error) {
    rejection = error?.message || String(error);
  }
  assert(
    rejection.includes("exceeded wasm memory bounds"),
    "wasm buffer bounds reject out-of-range windows",
    rejection || "expected wasm bounds rejection",
  );
}

function testWorkerEnvelopeValidation(assert) {
  validateWorkerRequestEnvelope({ kind: "boot" });
  validateWorkerRequestEnvelope({
    kind: "run",
    entry_path: "main.go",
    files: [{ path: "main.go", contents: "package main\nfunc main() {}\n" }],
  });
  assert(true, "worker envelope validation accepts supported request shapes", "");

  let rejection = "";
  try {
    validateWorkerRequestEnvelope({ kind: "load_module_graph", modules: [{ module_path: "" }] });
  } catch (error) {
    rejection = error?.message || String(error);
  }
  assert(
    rejection.includes("load_module_graph.modules[0].module_path"),
    "worker envelope validation rejects malformed module requests",
    rejection || "expected malformed module request rejection",
  );
}

async function testMalformedWorkerMessages(assert) {
  const scalarResult = await sendWorkerMessage(42);
  assert(
    scalarResult.kind === "fatal" &&
      String(scalarResult.message || "").includes("worker request must be a JSON object"),
    "worker rejects malformed scalar messages with a fatal response",
    JSON.stringify(scalarResult),
  );

  const unknownKindResult = await sendWorkerMessage({ kind: "<img src=x onerror=1>" });
  assert(
    unknownKindResult.kind === "fatal" &&
      String(unknownKindResult.message || "").includes("unsupported worker request kind"),
    "worker rejects unsupported request kinds with a fatal response",
    JSON.stringify(unknownKindResult),
  );
}

async function testModuleCachePoisoning(assert) {
  const modulePath = `example.com/security/cache-${Date.now()}`;
  const version = "v1.2.3";
  await resetModuleCacheDatabase();
  await seedModuleCacheRecord({
    cache_key: `${modulePath}@${version}`,
    schema_version: MODULE_CACHE_SCHEMA_VERSION,
    stored_at_ms: Date.now(),
    module: { module_path: modulePath, version },
    origin_url: "https://example.invalid/module.json",
    files: [{ path: "../escape.go", contents: "package main\n" }],
  });

  const result = await lookupModuleCache({
    module: { module_path: modulePath, version },
  });
  assert(
    result.kind === "miss",
    "poisoned module cache entries are rejected before reuse",
    JSON.stringify(result),
  );
}

function testDiagnosticRenderingEscapes(assert) {
  globalThis.__gowasmSecurityHit = 0;
  const output = document.createElement("pre");
  const linksElement = document.createElement("div");
  const panelElement = document.createElement("section");
  const malicious = '<img src=x onerror="globalThis.__gowasmSecurityHit = 1">';
  output.textContent = formatDiagnostics([
    {
      file_path: "main.go",
      message: `${malicious}\nsecondary line`,
      source_span: {
        start: { line: 1, column: 1 },
        end: { line: 1, column: 5 },
      },
      source_excerpt: {
        line: 1,
        text: malicious,
        highlight_start_column: 1,
        highlight_end_column: 5,
      },
      runtime: {
        root_message: malicious,
        stack_trace: [
          {
            function: malicious,
            source_location: { path: "main.go", line: 1, column: 1 },
          },
        ],
      },
    },
  ]);
  renderDiagnosticSourcePanel({
    diagnostics: [
      {
        file_path: "main.go",
        message: malicious,
        source_span: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 5 },
        },
      },
    ],
    files: [{ path: "main.go", contents: "package main\n" }],
    linksElement,
    onSelectLink() {},
    panelElement,
  });

  assert(
    output.innerHTML.includes("&lt;img") &&
      !output.querySelector("img") &&
      !linksElement.querySelector("img") &&
      globalThis.__gowasmSecurityHit === 0,
    "diagnostic rendering keeps hostile HTML as plain text",
    `${output.innerHTML}\n${linksElement.innerHTML}`,
  );
}

function sendWorkerMessage(message) {
  const worker = new Worker("./engine-worker.js", { type: "module" });
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      worker.terminate();
      reject(new Error("timed out waiting for worker reply"));
    }, 10000);

    worker.addEventListener("message", ({ data }) => {
      clearTimeout(timer);
      worker.terminate();
      resolve(data);
    });
    worker.postMessage(message);
  });
}
