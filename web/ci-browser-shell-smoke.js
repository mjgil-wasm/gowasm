import {
  click,
  clickAndWaitFor,
  control,
  loadShellFrame,
  selectedSource,
  setInputValue,
  setWorkspaceFileContents,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";
import { createArchiveDataUrl } from "./test-browser-archive-fixtures.js";
import { resetBrowserShellCacheDatabase } from "./test-browser-shell-cache-fixtures.js";

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const frame = document.querySelector("#shell-frame");
const ciMode = new URLSearchParams(window.location.search).has("ci");

let passed = 0;
let failed = 0;

function log(message) {
  results.textContent += `${message}\n`;
}

function assert(condition, name, detail) {
  if (condition) {
    passed += 1;
    return;
  }
  failed += 1;
  log(`${name}: ${detail}`);
}

function finishCiSummary(text, className) {
  if (!ciMode) {
    return;
  }
  void fetch("/__gowasm_ci_complete", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ elementId: "summary", className, text }),
  });
}

async function runAll() {
  try {
    await resetBrowserShellCacheDatabase();
    let doc = await loadShellFrame(frame);
    try {
      await waitForShellReady(doc);
      assert(
        control(doc, "status").textContent.toLowerCase().includes("ready"),
        "shell ready",
        control(doc, "status").textContent,
      );
      assert(
        control(doc, "compatibility-status").textContent.includes("Support profile: Chromium 146+ (supported)"),
        "shell compatibility profile",
        control(doc, "compatibility-status").textContent,
      );

      control(doc, "packaged-example-select").value = "generics-channels";
      control(doc, "packaged-example-select").dispatchEvent(
        new doc.defaultView.Event("change", { bubbles: true }),
      );
      click(doc, "load-example-button");
      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes("2,4,6"),
        "packaged example run request",
      );
      assert(
        control(doc, "output").textContent.includes("2,4,6"),
        "shell packaged example run output",
        control(doc, "output").textContent,
      );

      await setInputValue(doc, "file-path-input", "pkg/smoke.go");
      click(doc, "add-file-button");
      await waitFor(
        () => Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/smoke.go"),
        "workspace smoke file added",
        doc,
      );
      click(doc, "use-selected-package-button");
      assert(
        control(doc, "package-target-input").value === "pkg/smoke.go",
        "shell quick package target control",
        control(doc, "package-target-input").value,
      );
      await setInputValue(doc, "rename-path-input", "pkg/renamed-smoke.go");
      click(doc, "rename-file-button");
      await waitFor(
        () =>
          Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/renamed-smoke.go")
            && !Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/smoke.go"),
        "workspace smoke rename completed",
        doc,
      );
      click(doc, "remove-file-button");
      await waitFor(
        () => !Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/renamed-smoke.go"),
        "workspace smoke cleanup completed",
        doc,
      );

      await setWorkspaceFileContents(
        doc,
        "main.go",
        `package main

import "fmt"

func main(){
fmt.Println("hello from shell smoke")
}
`,
      );
      await clickAndWaitFor(
        doc,
        "format-button",
        () => control(doc, "status").textContent === "Format complete",
        "format request",
      );
      assert(
        selectedSource(doc).includes('fmt.Println("hello from shell smoke")'),
        "shell format rewrites source",
        selectedSource(doc),
      );

      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes("hello from shell smoke"),
        "run request",
      );
      assert(
        control(doc, "output").textContent.includes("hello from shell smoke"),
        "shell run output",
        control(doc, "output").textContent,
      );

      control(doc, "archive-url-input").value = createArchiveDataUrl([
        {
          path: "browser-shell-smoke/go.mod",
          contents: "module example.com/browser-smoke\n\ngo 1.21\n",
        },
        {
          path: "browser-shell-smoke/main.go",
          contents: `package main

import "fmt"

func main() {
\tfmt.Println("archive shell smoke")
}
`,
        },
      ]);
      await clickAndWaitFor(
        doc,
        "archive-url-import-button",
        () =>
          control(doc, "status").textContent.startsWith("Imported")
            && control(doc, "output").textContent.includes("Stripped archive prefix: browser-shell-smoke/"),
        "archive import request",
      );
      assert(
        control(doc, "entry-path-input").value === "main.go"
          && selectedSource(doc).includes("archive shell smoke"),
        "shell archive import",
        control(doc, "output").textContent,
      );
      await clickAndWaitFor(
        doc,
        "refresh-cache-button",
        () =>
          control(doc, "cache-status").textContent.includes("Imported workspace cache: 1 entry valid"),
        "cache refresh request",
      );
      await setWorkspaceFileContents(
        doc,
        "main.go",
        `package main

func main() {}
`,
      );
      await clickAndWaitFor(
        doc,
        "restore-cached-workspace-button",
        () =>
          control(doc, "status").textContent === "Restored cached workspace"
            && selectedSource(doc).includes("archive shell smoke"),
        "cached workspace restore request",
      );
      assert(
        control(doc, "output").textContent.includes("Restored from browser cache")
          && selectedSource(doc).includes("archive shell smoke"),
        "shell cache restore",
        control(doc, "output").textContent,
      );

      await setWorkspaceFileContents(
        doc,
        "main.go",
        `package main

func main() {
  missing(
}
`,
      );
      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes("expected expression"),
        "compile failure request",
      );
      assert(
        control(doc, "output").textContent.includes("expected expression"),
        "shell compile failure output",
        control(doc, "output").textContent,
      );

      const snapshotText = await exportSnapshotText(doc);
      await setWorkspaceFileContents(
        doc,
        "main.go",
        `package main

func main() {}
`,
      );
      await importSnapshotText(doc, snapshotText);
      await waitForSnapshotImport(doc);
      assert(
        control(doc, "output").textContent.includes("Workspace snapshot v1.")
          && selectedSource(doc).includes("missing("),
        "shell snapshot round-trip",
        control(doc, "output").textContent,
      );

      await unloadShellFrame(frame);
      doc = await loadCustomShellFrame(
        frame,
        [
          `boot_manifest_url=${encodeURIComponent(
            `data:application/json,${encodeURIComponent(
              JSON.stringify({
                version: 1,
                files: [
                  {
                    path: "go.mod",
                    contents: "module example.com/bootsmoke\n\ngo 1.21\n",
                  },
                  {
                    path: "main.go",
                    contents: `package main

import "fmt"

func main() {
\tfmt.Println("boot shell smoke")
}
`,
                  },
                ],
              }),
            )}`,
          )}`,
          "boot_consent=1",
          "boot_action=run",
        ].join("&"),
      );
      await waitForBootRun(doc);
      assert(
        control(doc, "output").textContent.includes("boot shell smoke"),
        "shell boot URL run output",
        control(doc, "output").textContent,
      );
    } finally {
      await unloadShellFrame(frame);
      await resetBrowserShellCacheDatabase();
    }
  } catch (error) {
    failed += 1;
    log(error?.stack || error?.message || String(error));
  }

  if (failed === 0) {
    summary.className = "pass";
    summary.textContent = `all browser shell ci smoke tests passed (${passed} assertions)`;
  } else {
    summary.className = "fail";
    summary.textContent = `${failed} browser shell ci smoke failure(s), ${passed} assertions passed\n${results.textContent.trim()}`;
  }
  finishCiSummary(summary.textContent, summary.className);
}

