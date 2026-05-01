export const SUPPORTED_SHELL_BACKEND_ID = "worker";

export function createWorkerShellBackend(ctx) {
  let worker = null;
  let workerGeneration = 0;
  let ready = false;
  let activeRequestKind = null;
  let cancelPending = false;
  let cancelFallbackTimer = null;

  function currentState() {
    return {
      backendId: SUPPORTED_SHELL_BACKEND_ID,
      ready,
      activeRequestKind,
      cancelPending,
    };
  }

  function emitStateChange() {
    ctx.onStateChange?.(currentState());
  }

  function clearCancelFallback() {
    if (cancelFallbackTimer === null) {
      return;
    }
    ctx.clearTimer(cancelFallbackTimer);
    cancelFallbackTimer = null;
  }

  function resetRequestState() {
    activeRequestKind = null;
    cancelPending = false;
    clearCancelFallback();
  }

  function boot(statusMessage, readySuffix = "") {
    workerGeneration += 1;
    if (worker) {
      worker.terminate();
      worker = null;
    }
    ready = false;
    resetRequestState();
    ctx.onBooting?.({ statusMessage });
    emitStateChange();

    const generation = workerGeneration;
    const nextWorker = ctx.createWorker();
    nextWorker.addEventListener("message", ({ data }) => {
      if (generation !== workerGeneration) {
        return;
      }

      ctx.deferResponse(() => {
        if (generation !== workerGeneration) {
          return;
        }

        cancelPending = false;
        clearCancelFallback();

        if (data.kind === "ready") {
          ready = true;
          emitStateChange();
          ctx.onReady?.({ info: data.info, readySuffix });
          return;
        }

        const requestKind = activeRequestKind;
        activeRequestKind = null;
        emitStateChange();
        ctx.onResponse?.({ requestKind, response: data });
      });
    });

    nextWorker.addEventListener("error", (event) => {
      if (generation !== workerGeneration) {
        return;
      }

      const requestKind = activeRequestKind;
      ready = false;
      resetRequestState();
      emitStateChange();
      ctx.onWorkerError?.({
        requestKind,
        message: event.message || "Worker error",
        filename: event.filename || "",
        lineno: event.lineno || 0,
        colno: event.colno || 0,
        stackText: event.error?.stack || "",
      });
    });

    nextWorker.postMessage({ kind: "boot" });
    worker = nextWorker;
  }

  function send(requestKind, request) {
    if (!worker || !ready || activeRequestKind !== null) {
      return false;
    }
    activeRequestKind = requestKind;
    emitStateChange();
    worker.postMessage(request);
    return true;
  }

  function cancel() {
    if (
      !worker ||
      !activeRequestKind ||
      cancelPending ||
      !ctx.isCancellableRequestKind(activeRequestKind)
    ) {
      return false;
    }

    const cancellingRequestKind = activeRequestKind;
    cancelPending = true;
    emitStateChange();
    ctx.onCancellationPending?.({ requestKind: cancellingRequestKind });
    worker.postMessage({ kind: "cancel" });

    clearCancelFallback();
    cancelFallbackTimer = ctx.setTimer(() => {
      if (!activeRequestKind) {
        return;
      }

      worker?.terminate();
      worker = null;
      ready = false;
      resetRequestState();
      emitStateChange();

      const timeoutView = ctx.cancellationTimeoutView(cancellingRequestKind);
      ctx.onCancellationTimeout?.({
        requestKind: cancellingRequestKind,
        timeoutView,
      });
      boot(timeoutView.statusText, timeoutView.readySuffix);
    }, 150);

    return true;
  }

  return {
    backendId: SUPPORTED_SHELL_BACKEND_ID,
    boot,
    cancel,
    currentState,
    send,
  };
}
