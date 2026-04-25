import { createArchiveDataUrl, createArchiveFile } from "./test-browser-archive-fixtures.js";
import {
  click,
  control,
  loadShellFrame,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellArchiveImport({ assert, frame, log }) {
  log("\n--- browser shell archive import harness ---");

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await importArchiveFile(doc, "workspace.zip", [
      {
        path: "workspace-root/go.mod",
        contents: "module example.com/archive\n\ngo 1.21\n",
      },
      {
        path: "workspace-root/main.go",
        contents: `package main

import "fmt"

func main() {
\tfmt.Println("archive upload")
}
`,
      },
      {
        path: "workspace-root/data.txt",
        contents: "alpha\n",
      },
    ]);
    assert(
      listWorkspacePaths(doc).includes("go.mod")
        && listWorkspacePaths(doc).includes("main.go")
        && listWorkspacePaths(doc).includes("data.txt"),
      "browser shell upload archive import replaces the editable workspace",
      shellSnapshot(doc),
    );
    assert(
      control(doc, "output").textContent.includes("Stripped archive prefix: workspace-root/")
        && control(doc, "entry-path-input").value === "main.go",
      "browser shell upload archive import strips a wrapper root and selects main.go",
      shellSnapshot(doc),
    );

    await importArchiveUrl(doc, [
      {
        path: "../escape.go",
        contents: "package main\n",
      },
    ]);
    assert(
      control(doc, "status").textContent === "Archive import failed"
        && control(doc, "output").textContent.includes("path traversal entry"),
      "browser shell archive import rejects path traversal entries",
      shellSnapshot(doc),
    );

    await importArchiveUrl(doc, [
      {
        path: "assets/logo.png",
        contents: new Uint8Array([0x50, 0x4e, 0x47, 0x21]),
      },
    ]);
    assert(
      control(doc, "status").textContent === "Archive import failed"
        && control(doc, "output").textContent.includes("supported editable workspace file type"),
      "browser shell archive import rejects unsupported file types",
      shellSnapshot(doc),
    );

    await importArchiveUrl(doc, [
      {
        path: "download-root/README.md",
        contents: "# ignored wrapper readme\n",
      },
      {
        path: "download-root/project/go.mod",
        contents: "module example.com/detected\n\ngo 1.21\n",
      },
      {
        path: "download-root/project/cmd/app/main.go",
        contents: `package main

func main() {}
`,
      },
      {
        path: "download-root/project/internal/helper/helper.go",
        contents: `package helper

func Message() string { return "ok" }
`,
      },
    ]);
    assert(
      listWorkspacePaths(doc).includes("cmd/app/main.go")
        && listWorkspacePaths(doc).includes("internal/helper/helper.go")
        && !listWorkspacePaths(doc).some((path) => path.startsWith("download-root/")),
      "browser shell archive import detects a nested module root and strips it",
      shellSnapshot(doc),
    );
    assert(
      control(doc, "output").textContent.includes(
        "Ignored 1 file(s) outside the detected module root.",
      ) && control(doc, "entry-path-input").value === "cmd/app/main.go",
      "browser shell archive import reports ignored wrapper files and updates the entry path",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
  }
}

async function importArchiveFile(doc, archiveName, entries) {
  const fileInput = control(doc, "archive-file-input");
  const file = createArchiveFile(doc.defaultView, archiveName, entries);
  const dataTransfer = new doc.defaultView.DataTransfer();
  dataTransfer.items.add(file);
  Object.defineProperty(fileInput, "files", {
    configurable: true,
    value: dataTransfer.files,
  });
  fileInput.dispatchEvent(new doc.defaultView.Event("change", { bubbles: true }));
  await waitForArchiveImport(doc, `archive file ${archiveName} imported`);
}

async function importArchiveUrl(doc, entries) {
  const archiveUrlInput = control(doc, "archive-url-input");
  archiveUrlInput.value = createArchiveDataUrl(entries);
  click(doc, "archive-url-import-button");
  await waitFor(
    () =>
      !control(doc, "run-button").disabled
        && (control(doc, "status").textContent.startsWith("Imported")
          || control(doc, "status").textContent === "Archive import failed"),
    "archive URL import completed",
    doc,
  );
}

async function waitForArchiveImport(doc, description) {
  await waitFor(
    () => !control(doc, "run-button").disabled && control(doc, "status").textContent.startsWith("Imported"),
    description,
    doc,
  );
}

function listWorkspacePaths(doc) {
  return Array.from(control(doc, "file-list").options).map((option) => option.value);
}
