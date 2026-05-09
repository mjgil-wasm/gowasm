export function createBrowserShellTestHooks(win) {
  let nextWorkerResponseDelayMs = 0;

  if (win && typeof win === "object") {
    win.__gowasmDelayNextWorkerResponse = (durationMs) => {
      const numeric = Math.trunc(Number(durationMs));
      if (!Number.isFinite(numeric) || numeric <= 0) {
        nextWorkerResponseDelayMs = 0;
        return;
      }
      nextWorkerResponseDelayMs = Math.min(numeric, 5000);
    };
  }

  return {
    maybeDelayNextWorkerResponse(callback) {
      if (nextWorkerResponseDelayMs <= 0) {
        callback();
        return;
      }
      const delayMs = nextWorkerResponseDelayMs;
      nextWorkerResponseDelayMs = 0;
      win.setTimeout(callback, delayMs);
    },
  };
}
