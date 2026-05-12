import {
  click,
  control,
  loadShellFrame,
  selectedSource,
  setInputValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellEditor({ assert, frame, log }) {
  log("\n--- browser shell editor harness ---");

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);

    // Sidebar toggle collapse/expand
    assert(
      !doc.getElementById("workspace-shell").classList.contains("collapsed"),
      "editor sidebar starts expanded",
      shellSnapshot(doc),
    );
    click(doc, "sidebar-toggle");
    await waitFor(
      () => doc.getElementById("workspace-shell").classList.contains("collapsed"),
      "sidebar collapsed",
      doc,
    );
    assert(
      doc.getElementById("workspace-shell").classList.contains("collapsed"),
      "editor sidebar collapses when toggle is clicked",
      shellSnapshot(doc),
    );
    click(doc, "sidebar-toggle");
    await waitFor(
      () => !doc.getElementById("workspace-shell").classList.contains("collapsed"),
      "sidebar expanded",
      doc,
    );
    assert(
      !doc.getElementById("workspace-shell").classList.contains("collapsed"),
      "editor sidebar expands when toggle is clicked again",
      shellSnapshot(doc),
    );

    // Tab creation on file selection
    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main\n\nimport "fmt"\n\nfunc main() {\n\tfmt.Println("hello")\n}\n`,
    );
    await waitFor(
      () => doc.querySelectorAll("#editor-tabs .editor-tab").length === 1,
      "first tab opened",
      doc,
    );
    assert(
      doc.querySelector("#editor-tabs .editor-tab.active")?.textContent.includes("main.go"),
      "editor opens a tab for the selected file",
      shellSnapshot(doc),
    );

    // Add a second file and open it
    await setInputValue(doc, "file-path-input", "helper.go");
    click(doc, "add-file-button");
    await waitFor(
      () => doc.querySelectorAll("#editor-tabs .editor-tab").length === 2,
      "second tab opened",
      doc,
    );
    assert(
      doc.querySelectorAll("#editor-tabs .editor-tab").length === 2,
      "editor opens a second tab when a new file is added and selected",
      shellSnapshot(doc),
    );

    // Switch tabs by clicking
    const tabs = Array.from(doc.querySelectorAll("#editor-tabs .editor-tab"));
    const firstTab = tabs.find((t) => t.textContent.includes("main.go"));
    assert(firstTab != null, "main.go tab exists", shellSnapshot(doc));
    firstTab.click();
    await waitFor(
      () => selectedSource(doc).includes("hello"),
      "tab switch updated editor",
      doc,
    );
    assert(
      selectedSource(doc).includes("hello"),
      "editor switches content when a tab is clicked",
      shellSnapshot(doc),
    );

    // Dirty indicator
    const view = doc.defaultView._codeEditorView;
    view.dispatch({
      changes: { from: view.state.doc.length, insert: " // edit" },
    });
    await waitFor(
      () => doc.querySelector("#editor-tabs .editor-tab.active .tab-dirty") !== null,
      "dirty indicator appears",
      doc,
    );
    assert(
      doc.querySelector("#editor-tabs .editor-tab.active .tab-dirty") !== null,
      "editor tab shows a dirty indicator after the file is edited",
      shellSnapshot(doc),
    );

    // Close tab
    const closeBtn = doc.querySelector("#editor-tabs .editor-tab.active .tab-close");
    assert(closeBtn != null, "close button exists on active tab", shellSnapshot(doc));
    closeBtn.click();
    await waitFor(
      () => doc.querySelectorAll("#editor-tabs .editor-tab").length === 1,
      "tab closed",
      doc,
    );
    assert(
      doc.querySelectorAll("#editor-tabs .editor-tab").length === 1,
      "editor tab closes when its close button is clicked",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
  }
}