void runAll();

async function loadCustomShellFrame(frame, query) {
  frame.src = `./index.html?${query}&ci-browser-shell-smoke=${Date.now()}`;
  await new Promise((resolve, reject) => {
    const timer = self.setTimeout(() => {
      frame.removeEventListener("load", onLoad);
      reject(new Error("timed out waiting for custom browser shell frame load"));
    }, 20000);

    function onLoad() {
      self.clearTimeout(timer);
      resolve();
    }

    frame.addEventListener("load", onLoad, { once: true });
  });
  return frame.contentDocument;
}

async function waitForBootRun(doc) {
  const start = Date.now();
  while (Date.now() - start < 20000) {
    if (
      control(doc, "status").textContent === "Worker responded"
      && control(doc, "output").textContent.includes("boot shell smoke")
    ) {
      return;
    }
    await new Promise((resolve) => self.setTimeout(resolve, 25));
  }
  throw new Error(`timed out waiting for boot URL run\n${control(doc, "output").textContent}`);
}

async function exportSnapshotText(doc) {
  const captured = { text: "" };
  const originalCreateObjectUrl = doc.defaultView.URL.createObjectURL;
  const originalRevokeObjectUrl = doc.defaultView.URL.revokeObjectURL;
  const originalClick = doc.defaultView.HTMLAnchorElement.prototype.click;

  doc.defaultView.URL.createObjectURL = (blob) => {
    void blob.text().then((text) => {
      captured.text = text;
    });
    return "blob:gowasm-ci-snapshot";
  };
  doc.defaultView.URL.revokeObjectURL = () => {};
  doc.defaultView.HTMLAnchorElement.prototype.click = () => {};

  try {
    control(doc, "snapshot-export-button").click();
    const start = Date.now();
    while (Date.now() - start < 20000) {
      if (
        control(doc, "status").textContent === "Workspace snapshot exported"
        && captured.text.includes('"version": 1')
      ) {
        return captured.text;
      }
      await new Promise((resolve) => self.setTimeout(resolve, 25));
    }
    throw new Error("timed out waiting for snapshot export");
  } finally {
    doc.defaultView.URL.createObjectURL = originalCreateObjectUrl;
    doc.defaultView.URL.revokeObjectURL = originalRevokeObjectUrl;
    doc.defaultView.HTMLAnchorElement.prototype.click = originalClick;
  }
}

async function importSnapshotText(doc, text) {
  const fileInput = control(doc, "snapshot-file-input");
  const file = new doc.defaultView.File([text], "gowasm-workspace.snapshot.json", {
    type: "application/json",
  });
  const dataTransfer = new doc.defaultView.DataTransfer();
  dataTransfer.items.add(file);
  Object.defineProperty(fileInput, "files", {
    configurable: true,
    value: dataTransfer.files,
  });
  fileInput.dispatchEvent(new doc.defaultView.Event("change", { bubbles: true }));
}

async function waitForSnapshotImport(doc) {
  const start = Date.now();
  while (Date.now() - start < 20000) {
    if (control(doc, "status").textContent === "Workspace snapshot imported") {
      return;
    }
    await new Promise((resolve) => self.setTimeout(resolve, 25));
  }
  throw new Error("timed out waiting for snapshot import");
}
