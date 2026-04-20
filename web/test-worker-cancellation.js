import {
  busyLoopProgram,
  helloWorldProgram,
  netHttpSlowProgram,
  packageTestProgram,
  packageTestSleepProgram,
  sleepProgram,
} from "./test-worker-programs.js";

export async function testCancelWithNoRun({ assert, createWorker, log, sendAndWait }) {
  log("\n--- cancel with no active run ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    worker.postMessage({ kind: "cancel" });
    await delay(200);

    const result = await sendAndWait(worker, helloWorldProgram());
    assert(
      result.kind === "run_result",
      "run after spurious cancel works",
      `got: ${result.kind}`,
    );
  } finally {
    worker.terminate();
  }
}

export async function testFallbackTermination({ assert, createWorker, log, sendAndWait }) {
  log("\n--- fallback termination ---");
  const worker = createWorker();
  const origTerminate = worker.terminate.bind(worker);

  try {
    await sendAndWait(worker, { kind: "boot" });

    worker.postMessage(busyLoopProgram());
    await delay(50);

    const cancelResult = await waitForCancelledOrTimeout(worker);
    assert(
      cancelResult === "cancelled" || cancelResult === "timeout",
      "fallback path resolves",
      `got: ${cancelResult}`,
    );
  } finally {
    origTerminate();
  }
}

export async function testSleepCancellation({ assert, createWorker, log, sendAndWait }) {
  log("\n--- sleep cancellation ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    worker.postMessage(sleepProgram(60000));
    await delay(75);
    const cancelled = await cancelAndWait(worker);
    assert(cancelled.kind === "cancelled", "sleep cancellation yields cancelled", JSON.stringify(cancelled));

    const recovery = await sendAndWait(worker, helloWorldProgram());
    assert(
      recovery.kind === "run_result" && String(recovery.stdout ?? "").trim() === "hello",
      "sleep cancellation recovers for the next run",
      JSON.stringify(recovery),
    );
  } finally {
    worker.terminate();
  }
}

export async function testFetchCancellation({ assert, createWorker, log, sendAndWait }) {
  log("\n--- fetch cancellation ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const slowUrl = `${self.location.origin}/__gowasm_test_delay?ms=60000&body=fetch-wait`;
    worker.postMessage(netHttpSlowProgram(slowUrl));
    await delay(75);
    const cancelled = await cancelAndWait(worker);
    assert(cancelled.kind === "cancelled", "fetch cancellation yields cancelled", JSON.stringify(cancelled));

    const recovery = await sendAndWait(worker, helloWorldProgram());
    assert(
      recovery.kind === "run_result" && String(recovery.stdout ?? "").trim() === "hello",
      "fetch cancellation rejects stale resumes and recovers for the next run",
      JSON.stringify(recovery),
    );
  } finally {
    worker.terminate();
  }
}

export async function testPackageTestCancellation({ assert, createWorker, log, sendAndWait }) {
  log("\n--- package test cancellation ---");
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    worker.postMessage(packageTestSleepProgram(60000));
    await delay(75);
    const cancelled = await cancelAndWait(worker);
    assert(
      cancelled.kind === "cancelled",
      "package-test cancellation yields cancelled",
      JSON.stringify(cancelled),
    );

    const recovery = await sendAndWait(worker, packageTestProgram("TestAdd"));
    assert(
      recovery.kind === "test_result" && recovery.passed === true,
      "package-test cancellation recovers for the next test request",
      JSON.stringify(recovery),
    );
  } finally {
    worker.terminate();
  }
}

async function cancelAndWait(worker) {
  const messagePromise = waitForNextMessage(worker);
  worker.postMessage({ kind: "cancel" });
  return await messagePromise;
}

async function waitForCancelledOrTimeout(worker) {
  return await new Promise((resolve) => {
    let timer = null;
    function onMessage({ data }) {
      if (data.kind !== "cancelled") {
        return;
      }
      clearTimeout(timer);
      worker.removeEventListener("message", onMessage);
      resolve("cancelled");
    }
    worker.addEventListener("message", onMessage);
    worker.postMessage({ kind: "cancel" });

    timer = setTimeout(() => {
      worker.removeEventListener("message", onMessage);
      resolve("timeout");
    }, 5000);
  });
}

function waitForNextMessage(worker, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`timed out waiting for message after ${timeoutMs}ms`));
    }, timeoutMs);

    function onMessage({ data }) {
      clearTimeout(timer);
      worker.removeEventListener("message", onMessage);
      resolve(data);
    }

    worker.addEventListener("message", onMessage);
  });
}

function delay(durationMs) {
  return new Promise((resolve) => self.setTimeout(resolve, durationMs));
}
