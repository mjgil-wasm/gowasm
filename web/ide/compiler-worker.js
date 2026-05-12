/**
 * IDE compiler worker — thin wrapper around the gowasm engine worker.
 *
 * Accepts messages:
 *   { action: "build", files: [{ path, contents }], entry_path: string }
 *   { action: "compile", files, entry_path }
 *   { action: "cancel" }
 *
 * Posts back:
 *   { kind: "ready" }
 *   { kind: "build_result", success, wasmBytes?, diagnostics? }
 *   { kind: "compile_result", success, diagnostics? }
 *   { kind: "fatal", message }
 */

const ENGINE_URL = new URL("../engine-worker.js", self.location.href);
ENGINE_URL.searchParams.set("cb", Date.now());
const ENGINE_URL_HREF = ENGINE_URL.href;
let engine = null;
let engineReady = false;
let pendingBuild = null;

self.addEventListener("message", ({ data }) => {
  if (!data || typeof data !== "object") return;

  if (data.action === "cancel") {
    if (engine) engine.postMessage({ kind: "cancel" });
    return;
  }

  if (data.action === "build" || data.action === "compile") {
    if (!engineReady) {
      pendingBuild = data;
      return;
    }
    sendEngineRequest(data);
    return;
  }
});

function boot() {
  engine = new Worker(ENGINE_URL_HREF, { type: "module" });
  engine.addEventListener("message", ({ data }) => {
    if (data?.kind === "ready") {
      engineReady = true;
      self.postMessage({ kind: "ready" });
      if (pendingBuild) {
        sendEngineRequest(pendingBuild);
        pendingBuild = null;
      }
      return;
    }
    // Forward engine responses to the IDE
    self.postMessage(data);
  });
  engine.addEventListener("error", (e) => {
    self.postMessage({ kind: "fatal", message: e.message || "compiler worker error" });
  });
  engine.postMessage({ kind: "boot" });
}

function sendEngineRequest(req) {
  if (req.action === "build") {
    // Build = compile + return wasm (the engine doesn't directly expose wasm bytes,
    // so we compile to verify, then the IDE can request a run.)
    engine.postMessage({
      kind: "compile",
      entry_path: req.entry_path,
      files: req.files,
    });
  } else if (req.action === "compile") {
    engine.postMessage({
      kind: "compile",
      entry_path: req.entry_path,
      files: req.files,
    });
  }
}

boot();
