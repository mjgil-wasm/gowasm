import {
  createWorkerShellBackend,
  SUPPORTED_SHELL_BACKEND_ID,
} from "./browser-shell-backend.js";

export function testWorkerShellBackendBootAndRequestPath({ assert }) {
  const events = [];
  const worker = createFakeWorker();
  const backend = createWorkerShellBackend({
    cancellationTimeoutView(requestKind) {
      return {
        statusText: `timeout ${requestKind}`,
        readySuffix: "restarted after timeout",
        outputText: "request timed out",
      };
    },
    clearTimer() {},
    createWorker() {
      return worker;
    },
    deferResponse(callback) {
      callback();
    },
    isCancellableRequestKind(requestKind) {
      return requestKind === "run";
    },
    onBooting(event) {
      events.push({ type: "booting", ...event });
    },
    onCancellationPending() {},
    onCancellationTimeout() {},
    onReady(event) {
      events.push({ type: "ready", ...event });
    },
    onResponse(event) {
      events.push({ type: "response", ...event });
    },
    onStateChange() {},
    onWorkerError(event) {
      events.push({ type: "worker_error", ...event });
    },
    setTimer() {
      return 1;
    },
  });

  backend.boot("Booting worker...");
  worker.emitMessage({
    kind: "ready",
    info: { engine_name: "gowasm", protocol_version: 12 },
  });
  const runRequest = {
    kind: "run",
    entry_path: "main.go",
    files: [{ path: "main.go", contents: "package main\n" }],
  };
  const sent = backend.send("run", runRequest);
  worker.emitMessage({
    kind: "run_result",
    stdout: "ok\n",
    diagnostics: [],
  });

  assert(
    backend.backendId === SUPPORTED_SHELL_BACKEND_ID
      && worker.messages[0]?.kind === "boot",
    "worker shell backend keeps the current worker backend id and boot request path",
    JSON.stringify({ backendId: backend.backendId, messages: worker.messages }),
  );
  assert(
    sent === true && worker.messages[1] === runRequest,
    "worker shell backend forwards run requests through the unchanged worker payload path",
    JSON.stringify(worker.messages),
  );
  assert(
    events.some((event) => event.type === "ready" && event.info?.engine_name === "gowasm")
      && events.some(
        (event) => event.type === "response"
          && event.requestKind === "run"
          && event.response?.kind === "run_result",
      ),
    "worker shell backend reports ready and run_result events without changing response kinds",
    JSON.stringify(events),
  );
}

export function testWorkerShellBackendCancellationTimeoutRestart({ assert }) {
  const events = [];
  const workers = [];
  let timeoutCallback = null;

  const backend = createWorkerShellBackend({
    cancellationTimeoutView(requestKind) {
      return {
        statusText: `Timeout while cancelling ${requestKind}`,
        readySuffix: "restarted after timeout",
        outputText: "Execution cancelled after restart timeout.",
      };
    },
    clearTimer() {
      timeoutCallback = null;
    },
    createWorker() {
      const worker = createFakeWorker();
      workers.push(worker);
      return worker;
    },
    deferResponse(callback) {
      callback();
    },
    isCancellableRequestKind(requestKind) {
      return requestKind === "run";
    },
    onBooting(event) {
      events.push({ type: "booting", ...event });
    },
    onCancellationPending(event) {
      events.push({ type: "cancellation_pending", ...event });
    },
    onCancellationTimeout(event) {
      events.push({ type: "cancellation_timeout", ...event });
    },
    onReady() {},
    onResponse() {},
    onStateChange() {},
    onWorkerError() {},
    setTimer(callback) {
      timeoutCallback = callback;
      return 9;
    },
  });

  backend.boot("Booting worker...");
  workers[0].emitMessage({
    kind: "ready",
    info: { engine_name: "gowasm", protocol_version: 12 },
  });
  backend.send("run", {
    kind: "run",
    entry_path: "main.go",
    files: [{ path: "main.go", contents: "package main\n" }],
  });
  const cancelled = backend.cancel();
  timeoutCallback?.();

  assert(
    cancelled === true && workers[0].messages.at(-1)?.kind === "cancel",
    "worker shell backend keeps the current cancel request payload path",
    JSON.stringify(workers[0].messages),
  );
  assert(
    workers[0].terminated === true
      && workers[1]?.messages[0]?.kind === "boot"
      && backend.currentState().backendId === SUPPORTED_SHELL_BACKEND_ID,
    "worker shell backend restarts through the same supported worker backend after cancel timeout",
    JSON.stringify({
      firstWorkerTerminated: workers[0].terminated,
      secondWorkerMessages: workers[1]?.messages ?? [],
      state: backend.currentState(),
    }),
  );
  assert(
    events.some((event) => event.type === "cancellation_pending" && event.requestKind === "run")
      && events.some(
        (event) => event.type === "cancellation_timeout"
          && event.timeoutView?.readySuffix === "restarted after timeout",
      ),
    "worker shell backend reports cancellation timeout events without widening the current backend surface",
    JSON.stringify(events),
  );
}

function createFakeWorker() {
  const listeners = new Map();
  return {
    messages: [],
    terminated: false,
    addEventListener(type, listener) {
      const callbacks = listeners.get(type) ?? [];
      callbacks.push(listener);
      listeners.set(type, callbacks);
    },
    emitMessage(data) {
      for (const listener of listeners.get("message") ?? []) {
        listener({ data });
      }
    },
    postMessage(message) {
      this.messages.push(message);
    },
    terminate() {
      this.terminated = true;
    },
  };
}
