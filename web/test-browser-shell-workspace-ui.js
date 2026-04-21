import { createArchiveDataUrl } from "./test-browser-archive-fixtures.js";
import {
  click,
  control,
  loadShellFrame,
  setInputValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellWorkspaceUi({ assert, frame, log }) {
  log("\n--- browser shell workspace UI harness ---");

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    assert(
      control(doc, "workspace-dirty-status").textContent.includes("matches the last imported or bootstrapped baseline"),
      "browser shell workspace UI starts from a clean baseline",
      shellSnapshot(doc),
    );

    await setInputValue(doc, "file-path-input", "pkg/helper.go");
    click(doc, "add-file-button");
    await waitFor(
      () => listWorkspacePaths(doc).includes("pkg/helper.go"),
      "workspace helper.go added",
      doc,
    );
    await setWorkspaceFileContents(
      doc,
      "pkg/helper.go",
      `package pkg

func Helper() string { return "ok" }
`,
    );
    assert(
      findWorkspaceTreeButton(doc, "pkg/helper.go") !== null
        && control(doc, "workspace-dirty-status").textContent.includes("1 current file(s) differ from baseline"),
      "browser shell workspace UI renders nested tree entries and dirty-state messaging for added files",
      shellSnapshot(doc),
    );

    await selectTreePath(doc, "pkg/helper.go");
    click(doc, "use-selected-entry-button");
    click(doc, "use-selected-package-button");
    assert(
      control(doc, "entry-path-input").value === "pkg/helper.go"
        && control(doc, "package-target-input").value === "pkg/helper.go",
      "browser shell workspace UI quick target controls adopt the selected Go file",
      shellSnapshot(doc),
    );

    await setInputValue(doc, "rename-path-input", "pkg/renamed.go");
    click(doc, "rename-file-button");
    await waitFor(
      () =>
        listWorkspacePaths(doc).includes("pkg/renamed.go")
          && !listWorkspacePaths(doc).includes("pkg/helper.go")
          && control(doc, "editor-file-label").textContent === "pkg/renamed.go",
      "workspace file renamed",
      doc,
    );
    assert(
      control(doc, "entry-path-input").value === "pkg/renamed.go"
        && control(doc, "package-target-input").value === "pkg/renamed.go",
      "browser shell workspace UI keeps entry and package targets aligned with rename operations",
      shellSnapshot(doc),
    );

    await setInputValue(doc, "file-path-input", "assets/logo.bin");
    click(doc, "add-file-button");
    await waitFor(
      () => listWorkspacePaths(doc).includes("assets/logo.bin"),
      "workspace binary-ish file added",
      doc,
    );
    await selectTreePath(doc, "assets/logo.bin");
    assert(
      control(doc, "workspace-selection-note").textContent.includes("supported editable file-type slice")
        && control(doc, "use-selected-entry-button").disabled
        && control(doc, "use-selected-package-button").disabled,
      "browser shell workspace UI warns on unsupported editable file paths and disables Go-only quick target controls",
      shellSnapshot(doc),
    );

    control(doc, "archive-url-input").value = createArchiveDataUrl([
      {
        path: "workspace-root/go.mod",
        contents: "module example.com/workspace-ui\n\ngo 1.21\n",
      },
      {
        path: "workspace-root/cmd/app/main.go",
        contents: `package main

func main() {}
`,
      },
    ]);
    click(doc, "archive-url-import-button");
    await waitFor(
      () =>
        control(doc, "status").textContent.startsWith("Imported")
          && control(doc, "workspace-dirty-status").textContent.includes("matches the last imported or bootstrapped baseline"),
      "archive import baseline reset",
      doc,
    );
    assert(
      listWorkspacePaths(doc).includes("cmd/app/main.go")
        && !listWorkspacePaths(doc).includes("pkg/renamed.go")
        && control(doc, "workspace-dirty-status").textContent.includes("matches the last imported or bootstrapped baseline"),
      "browser shell workspace UI resets dirty-state tracking when a full archive import replaces the editable workspace",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
  }
}

function listWorkspacePaths(doc) {
  return Array.from(control(doc, "file-list").options).map((option) => option.value);
}

function findWorkspaceTreeButton(doc, path) {
  return doc.querySelector(`#workspace-tree [data-path="${doc.defaultView.CSS.escape(path)}"]`);
}

async function selectTreePath(doc, path) {
  const button = findWorkspaceTreeButton(doc, path);
  if (!button) {
    throw new Error(`missing workspace tree button for ${path}\n${shellSnapshot(doc)}`);
  }
  button.click();
  await waitFor(
    () => control(doc, "editor-file-label").textContent === path,
    `workspace tree selected ${path}`,
    doc,
  );
}
