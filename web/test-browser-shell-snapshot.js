import {
  click,
  control,
  loadShellFrame,
  setInputValue,
  setPlainValue,
  setWorkspaceFileContents,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellSnapshots({ assert, frame, log }) {
  log("\n--- browser shell snapshot harness ---");

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

import "fmt"

func main() {
\tfmt.Println("snapshot-main")
}
`,
    );
    await setInputValue(doc, "file-path-input", "pkg/helper.go");
    click(doc, "add-file-button");
    await waitFor(
      () => Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/helper.go"),
      "snapshot helper file added",
      doc,
    );
    await setWorkspaceFileContents(
      doc,
      "pkg/helper.go",
      `package helper

func Message() string {
\treturn "snapshot-helper"
}
`,
    );
    await setPlainValue(doc, "entry-path-input", "main.go");
    await setPlainValue(doc, "package-target-input", "pkg/helper.go");
    await setInputValue(
      doc,
      "module-roots",
      "example.com/remote/snapshot v1.0.0 https://example.invalid/module.json",
    );

    const snapshotText = await exportSnapshotText(doc);
    assert(
      snapshotText.includes('"version": 1')
        && snapshotText.includes('"path": "pkg/helper.go"')
        && snapshotText.includes('"module_path": "example.com/remote/snapshot"'),
      "browser shell snapshot export serializes workspace files and module-root references",
      snapshotText,
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

func main() {}
`,
    );
    await setInputValue(doc, "file-path-input", "temp.go");
    click(doc, "add-file-button");
    await waitFor(
      () => Array.from(control(doc, "file-list").options).some((option) => option.value === "temp.go"),
      "temporary file added",
      doc,
    );
    await setInputValue(doc, "module-roots", "");
    await importSnapshotText(doc, snapshotText);
    await waitFor(
      () =>
        control(doc, "status").textContent === "Workspace snapshot imported"
          && control(doc, "entry-path-input").value === "main.go"
          && control(doc, "package-target-input").value === "pkg/helper.go"
          && control(doc, "module-roots").value.includes("example.com/remote/snapshot"),
      "snapshot round-trip restore completed",
      doc,
    );
    assert(
      Array.from(control(doc, "file-list").options).some((option) => option.value === "pkg/helper.go")
        && !Array.from(control(doc, "file-list").options).some((option) => option.value === "temp.go")
        && control(doc, "output").textContent.includes("Workspace snapshot v1."),
      "browser shell snapshot import restores workspace files and selected targets",
      shellSnapshot(doc),
    );

    await importSnapshotText(
      doc,
      JSON.stringify({
        version: 1,
        workspace: {
          files: [{ path: "go.mod", contents: "module example.com/bad\n" }],
          selected_file_path: "missing.go",
          entry_path: "missing.go",
          package_target: "",
        },
        module_roots: [],
        loaded_module_refs: [],
        settings: {},
        examples: {},
      }),
    );
    await waitFor(
      () => control(doc, "status").textContent === "Snapshot import failed",
      "invalid snapshot failure surfaced",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("missing from the workspace files"),
      "browser shell snapshot import rejects inconsistent restore payloads",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
  }
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
    return "blob:gowasm-test-snapshot";
  };
  doc.defaultView.URL.revokeObjectURL = () => {};
  doc.defaultView.HTMLAnchorElement.prototype.click = () => {};

  try {
    click(doc, "snapshot-export-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Workspace snapshot exported"
          && captured.text.includes('"version": 1'),
      "snapshot export completed",
      doc,
    );
    return captured.text;
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

function shellSnapshot(doc) {
  return JSON.stringify(
    {
      entry_path: control(doc, "entry-path-input").value,
      module_roots: control(doc, "module-roots").value,
      output: control(doc, "output").textContent,
      package_target: control(doc, "package-target-input").value,
      status: control(doc, "status").textContent,
    },
    null,
    2,
  );
}
