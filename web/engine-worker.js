const decoder = new TextDecoder();
const encoder = new TextEncoder();

let enginePromise;
let activeRunState = null;
let workerRuntimePromise = null;

self.addEventListener("message", ({ data }) => {
  void handleWorkerMessage(data);
});

async function handleWorkerMessage(data) {
  if (data?.kind === "cancel") {
    const cancelled = await cancelActiveRun();
    if (cancelled) {
      self.postMessage({ kind: "cancelled" });
    }
    return;
  }

  try {
    const { formatError, validateWorkerRequestEnvelope } = await loadWorkerRuntime();
    validateWorkerRequestEnvelope(data);
    const engine = await loadEngine();
    const response = await sendWorkerRequest(engine, data);
    if (response) {
      self.postMessage(response);
    }
  } catch (error) {
    if (isCancelledRunError(error)) {
      return;
    }
    const formatter = await loadWorkerRuntime()
      .then((runtime) => runtime.formatError)
      .catch(() => (fallbackError) => fallbackError?.message || String(fallbackError));
    self.postMessage({
      kind: "fatal",
      message: formatter(error),
    });
  }
}

async function sendWorkerRequest(engine, request) {
  if (!isExecutionRequest(request)) {
    return engine.send(request);
  }

  const runState = {
    cancelled: false,
    abortCapability: null,
    cancelResponse: null,
    engine,
    fetchSessions: new Map(),
  };
  activeRunState = runState;
  try {
    return await engine.send(request, runState);
  } finally {
    if (activeRunState === runState) {
      activeRunState = null;
    }
    const { disposeFetchSessions } = await loadWorkerRuntime();
    disposeFetchSessions(runState);
    runState.abortCapability = null;
  }
}

function isExecutionRequest(request) {
  return (
    request?.kind === "compile" ||
    request?.kind === "run" ||
    request?.kind === "test_package" ||
    request?.kind === "test_snippet"
  );
}

async function cancelActiveRun() {
  const runState = activeRunState;
  if (!runState) {
    return false;
  }

  runState.cancelled = true;
  if (typeof runState.abortCapability === "function") {
    runState.abortCapability();
    runState.abortCapability = null;
  }
  if (!runState.cancelResponse) {
    runState.cancelResponse = Promise.resolve(runState.engine.cancel());
  }
  await runState.cancelResponse;
  return true;
}

async function loadEngine() {
  if (!enginePromise) {
    enginePromise = createEngine();
  }
  return enginePromise;
}

async function createEngine() {
  const { buildCapabilityResumeRequest, withHostClock, buildModuleResumeRequest } =
    await loadWorkerRuntime();
  const url = new URL("./generated/gowasm_engine_wasm.wasm", self.location.href);
  const bytes = await fetch(url).then(async (response) => {
    if (!response.ok) {
      throw new Error(
        `failed to fetch ${url.pathname}: ${response.status} ${response.statusText}`,
      );
    }
    return response.arrayBuffer();
  });

  const { instance } = await WebAssembly.instantiate(bytes, {});
  const exports = instance.exports;
  if (
    typeof exports.alloc_request_buffer !== "function" ||
    typeof exports.free_request_buffer !== "function" ||
    typeof exports.handle_request !== "function" ||
    typeof exports.response_ptr !== "function" ||
    typeof exports.response_len !== "function" ||
    typeof exports.free_response_buffer !== "function" ||
    !(exports.memory instanceof WebAssembly.Memory)
  ) {
    throw new Error("wasm engine exports were incomplete");
  }

  return {
    async send(request, runState = null) {
      let pendingRequest = withHostClock(request);
      for (;;) {
        if (runState?.cancelled) {
          throw new CancelledRunError();
        }
        const response = sendJsonRequest(exports, pendingRequest);
        if (response?.kind === "capability_request") {
          pendingRequest = await buildCapabilityResumeRequest(response, runState);
          continue;
        }
        if (response?.kind === "module_request") {
          pendingRequest = await buildModuleResumeRequest(response);
          continue;
        }
        return response;
      }
    },
    cancel() {
      return sendJsonRequest(exports, { kind: "cancel" });
    },
  };
}

async function loadWorkerRuntime() {
  if (!workerRuntimePromise) {
    const v = self.location.search || "";
    workerRuntimePromise = Promise.all([
      import(`./engine-worker-runtime.js${v}`),
      import(`./browser-capability-security.js${v}`),
      import(`./engine-worker-modules.js${v}`),
    ]).then(([runtime, security, modules]) => ({
      buildCapabilityResumeRequest: runtime.buildCapabilityResumeRequest,
      buildModuleResumeRequest: modules.buildModuleResumeRequest,
      disposeFetchSessions: runtime.disposeFetchSessions,
      formatError: runtime.formatError,
      validateWasmBufferWindow: security.validateWasmBufferWindow,
      validateWorkerRequestEnvelope: security.validateWorkerRequestEnvelope,
      withHostClock: runtime.withHostClock,
    }));
  }
  return workerRuntimePromise;
}

function isCancelledRunError(error) {
  return (
    error?.name === "CancelledRunError" ||
    error?.constructor?.name === "CancelledRunError"
  );
}

function sendJsonRequest(exports, request) {
  const payload = encoder.encode(JSON.stringify(request));
  const ptr = exports.alloc_request_buffer(payload.length);
  try {
    new Uint8Array(exports.memory.buffer, ptr, payload.length).set(payload);

    const status = exports.handle_request(ptr, payload.length);
    const responsePtr = exports.response_ptr();
    const responseLen = exports.response_len();
    const memory = exports.memory;
    if (!(memory?.buffer instanceof ArrayBuffer)) {
      throw new Error("response buffer memory was unavailable");
    }
    const end = responsePtr + responseLen;
    if (
      !Number.isSafeInteger(responsePtr) ||
      responsePtr < 0 ||
      !Number.isSafeInteger(responseLen) ||
      responseLen < 0 ||
      !Number.isSafeInteger(end) ||
      end > memory.buffer.byteLength
    ) {
      throw new Error("response buffer exceeded wasm memory bounds");
    }
    const responseBytes = new Uint8Array(exports.memory.buffer, responsePtr, responseLen).slice();
    const response = JSON.parse(decoder.decode(responseBytes));
    exports.free_response_buffer(responsePtr, responseLen);

    if (status !== 0) {
      throw new Error(response?.message || "wasm engine reported a fatal bridge error");
    }

    return response;
  } finally {
    exports.free_request_buffer(ptr, payload.length);
  }
}
