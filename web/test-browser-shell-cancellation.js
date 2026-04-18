import { resetModuleCacheDatabase } from "./test-worker-modules.js";
import {
  click,
  clickAndWaitFor,
  control,
  loadShellFrame,
  setInputValue,
  setPlainValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellCancellationRecovery({ assert, frame, log }) {
  log("\n--- browser shell cancellation and recovery harness ---");

  const seed = Date.now();
  const remoteModulePath = `example.com/remote/browser-cancel-${seed}`;
  const firstVersion = "v2.0.0";
  const secondVersion = "v2.0.1";

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await setInputValue(
      doc,
      "module-roots",
      moduleRootLine(
        remoteModulePath,
        firstVersion,
        moduleBundleFetchUrl({
          modulePath: remoteModulePath,
          version: firstVersion,
          message: "cancel-remote",
        }),
      ),
    );
    await setWorkspaceFileContents(
      doc,
      "main.go",
      sleepingMainSource(remoteModulePath, 60000, "run-done"),
    );

    click(doc, "run-button");
    await waitForCancellationReady(doc, "run cancellation");
    await delay(150);
    click(doc, "cancel-button");
    await waitForCancellationMessage(doc, "Execution cancelled.", "run cancellation");
    assert(
      control(doc, "status").textContent === "Run cancelled"
        && control(doc, "module-status").textContent.includes(
          `${remoteModulePath}@${firstVersion}`,
        ),
      "browser shell run cancellation preserves recovery-ready module state after autoload",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      sleepingMainSource(remoteModulePath, 5, "run-done"),
    );
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("run-done cancel-remote"),
      "run recovery request",
    );
    assert(
      control(doc, "output").textContent.includes("run-done cancel-remote"),
      "browser shell run recovers after a cancelled paused host wait",
      shellSnapshot(doc),
    );

    await setInputValue(
      doc,
      "module-roots",
      moduleRootLine(
        remoteModulePath,
        secondVersion,
        moduleBundleFetchUrl({
          modulePath: remoteModulePath,
          version: secondVersion,
          message: "snippet-remote",
        }),
      ),
    );
    await setWorkspaceFileContents(
      doc,
      "main.go",
      sleepingMainSource(remoteModulePath, 60000, "snippet-done"),
    );

    click(doc, "test-snippet-button");
    await waitForCancellationReady(doc, "snippet cancellation");
    await delay(150);
    click(doc, "cancel-button");
    await waitForCancellationMessage(doc, "Snippet test cancelled.", "snippet cancellation");
    assert(
      control(doc, "status").textContent === "Snippet test cancelled"
        && control(doc, "module-status").textContent.includes(
          `${remoteModulePath}@${secondVersion}`,
        ),
      "browser shell snippet cancellation still recovers after an autoloaded module change",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      sleepingMainSource(remoteModulePath, 5, "snippet-done"),
    );
    await clickAndWaitFor(
      doc,
      "test-snippet-button",
      () =>
        control(doc, "status").textContent.includes("Snippet test passed")
          && control(doc, "output").textContent.includes("snippet-done snippet-remote"),
      "snippet recovery request",
    );
    assert(
      control(doc, "output").textContent.includes("Snippet test passed.")
        && control(doc, "output").textContent.includes("snippet-done snippet-remote"),
      "browser shell snippet flow recovers after a cancelled paused host wait",
      shellSnapshot(doc),
    );

    await addWorkspaceFile(doc, "calc/calc.go", sleepingCalcSource(60000));
    await addWorkspaceFile(doc, "calc/calc_test.go", calcTestSource());
    await setPlainValue(doc, "package-target-input", "calc/calc.go");

    click(doc, "test-package-button");
    await waitForCancellationReady(doc, "package-test cancellation");
    await delay(150);
    click(doc, "cancel-button");
    await waitForCancellationMessage(doc, "Package test cancelled.", "package-test cancellation");
    assert(
      control(doc, "status").textContent === "Package test cancelled",
      "browser shell package-test cancellation reports the dedicated recovery message",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(doc, "calc/calc.go", sleepingCalcSource(5));
    await clickAndWaitFor(
      doc,
      "test-package-button",
      () =>
        control(doc, "status").textContent.includes("Package tests passed")
          && control(doc, "output").textContent.includes("PASS TestAdd"),
      "package-test recovery request",
    );
    assert(
      control(doc, "output").textContent.includes("Completed: TestAdd"),
      "browser shell package-test flow recovers after cancellation",
      shellSnapshot(doc),
    );

    const slowFetchUrl = `${doc.defaultView.location.origin}/__gowasm_test_delay?ms=60000&body=http-cancel`;
    const quickFetchUrl = `${doc.defaultView.location.origin}/__gowasm_test_delay?ms=10&body=http-ok`;
    await setWorkspaceFileContents(doc, "main.go", httpWaitMainSource(slowFetchUrl));

    click(doc, "run-button");
    await waitForCancellationReady(doc, "http cancellation");
    await delay(150);
    click(doc, "cancel-button");
    await waitForCancellationMessage(doc, "Execution cancelled.", "http cancellation");
    assert(
      control(doc, "status").textContent === "Run cancelled",
      "browser shell HTTP wait cancellation reports the run-cancelled recovery message",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(doc, "main.go", httpWaitMainSource(quickFetchUrl));
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("200 http-ok"),
      "http recovery request",
    );
    assert(
      control(doc, "output").textContent.includes("200 http-ok"),
      "browser shell HTTP path recovers after cancellation",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(doc, "main.go", compileHeavyMainSource());
    doc.defaultView.__gowasmDelayNextWorkerResponse?.(500);
    click(doc, "run-button");
    await waitForCancellationReady(doc, "compile cancellation");
    click(doc, "cancel-button");
    await waitForRestartRecovery(doc, "compile cancellation");
    assert(
      control(doc, "status").textContent.includes("recovered after cancellation timeout")
        && control(doc, "output").textContent.includes(
          "Run cancellation timed out before the worker yielded. Restarting worker for recovery…",
        ),
      "browser shell compile cancellation restarts the worker with a visible recovery message",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(doc, "main.go", simpleMainSource("compile-recovery"));
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("compile-recovery"),
      "compile recovery request",
    );
    assert(
      control(doc, "output").textContent.includes("compile-recovery"),
      "browser shell run flow recovers after forced worker restart",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await delay(50);
    await resetModuleCacheDatabase();
  }
}

async function addWorkspaceFile(doc, path, contents) {
  await setInputValue(doc, "file-path-input", path);
  click(doc, "add-file-button");
  await waitFor(
    () => Array.from(control(doc, "file-list").options).some((option) => option.value === path),
    `workspace file ${path} added`,
    doc,
  );
  await setWorkspaceFileContents(doc, path, contents);
}

async function waitForCancellationReady(doc, description) {
  await waitFor(
    () => !control(doc, "cancel-button").disabled,
    `${description} became available`,
    doc,
  );
}

async function waitForCancellationMessage(doc, expectedText, description) {
  await waitFor(
    () =>
      !control(doc, "run-button").disabled
        && control(doc, "output").textContent.includes(expectedText),
    `${description} completed`,
    doc,
  );
}

async function waitForRestartRecovery(doc, description) {
  await waitFor(
    () =>
      !control(doc, "run-button").disabled
        && control(doc, "status").textContent.includes("recovered after cancellation timeout")
        && control(doc, "output").textContent.includes("Restarting worker for recovery"),
    `${description} restarted the worker`,
    doc,
  );
}

function sleepingMainSource(remoteModulePath, durationMs, label) {
  return `package main

import (
\t"fmt"
\t"time"
\t"${remoteModulePath}/greeter"
)

func main() {
\ttime.Sleep(${durationMs * 1_000_000})
\tfmt.Println("${label}", greeter.Message())
}
`;
}

function sleepingCalcSource(durationMs) {
  return `package calc

import "time"

func Add(left int, right int) int {
\ttime.Sleep(${durationMs * 1_000_000})
\treturn left + right
}
`;
}

function calcTestSource() {
  return `package calc

func TestAdd() {
\tif Add(2, 3) != 5 {
\t\tpanic("expected Add to sum inputs")
\t}
}
`;
}

function httpWaitMainSource(url) {
  return `package main

import (
\t"fmt"
\t"net/http"
)

func main() {
\tresp, err := http.Get(${JSON.stringify(url)})
\tif err != nil {
\t\tfmt.Println("fetch-failed", err)
\t\treturn
\t}
\tdefer resp.Body.Close()
\tbuf := make([]byte, 64)
\tn, err := resp.Body.Read(buf)
\tif err != nil && err.Error() != "EOF" {
\t\tfmt.Println("read-failed", err)
\t\treturn
\t}
\tfmt.Println(resp.StatusCode, string(buf[:n]))
}
`;
}

function simpleMainSource(label) {
  return `package main

import "fmt"

func main() {
\tfmt.Println("${label}")
}
`;
}

function compileHeavyMainSource() {
  let source = "package main\n\n";
  for (let index = 0; index < 400000; index += 1) {
    source += `func generatedValue${index}() int { return ${index} }\n`;
  }
  source += "\nfunc main() {\n\t_ = 0\n}\n";
  return source;
}

function moduleBundleFetchUrl({ modulePath, version, message }) {
  return `data:application/json,${encodeURIComponent(
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
          path: "greeter/greeter.go",
          contents: `package greeter

func Message() string {
\treturn "${message}"
}
`,
        },
      ],
    }),
  )}`;
}

function moduleRootLine(modulePath, version, fetchUrl) {
  return `${modulePath} ${version} ${fetchUrl}`;
}

function delay(durationMs) {
  return new Promise((resolve) => self.setTimeout(resolve, durationMs));
}
