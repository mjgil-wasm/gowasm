import { resetModuleCacheDatabase } from "./test-worker-modules.js";
import {
  click,
  clickAndWaitFor,
  control,
  loadShellFrame,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellDiagnosticNavigation({ assert, frame, log }) {
  log("\n--- browser shell diagnostic navigation harness ---");

  const seed = Date.now();
  const remoteModulePath = `example.com/remote/browser-trace-${seed}`;
  const remoteVersion = "v1.0.0";

  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

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
        control(doc, "status").textContent === "Diagnostics received"
          && control(doc, "output").textContent.includes("undefined: undeclaredFunction"),
      "compile diagnostic request",
    );

    findSourceLinkButton(doc, "primary: main.go:4:2").click();
    await waitFor(
      () =>
        control(doc, "editor-file-label").textContent === "main.go"
          && selectedText(doc).includes("undeclaredFunction"),
      "compile diagnostic jump",
      doc,
    );
    assert(
      selectedText(doc).includes("undeclaredFunction"),
      "browser shell jumps compile diagnostics back into the editable source",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

import "${remoteModulePath}/panicmod"

func main() {
\tpanicmod.Run()
}
`,
    );
    control(doc, "module-roots").value = moduleRootLine(
      remoteModulePath,
      remoteVersion,
      moduleBundleFetchUrl(remoteModulePath, remoteVersion),
    );
    control(doc, "module-roots").dispatchEvent(
      new doc.defaultView.Event("input", { bubbles: true }),
    );

    await clickAndWaitFor(
      doc,
      "run-button",
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("division by zero"),
      "runtime diagnostic request",
    );

    findSourceLinkButton(doc, "frame 1: explode").click();
    await waitFor(
      () =>
        control(doc, "editor-file-label").textContent.includes("__module_cache__/")
          && control(doc, "source").readOnly
          && selectedText(doc).includes("1 / value"),
      "runtime frame jump into projected source",
      doc,
    );
    assert(
      control(doc, "editor-file-label").textContent.includes("__module_cache__/")
        && control(doc, "source").readOnly
        && selectedText(doc).includes("1 / value"),
      "browser shell jumps runtime stack frames into projected read-only module sources",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await new Promise((resolve) => self.setTimeout(resolve, 50));
    await resetModuleCacheDatabase();
  }
}

function findSourceLinkButton(doc, text) {
  const button = Array.from(doc.querySelectorAll("#source-links button")).find((candidate) =>
    candidate.textContent.includes(text),
  );
  if (!button) {
    throw new Error(`missing source-link button containing ${JSON.stringify(text)}`);
  }
  return button;
}

function selectedText(doc) {
  const textarea = control(doc, "source");
  return textarea.value.slice(textarea.selectionStart, textarea.selectionEnd);
}

function moduleRootLine(modulePath, version, fetchUrl) {
  return `${modulePath} ${version} ${fetchUrl}`;
}

function moduleBundleFetchUrl(modulePath, version) {
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
          path: "panicmod/panicmod.go",
          contents: `package panicmod

func Run() {
\texplode(0)
}

func explode(value int) int {
\treturn 1 / value
}
`,
        },
      ],
    }),
  )}`;
}
