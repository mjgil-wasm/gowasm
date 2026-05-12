import { resetModuleCacheDatabase } from "./test-worker-modules.js";

const ACTION_TIMEOUT_MS = 20000;
const POLL_INTERVAL_MS = 25;

export async function testBrowserShellEndToEnd({ assert, frame, log }) {
  log("\n--- browser shell end-to-end harness ---");

  const seed = Date.now();
  const remoteModulePath = `example.com/remote/browser-shell-${seed}`;
  const brokenModulePath = `example.com/broken/browser-shell-${seed}`;
  const initialVersion = "v1.2.3";
  const refreshedVersion = "v1.2.4";

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await addWorkspaceFile(doc, "data.txt", "alpha");
    await addWorkspaceFile(doc, "calc/calc.go", calcSource(remoteModulePath));
    await addWorkspaceFile(doc, "calc/calc_test.go", calcTestSource("remote"));
    await setWorkspaceFileContents(doc, "main.go", unformattedMainSource(remoteModulePath));

    await clickAndWaitForIdle(doc, "lint-button", "lint request");
    const lintOutput = control(doc, "output").textContent;
    assert(
      lintOutput.includes("run Format"),
      "browser shell lint flow reports formatter drift through the product path",
      shellSnapshot(doc),
    );
    assert(
      lintOutput.includes("never references `strings`"),
      "browser shell lint flow reports unused imports through the product path",
      shellSnapshot(doc),
    );

    await clickAndWaitForIdle(doc, "format-button", "format request");
    const formattedMain = selectedSource(doc);
    assert(
      formattedMain.includes("func joinValue[T any](value T) string {"),
      "browser shell format flow preserves supported generic helpers through the product path",
      shellSnapshot(doc),
    );
    assert(
      formattedMain.includes("\t// normalized comment"),
      "browser shell format flow keeps supported line comments while reindenting generic helpers",
      shellSnapshot(doc),
    );
    assert(
      formattedMain.includes('\tdata, err := os.ReadFile("data.txt")'),
      "browser shell format flow rewrites main.go indentation through the product path",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(doc, "main.go", formattedMain.replace('\n\t"strings"', ""));
    assert(
      !selectedSource(doc).includes('"strings"'),
      "browser shell edit flow updates the selected file contents in place",
      shellSnapshot(doc),
    );

    await setInputValue(
      doc,
      "module-roots",
      moduleRootLine(
        remoteModulePath,
        initialVersion,
        moduleBundleFetchUrl({
          modulePath: remoteModulePath,
          version: initialVersion,
          message: "remote",
        }),
      ),
    );
    await clickAndWaitFor(
      doc,
      "load-modules-button",
      () =>
        control(doc, "module-status").textContent.includes(
          `${remoteModulePath}@${initialVersion}`,
        ) && control(doc, "output").textContent.includes("Loaded 1 remote module bundle(s)."),
      "module load request",
    );
    assert(
      control(doc, "module-status").textContent.includes(
        "Loaded 1 bundle(s) projecting 2 virtual file(s).",
      ),
      "browser shell module load flow reports projected bundle state",
      shellSnapshot(doc),
    );

    await setPlainValue(doc, "entry-path-input", "main.go");
    await clickAndWaitFor(
      doc,
      "test-snippet-button",
      () =>
        control(doc, "status").textContent.includes("Snippet test passed")
          && control(doc, "output").textContent.includes("Entry: main.go")
          && control(doc, "output").textContent.includes("remote:alpha"),
      "snippet test request",
    );
    assert(
      control(doc, "output").textContent.includes("Snippet test passed."),
      "browser shell snippet flow renders structured snippet output",
      shellSnapshot(doc),
    );

    await setPlainValue(doc, "package-target-input", "calc/calc.go");
    await clickAndWaitFor(
      doc,
      "test-package-button",
      () =>
        control(doc, "status").textContent.includes("Package tests passed")
          && control(doc, "output").textContent.includes("Target: calc/calc.go")
          && control(doc, "output").textContent.includes("PASS TestRemoteGreeting"),
      "package test request",
    );
    assert(
      control(doc, "output").textContent.includes("Completed: TestRemoteGreeting"),
      "browser shell package flow renders structured package-test details",
      shellSnapshot(doc),
    );

    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("remote:alpha"),
      "run request",
    );
    assert(
      control(doc, "output").textContent.includes("remote:alpha"),
      "browser shell run flow executes through the loaded remote bundle",
      shellSnapshot(doc),
    );

    await setInputValue(
      doc,
      "module-roots",
      moduleRootLine(
        brokenModulePath,
        "v9.9.9",
        "data:application/json,%7Bbroken-json",
      ),
    );
    await clickAndWaitFor(
      doc,
      "load-modules-button",
      () =>
        control(doc, "status").textContent === "Worker failed"
          && control(doc, "module-status").textContent.includes("Last module load failed:")
          && control(doc, "module-status").textContent.includes(
            "Previously loaded bundles are stale; retry Load Modules or run/test to refresh them.",
          ),
      "failed module refresh request",
    );
    assert(
      control(doc, "module-status").textContent.includes("Last module load failed:"),
      "browser shell module refresh flow preserves the last load failure",
      shellSnapshot(doc),
    );

    await setInputValue(
      doc,
      "module-roots",
      moduleRootLine(
        remoteModulePath,
        refreshedVersion,
        moduleBundleFetchUrl({
          modulePath: remoteModulePath,
          version: refreshedVersion,
          message: "remote-reloaded",
        }),
      ),
    );
    await setWorkspaceFileContents(doc, "data.txt", "beta");
    await clickAndWaitFor(
      doc,
      "load-modules-button",
      () =>
        control(doc, "module-status").textContent.includes(
          `${remoteModulePath}@${refreshedVersion}`,
        ) && !control(doc, "module-status").textContent.includes("Last module load failed:"),
      "module reload recovery request",
    );
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("remote-reloaded:beta"),
      "run request after module reload recovery",
    );
    assert(
      control(doc, "output").textContent.includes("remote-reloaded:beta"),
      "browser shell module reload recovery updates subsequent runs",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

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
    await waitForCancellationComplete(doc, "run cancellation");
    assert(
      control(doc, "status").textContent === "Worker cancelled"
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
    await waitForCancellationComplete(doc, "snippet cancellation");
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
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

export async function testBrowserShellRepresentativeSoak({ assert, frame, log }) {
  log("\n--- browser shell representative soak harness ---");

  const seed = Date.now();
  const remoteModulePath = `example.com/remote/browser-soak-${seed}`;
  const iterations = [
    {
      version: "v3.0.0",
      message: "soak-alpha",
      data: "alpha",
      testName: "TestRemoteGreetingCycleOne",
    },
    {
      version: "v3.0.1",
      message: "soak-beta",
      data: "beta",
      testName: "TestRemoteGreetingCycleTwo",
    },
    {
      version: "v3.0.2",
      message: "soak-gamma",
      data: "gamma",
      testName: "TestRemoteGreetingCycleThree",
    },
    {
      version: "v3.0.3",
      message: "soak-delta",
      data: "delta",
      testName: "TestRemoteGreetingCycleFour",
    },
  ];

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await addWorkspaceFile(doc, "data.txt", iterations[0].data);
    await addWorkspaceFile(doc, "calc/calc.go", calcSource(remoteModulePath));
    await addWorkspaceFile(
      doc,
      "calc/calc_test.go",
      calcTestSource(iterations[0].message, iterations[0].testName),
    );
    await setWorkspaceFileContents(doc, "main.go", appMainSource(remoteModulePath));
    await setPlainValue(doc, "entry-path-input", "main.go");
    await setPlainValue(doc, "package-target-input", "calc/calc.go");

    for (const [index, iteration] of iterations.entries()) {
      await setWorkspaceFileContents(doc, "data.txt", iteration.data);
      await setWorkspaceFileContents(
        doc,
        "calc/calc_test.go",
        calcTestSource(iteration.message, iteration.testName),
      );
      await setInputValue(
        doc,
        "module-roots",
        moduleRootLine(
          remoteModulePath,
          iteration.version,
          moduleBundleFetchUrl({
            modulePath: remoteModulePath,
            version: iteration.version,
            message: iteration.message,
          }),
        ),
      );

      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes(
              `${iteration.message}:${iteration.data}`,
            ),
        `soak run cycle ${index + 1}`,
      );
      assert(
        control(doc, "module-status").textContent.includes(
          `${remoteModulePath}@${iteration.version}`,
        ) && control(doc, "output").textContent.includes(
          `${iteration.message}:${iteration.data}`,
        ),
        `browser shell soak run cycle ${index + 1} completed through autoloaded module state`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "test-snippet-button",
        () =>
          control(doc, "status").textContent.includes("Snippet test passed")
            && control(doc, "output").textContent.includes(
              `${iteration.message}:${iteration.data}`,
            ),
        `soak snippet cycle ${index + 1}`,
      );
      assert(
        control(doc, "output").textContent.includes("Snippet test passed.")
          && control(doc, "output").textContent.includes(
            `${iteration.message}:${iteration.data}`,
          ),
        `browser shell soak snippet cycle ${index + 1} completed`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "test-package-button",
        () =>
          control(doc, "status").textContent.includes("Package tests passed")
            && control(doc, "output").textContent.includes(iteration.testName),
        `soak package cycle ${index + 1}`,
      );
      assert(
        control(doc, "output").textContent.includes(`Completed: ${iteration.testName}`)
          && control(doc, "module-status").textContent.includes(
            `${remoteModulePath}@${iteration.version}`,
          ),
        `browser shell soak package cycle ${index + 1} completed`,
        shellSnapshot(doc),
      );
    }
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

function unformattedMainSource(remoteModulePath) {
  return `package main

import (
"fmt"
"os"
\t"strings"
"${remoteModulePath}/greeter"
)

func joinValue[T any](value T) string {
// normalized comment
return fmt.Sprint(value)
}

func main() {
data, err := os.ReadFile("data.txt")
if err != nil {
panic(err)
}
fmt.Println(joinValue(greeter.Message() + ":" + string(data)))
}
`;
}

function appMainSource(remoteModulePath) {
  return `package main

import (
\t"fmt"
\t"os"
\t"${remoteModulePath}/greeter"
)

func main() {
\tdata, err := os.ReadFile("data.txt")
\tif err != nil {
\t\tpanic(err)
\t}
\tfmt.Println(greeter.Message() + ":" + string(data))
}
`;
}

function sleepingMainSource(remoteModulePath, durationMs, label) {
  const pauseExpr = durationMs > 1000 ? "time.Second" : "time.Millisecond";
  return `package main

import (
\t"fmt"
\t"time"
\t"${remoteModulePath}/greeter"
)

func main() {
\ttime.Sleep(${pauseExpr})
\tfmt.Println("${label}", greeter.Message())
}
`;
}

function calcSource(remoteModulePath) {
  return `package calc

import "${remoteModulePath}/greeter"

func Greeting() string {
\treturn greeter.Message()
}
`;
}

function calcTestSource(expectedGreeting, testName = "TestRemoteGreeting") {
  return `package calc

func ${testName}() {
\tif Greeting() != "${expectedGreeting}" {
\t\tpanic("expected ${expectedGreeting} greeting")
\t}
}
`;
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

export async function loadShellFrame(frame) {
  frame.src = `./index.html?browser-shell-harness=${Date.now()}`;
  await waitForFrameLoad(frame);
  return frame.contentDocument;
}

export async function unloadShellFrame(frame) {
  frame.src = "about:blank";
  await waitForFrameLoad(frame);
}

async function waitForFrameLoad(frame) {
  await new Promise((resolve, reject) => {
    const timer = self.setTimeout(() => {
      frame.removeEventListener("load", handleLoad);
      reject(new Error("timed out waiting for browser shell frame load"));
    }, ACTION_TIMEOUT_MS);

    function handleLoad() {
      self.clearTimeout(timer);
      resolve();
    }

    frame.addEventListener("load", handleLoad, { once: true });
  });
}

export async function waitForShellReady(doc) {
  await waitFor(
    () =>
      control(doc, "status").textContent.startsWith("Worker ready:")
        && !control(doc, "run-button").disabled,
    "browser shell worker ready",
    doc,
  );
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

export async function setWorkspaceFileContents(doc, path, contents) {
  await selectWorkspaceFile(doc, path);
  const view = doc.defaultView._codeEditorView;
  if (!view) {
    throw new Error("CodeMirror editor not available");
  }
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: contents },
  });
  await waitFor(
    () => view.state.doc.toString() === contents,
    `workspace file ${path} edited`,
    doc,
  );
}

async function selectWorkspaceFile(doc, path) {
  const fileList = control(doc, "file-list");
  fileList.value = path;
  dispatch(doc, fileList, "change");
  await waitFor(
    () => control(doc, "editor-file-label").textContent === path,
    `workspace file ${path} selected`,
    doc,
  );
}

export async function setInputValue(doc, id, value) {
  const element = control(doc, id);
  element.value = value;
  dispatch(doc, element, "input");
  await waitFor(() => control(doc, id).value === value, `${id} updated`, doc);
}

export async function setPlainValue(doc, id, value) {
  const element = control(doc, id);
  element.value = value;
  await waitFor(() => control(doc, id).value === value, `${id} updated`, doc);
}

async function clickAndWaitForIdle(doc, buttonId, description) {
  await clickAndWaitFor(doc, buttonId, () => true, description);
}

export async function clickAndWaitFor(doc, buttonId, predicate, description) {
  click(doc, buttonId);
  await waitFor(
    () => !control(doc, "run-button").disabled && predicate(),
    `${description} completed`,
    doc,
  );
}

export function click(doc, id) {
  control(doc, id).click();
}

export function selectedSource(doc) {
  const view = doc.defaultView._codeEditorView;
  if (!view) {
    throw new Error("CodeMirror editor not available");
  }
  return view.state.doc.toString();
}

export function control(doc, id) {
  const element = doc.getElementById(id);
  if (!element) {
    throw new Error(`missing browser shell control #${id}`);
  }
  return element;
}

function dispatch(doc, element, type) {
  element.dispatchEvent(new doc.defaultView.Event(type, { bubbles: true }));
}

async function waitForCancellationReady(doc, description) {
  await waitFor(
    () => !control(doc, "cancel-button").disabled,
    `${description} became available`,
    doc,
  );
}

async function waitForCancellationComplete(doc, description) {
  await waitFor(
    () =>
      !control(doc, "run-button").disabled
        && (control(doc, "output").textContent.includes("Execution cancelled.")
          || control(doc, "output").textContent.includes("Snippet test cancelled.")),
    `${description} completed`,
    doc,
  );
}

export async function waitFor(predicate, description, doc) {
  const start = Date.now();
  while (Date.now() - start < ACTION_TIMEOUT_MS) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => self.setTimeout(resolve, POLL_INTERVAL_MS));
  }

  throw new Error(`${description} timed out\n${shellSnapshot(doc)}`);
}

export function shellSnapshot(doc) {
  return JSON.stringify(
    {
      status: safeText(doc, "status"),
      module_status: safeText(doc, "module-status"),
      output: safeText(doc, "output"),
      selected_file: safeValue(doc, "file-list"),
      entry_path: safeValue(doc, "entry-path-input"),
      package_target: safeValue(doc, "package-target-input"),
    },
    null,
    2,
  );
}

function safeText(doc, id) {
  return doc.getElementById(id)?.textContent ?? "";
}

function safeValue(doc, id) {
  return doc.getElementById(id)?.value ?? "";
}

function delay(durationMs) {
  return new Promise((resolve) => self.setTimeout(resolve, durationMs));
}
