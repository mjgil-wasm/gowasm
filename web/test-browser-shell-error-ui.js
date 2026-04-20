import { resetModuleCacheDatabase } from "./test-worker-modules.js";
import {
  clickAndWaitFor,
  control,
  loadShellFrame,
  setInputValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellErrorUi({ assert, frame, log }) {
  log("\n--- browser shell error UI harness ---");

  const seed = Date.now();
  const brokenModulePath = `example.com/broken/browser-error-ui-${seed}`;

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await setInputValue(doc, "module-roots", "missing-fields");
    await clickAndWaitFor(
      doc,
      "load-modules-button",
      () => control(doc, "status").textContent === "Module roots config is invalid",
      "invalid module roots request",
    );
    assert(
      summaryPanelText(doc).includes("tooling")
        && summaryPanelText(doc).includes("Fix the module root list and retry Load Modules."),
      "browser shell renders shell-owned configuration failures with a tooling category and next step",
      shellSnapshot(doc),
    );
    assert(
      control(doc, "output").textContent.includes("missing-fields"),
      "browser shell keeps raw shell-side validation detail visible below the structured issue summary",
      shellSnapshot(doc),
    );

    await setInputValue(doc, "module-roots", "");
    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

func main() {
\tundeclaredFunction()
}
`,
    );
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.length > 0,
      "compile diagnostic request",
    );
    assert(
      !control(doc, "diagnostic-summary-panel").hidden
        && summaryPanelText(doc).includes("compile_error")
        && summaryPanelText(doc).includes("main.go")
        && summaryPanelText(doc).includes("Fix the source error and compile again."),
      "browser shell renders compile diagnostics with explicit location and next-step guidance",
      `${shellSnapshot(doc)}\nsummary=${JSON.stringify(summaryPanelText(doc))}`,
    );
    assert(
      control(doc, "output").textContent.length > 0,
      "browser shell keeps raw compile diagnostics visible below the structured issue summary",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

func explode(value int) int {
\treturn 1 / value
}

func main() {
\texplode(0)
}
`,
    );
    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("division by zero"),
      "runtime diagnostic request",
    );
    assert(
      !control(doc, "diagnostic-summary-panel").hidden
        && summaryPanelText(doc).includes("explode (main.go:")
        && summaryPanelText(doc).includes("main (main.go:"),
      "browser shell renders runtime diagnostics with a visible stack summary",
      shellSnapshot(doc),
    );
    assert(
      control(doc, "output").textContent.includes("division by zero")
        && control(doc, "output").textContent.includes("stack trace:"),
      "browser shell keeps raw runtime diagnostics visible below the structured issue summary",
      shellSnapshot(doc),
    );

    await setInputValue(
      doc,
      "module-roots",
      `${brokenModulePath} v9.9.9 data:application/json,%7Bbroken-json`,
    );
    await clickAndWaitFor(
      doc,
      "load-modules-button",
      () =>
        control(doc, "status").textContent === "Worker failed"
          && summaryPanelText(doc).includes("host_error"),
      "worker failure request",
    );
    assert(
      summaryPanelText(doc).includes("Retry the request. If the worker keeps failing, reload the page and rerun."),
      "browser shell renders worker-side failures with an explicit recovery hint",
      shellSnapshot(doc),
    );
    assert(
      control(doc, "output").textContent.length > 0,
      "browser shell keeps raw worker failure detail visible below the structured issue summary",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

function summaryPanelText(doc) {
  return control(doc, "diagnostic-summary").textContent;
}
