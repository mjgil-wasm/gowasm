import {
  pickDirectory,
  restoreDirectory,
  listDir,
  readFile,
  writeFile,
  deleteEntry,
  createDirectory,
  renameEntry,
  createMemoryFS,
  isFileSystemAccessSupported,
} from "./fs.js";
import { TabbedEditor } from "./editor.js";
import { exportZip, importZip } from "./zip.js";

/* ─── DOM refs ─── */
const fileTree = document.getElementById("file-tree");
const newFileBtn = document.getElementById("new-file-btn");
const newFolderBtn = document.getElementById("new-folder-btn");
const saveBtn = document.getElementById("save-btn");
const editorStatus = document.getElementById("editor-status");
const terminalHistory = document.getElementById("terminal-history");
const terminalInput = document.getElementById("terminal-input");
const clearOutputBtn = document.getElementById("clear-output-btn");
const abortBtn = document.getElementById("abort-btn");
const outputStatus = document.getElementById("output-status");
const runBtn = document.getElementById("run-btn");
const buildBtn = document.getElementById("build-btn");
const testBtn = document.getElementById("test-btn");
const formatBtn = document.getElementById("format-btn");
const vetBtn = document.getElementById("vet-btn");
const statusEl = document.getElementById("status");
const contextMenu = document.getElementById("context-menu");
const modal = document.getElementById("modal");
const modalMessage = document.getElementById("modal-message");
const modalInput = document.getElementById("modal-input");
const modalConfirm = document.getElementById("modal-confirm");
const modalCancel = document.getElementById("modal-cancel");
const moduleNameEl = document.getElementById("module-name");
const importZipBtn = document.getElementById("import-zip-btn");
const exportZipBtn = document.getElementById("export-zip-btn");
const zipImportInput = document.getElementById("zip-import-input");
const initModuleBtn = document.getElementById("init-module-btn");


/* ─── State ─── */
let dirHandle = null;
let memoryFS = createMemoryFS();
let useFSAPI = isFileSystemAccessSupported();
let treeData = new Map(); // path -> expanded
let openFiles = new Map(); // path -> content cache
let activePath = "";
let editor = null;
let worker = null;
let workerReady = false;
let activeRequestKind = null;
let moduleName = "";
let contextTargetPath = "";
let stdinQueue = [];
let awaitingInput = false;
let autosaveTimer = null;
const AUTOSAVE_DELAY = 500;

/* ─── Helpers ─── */
function setStatus(text, type = "") {
  statusEl.textContent = text;
  statusEl.className = type;
}

function logOutput(text, className = "") {
  const line = document.createElement("div");
  if (className) line.className = className;
  // Parse file:line:col patterns into clickable links
  const pattern = /([^\s:]+\.go):(\d+)(?::(\d+))?/g;
  let lastIndex = 0;
  let match;
  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      line.appendChild(document.createTextNode(text.slice(lastIndex, match.index)));
    }
    const link = document.createElement("span");
    link.className = "error-link";
    link.textContent = match[0];
    const filePath = match[1];
    const lineNum = parseInt(match[2], 10);
    const colNum = match[3] ? parseInt(match[3], 10) : 1;
    link.addEventListener("click", () => {
      openFile(filePath).then(() => {
        if (editor && editor.editorView) {
          const doc = editor.editorView.state.doc;
          const pos = doc.line(Math.min(lineNum, doc.lines)).from + Math.max(0, colNum - 1);
          editor.editorView.dispatch({ selection: { anchor: pos, head: pos } });
          editor.editorView.focus();
        }
      });
    });
    line.appendChild(link);
    lastIndex = pattern.lastIndex;
  }
  if (lastIndex < text.length) {
    line.appendChild(document.createTextNode(text.slice(lastIndex)));
  }
  if (lastIndex === 0) {
    line.textContent = text;
  }
  terminalHistory.appendChild(line);
  terminalHistory.scrollTop = terminalHistory.scrollHeight;
}

function clearTerminal() {
  terminalHistory.innerHTML = "";
}

function setOutputStatus(text) {
  outputStatus.textContent = text;
}

function setEditorStatus(text) {
  editorStatus.textContent = text;
}

/* ─── Engine worker ─── */
function initWorker() {
  if (worker) return;
  worker = new Worker("../engine-worker.js?v=2", { type: "module" });
  worker.addEventListener("message", ({ data }) => {
    handleWorkerMessage(data);
  });
  worker.addEventListener("error", (e) => {
    setStatus("Worker error: " + e.message, "error");
    activeRequestKind = null;
    syncControls();
  });
  worker.postMessage({ kind: "boot" });
}

function handleWorkerMessage(data) {
  if (data?.kind === "ready") {
    workerReady = true;
    setStatus("Engine ready");
    syncControls();
    return;
  }
  if (data?.kind === "fatal") {
    setStatus("Engine fatal: " + data.message, "error");
    logOutput("Fatal: " + data.message, "error");
    activeRequestKind = null;
    syncControls();
    return;
  }
  if (data?.kind === "cancelled") {
    setStatus("Cancelled");
    activeRequestKind = null;
    syncControls();
    return;
  }

  switch (data?.kind) {
    case "run_result": {
      activeRequestKind = null;
      const stdout = data.stdout || "";
      if (stdout) logOutput(stdout);
      const hasErrors = data.diagnostics?.some((d) => d.severity === "error");
      setOutputStatus(hasErrors ? "Run had errors" : "Finished");
      setStatus(hasErrors ? "Run had errors" : "Run complete", hasErrors ? "error" : "ok");
      if (data.diagnostics?.length) {
        for (const d of data.diagnostics) {
          logOutput(formatDiagnostic(d), d.severity);
        }
      }
      syncControls();
      break;
    }
    case "test_result": {
      activeRequestKind = null;
      const stdout = data.stdout || "";
      const passed = data.passed ?? false;
      if (stdout) logOutput(stdout);
      logOutput(`\n[tests ${passed ? "passed" : "failed"}]`, passed ? "ok" : "error");
      setStatus(passed ? "Tests passed" : "Tests failed", passed ? "ok" : "error");
      syncControls();
      break;
    }
    case "format_result": {
      activeRequestKind = null;
      // Apply formatted files back to FS and editor
      if (data.files?.length) {
        for (const f of data.files) {
          openFiles.set(f.path, f.contents);
          if (editor && editor.activePath === f.path) {
            editor.openTab(f.path, f.contents);
            editor.markClean(f.path);
          }
          if (useFSAPI && dirHandle) {
            writeFile(dirHandle, f.path, f.contents).catch(() => {});
          } else {
            memoryFS.writeFile(f.path, f.contents);
          }
        }
      }
      if (data.diagnostics?.length) {
        for (const d of data.diagnostics) {
          logOutput(formatDiagnostic(d), d.severity);
        }
      } else {
        logOutput("Formatted successfully", "ok");
      }
      setStatus("Formatted", "ok");
      syncControls();
      break;
    }
    case "lint_result": {
      activeRequestKind = null;
      if (data.diagnostics?.length) {
        for (const d of data.diagnostics) {
          logOutput(formatDiagnostic(d), d.severity);
        }
      } else {
        logOutput("No issues found", "ok");
      }
      setStatus("Vet complete", "ok");
      syncControls();
      break;
    }
    case "diagnostics": {
      activeRequestKind = null;
      if (data.diagnostics?.length) {
        for (const d of data.diagnostics) {
          logOutput(formatDiagnostic(d), d.severity);
        }
      } else {
        logOutput("Build succeeded", "ok");
      }
      setStatus(data.diagnostics?.length ? "Build failed" : "Build succeeded", data.diagnostics?.length ? "error" : "ok");
      syncControls();
      break;
    }
    default:
      // Engine may stream capability/module requests internally; ignore here
      break;
    }
}

function formatDiagnostic(d) {
  let s = "";
  if (d.path) s += d.path;
  if (d.line != null) s += `:${d.line}`;
  if (d.column != null) s += `:${d.column}`;
  if (s) s += ": ";
  s += d.message;
  return s;
}

function sendWorkerRequest(kind, request) {
  if (!workerReady || activeRequestKind) return;
  activeRequestKind = kind;
  syncControls();
  worker.postMessage(request);
}

function syncControls() {
  const busy = activeRequestKind !== null;
  runBtn.disabled = busy || !workerReady;
  buildBtn.disabled = busy || !workerReady;
  testBtn.disabled = busy || !workerReady;
  formatBtn.disabled = busy || !workerReady;
  vetBtn.disabled = busy || !workerReady;
  abortBtn.disabled = !busy;
}

/* ─── File system abstraction ─── */
async function fsListDir(path = "") {
  if (useFSAPI && dirHandle) {
    let h = dirHandle;
    if (path) {
      for (const part of path.split("/").filter(Boolean)) {
        h = await h.getDirectoryHandle(part);
      }
    }
    return await listDir(h, path);
  }
  return memoryFS.listDir(path);
}

async function fsReadFile(path) {
  if (openFiles.has(path)) return openFiles.get(path);
  if (useFSAPI && dirHandle) return await readFile(dirHandle, path);
  return memoryFS.readFile(path);
}

async function fsWriteFile(path, text) {
  if (useFSAPI && dirHandle) {
    await writeFile(dirHandle, path, text);
  } else {
    memoryFS.writeFile(path, text);
  }
  openFiles.set(path, text);
}

async function fsDeleteEntry(path) {
  if (useFSAPI && dirHandle) {
    await deleteEntry(dirHandle, path);
  } else {
    memoryFS.deleteEntry(path);
  }
  openFiles.delete(path);
}

async function fsCreateDirectory(path) {
  if (useFSAPI && dirHandle) {
    await createDirectory(dirHandle, path);
  } else {
    memoryFS.createDirectory(path);
  }
}

async function fsRenameEntry(oldPath, newPath) {
  if (useFSAPI && dirHandle) {
    await renameEntry(dirHandle, oldPath, newPath);
  } else {
    memoryFS.renameEntry(oldPath, newPath);
  }
  const content = openFiles.get(oldPath);
  if (content !== undefined) {
    openFiles.delete(oldPath);
    openFiles.set(newPath, content);
  }
}

function getAllWorkspaceFiles() {
  if (useFSAPI && dirHandle) {
    // For FS API, we need to gather all files recursively.
    // For the engine, we'll use openFiles as a cache and walk the tree.
    // Since walking the whole FS is async and complex, we'll collect from openFiles
    // plus a basic walk of known paths.
    const files = [];
    for (const [path, contents] of openFiles.entries()) {
      files.push({ path, contents });
    }
    return files;
  }
  return memoryFS.exportFiles();
}

/* ─── Tree rendering ─── */
async function renderTree() {
  fileTree.innerHTML = "";
  const root = await fsListDir("");
  for (const entry of root) {
    await renderNode(entry, "", fileTree);
  }
  updateModuleName();
}

async function renderNode(entry, parentPath, container) {
  const path = parentPath ? `${parentPath}/${entry.name}` : entry.name;
  const el = document.createElement("div");
  el.className = "tree-node" + (path === activePath ? " selected" : "");
  el.dataset.path = path;
  el.dataset.kind = entry.kind;

  const toggle = document.createElement("span");
  toggle.className = "toggle";
  toggle.textContent = entry.kind === "directory" ? (treeData.get(path) ? "▼" : "▶") : " ";

  const icon = document.createElement("span");
  icon.className = "icon";
  icon.textContent = entry.kind === "directory" ? "📁" : "📄";

  const label = document.createElement("span");
  label.className = "label";
  label.textContent = entry.name;

  el.appendChild(toggle);
  el.appendChild(icon);
  el.appendChild(label);

  if (entry.kind === "directory") {
    const childrenContainer = document.createElement("div");
    childrenContainer.className = "tree-children";
    childrenContainer.style.display = treeData.get(path) ? "block" : "none";
    if (treeData.get(path)) {
      try {
        const children = await fsListDir(path);
        for (const child of children) {
          await renderNode(child, path, childrenContainer);
        }
      } catch (e) {
        // ignore
      }
    }

    el.addEventListener("click", async (e) => {
      e.stopPropagation();
      if (treeData.get(path)) {
        treeData.delete(path);
      } else {
        treeData.set(path, true);
      }
      await renderTree();
    });

    container.appendChild(el);
    container.appendChild(childrenContainer);
  } else {
    el.addEventListener("click", (e) => {
      e.stopPropagation();
      openFile(path);
    });
    el.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      showContextMenu(e, path, "file");
    });
    container.appendChild(el);
  }
}

/* ─── Editor integration ─── */
async function openFile(path) {
  activePath = path;
  const content = await fsReadFile(path);
  openFiles.set(path, content);
  if (!editor) {
    editor = new TabbedEditor(document.getElementById("editor-panel"), {
      onChange: (p, text) => {
        openFiles.set(p, text);
        setEditorStatus("Unsaved");
        if (autosaveTimer) clearTimeout(autosaveTimer);
        autosaveTimer = setTimeout(() => {
          saveFile(p);
          autosaveTimer = null;
        }, AUTOSAVE_DELAY);
      },
      onSave: (p) => saveFile(p),
    });
  }
  editor.openTab(path, content);
  setEditorStatus("");
  await renderTree();
}

async function saveFile(path) {
  if (!editor || !path) return;
  const text = editor.getContent();
  await fsWriteFile(path, text);
  editor.markClean(path);
  setEditorStatus("Saved");
  setStatus(`Saved ${path}`, "ok");
}

/* ─── Toolbar actions ─── */
async function collectExecutionFiles() {
  // Gather all files for the engine. For in-memory FS this is easy.
  // For File System Access API, we need to walk the tree.
  if (useFSAPI && dirHandle) {
    const files = [];
    async function walk(handle, prefix) {
      for await (const [name, child] of handle.entries()) {
        const path = prefix ? `${prefix}/${name}` : name;
        if (child.kind === "directory") {
          await walk(child, path);
        } else {
          try {
            const f = await child.getFile();
            const text = await f.text();
            files.push({ path, contents: text });
          } catch (e) {
            // skip unreadable
          }
        }
      }
    }
    await walk(dirHandle, "");
    return files;
  }
  return memoryFS.exportFiles();
}

function findEntryPath(files) {
  // Prefer main.go, then any .go file with package main
  const mainGo = files.find((f) => f.path === "main.go");
  if (mainGo) return "main.go";
  for (const f of files) {
    if (f.path.endsWith(".go") && f.contents.includes("package main")) {
      return f.path;
    }
  }
  const anyGo = files.find((f) => f.path.endsWith(".go"));
  if (anyGo) return anyGo.path;
  return "";
}

runBtn.addEventListener("click", async () => {
  const files = await collectExecutionFiles();
  const entryPath = findEntryPath(files);
  if (!entryPath) {
    setStatus("No Go entry file found", "error");
    return;
  }
  clearTerminal();
  logOutput(`$ go run ${entryPath}`);
  setStatus("Running…");
  sendWorkerRequest("run", { kind: "run", entry_path: entryPath, files });
});

buildBtn.addEventListener("click", async () => {
  const files = await collectExecutionFiles();
  const entryPath = findEntryPath(files);
  if (!entryPath) {
    setStatus("No Go entry file found", "error");
    return;
  }
  clearTerminal();
  logOutput(`$ go build ${entryPath}`);
  setStatus("Building…");
  sendWorkerRequest("compile", { kind: "compile", entry_path: entryPath, files });
});

testBtn.addEventListener("click", async () => {
  const files = await collectExecutionFiles();
  const entryPath = findEntryPath(files);
  if (!entryPath) {
    setStatus("No Go entry file found", "error");
    return;
  }
  clearTerminal();
  logOutput(`$ go test ./... -v`);
  setStatus("Testing…");
  sendWorkerRequest("test_package", { kind: "test_package", target_path: entryPath, files });
});

formatBtn.addEventListener("click", async () => {
  const files = await collectExecutionFiles();
  clearTerminal();
  setStatus("Formatting…");
  sendWorkerRequest("format", { kind: "format", files });
});

vetBtn.addEventListener("click", async () => {
  const files = await collectExecutionFiles();
  clearTerminal();
  setStatus("Vetting…");
  sendWorkerRequest("lint", { kind: "lint", files });
});

abortBtn.addEventListener("click", () => {
  if (worker && activeRequestKind) {
    worker.postMessage({ kind: "cancel" });
  }
});

clearOutputBtn.addEventListener("click", () => {
  clearTerminal();
  clearTerminal();
});



/* ─── Context menu ─── */
function showContextMenu(e, path, kind) {
  contextTargetPath = path;
  contextMenu.style.left = e.pageX + "px";
  contextMenu.style.top = e.pageY + "px";
  contextMenu.hidden = false;
}

document.addEventListener("click", () => {
  contextMenu.hidden = true;
});

contextMenu.addEventListener("click", async (e) => {
  const action = e.target.dataset.action;
  if (!action || !contextTargetPath) return;
  switch (action) {
    case "new-file": {
      const dir = contextTargetPath;
      const name = await promptModal("New file name:");
      if (name) {
        const path = dir ? `${dir}/${name}` : name;
        await fsWriteFile(path, "");
        await renderTree();
        openFile(path);
      }
      break;
    }
    case "new-folder": {
      const dir = contextTargetPath;
      const name = await promptModal("New folder name:");
      if (name) {
        const path = dir ? `${dir}/${name}` : name;
        await fsCreateDirectory(path);
        await renderTree();
      }
      break;
    }
    case "rename": {
      const name = await promptModal("Rename to:", contextTargetPath.split("/").pop());
      if (name) {
        const parent = contextTargetPath.includes("/")
          ? contextTargetPath.slice(0, contextTargetPath.lastIndexOf("/"))
          : "";
        const newPath = parent ? `${parent}/${name}` : name;
        await fsRenameEntry(contextTargetPath, newPath);
        await renderTree();
      }
      break;
    }
    case "delete": {
      if (confirm(`Delete ${contextTargetPath}?`)) {
        await fsDeleteEntry(contextTargetPath);
        if (activePath === contextTargetPath) {
          activePath = "";
          if (editor) editor.closeTab(contextTargetPath);
        }
        await renderTree();
      }
      break;
    }
  }
});

/* ─── Modal ─── */
function promptModal(message, defaultValue = "") {
  return new Promise((resolve) => {
    modalMessage.textContent = message;
    modalInput.value = defaultValue;
    modal.hidden = false;
    modalInput.focus();

    function onConfirm() {
      cleanup();
      resolve(modalInput.value.trim());
    }
    function onCancel() {
      cleanup();
      resolve("");
    }
    function cleanup() {
      modal.hidden = true;
      modalConfirm.removeEventListener("click", onConfirm);
      modalCancel.removeEventListener("click", onCancel);
      modalInput.removeEventListener("keydown", onKey);
    }
    function onKey(e) {
      if (e.key === "Enter") onConfirm();
      if (e.key === "Escape") onCancel();
    }
    modalConfirm.addEventListener("click", onConfirm);
    modalCancel.addEventListener("click", onCancel);
    modalInput.addEventListener("keydown", onKey);
  });
}

/* ─── Panel buttons ─── */
newFileBtn.addEventListener("click", async () => {
  const name = await promptModal("New file name:");
  if (name) {
    await fsWriteFile(name, "");
    await renderTree();
    openFile(name);
  }
});

newFolderBtn.addEventListener("click", async () => {
  const name = await promptModal("New folder name:");
  if (name) {
    await fsCreateDirectory(name);
    await renderTree();
  }
});

saveBtn.addEventListener("click", () => {
  if (editor && editor.activePath) {
    saveFile(editor.activePath);
  }
});

importZipBtn.addEventListener("click", () => {
  zipImportInput.click();
});

zipImportInput.addEventListener("change", async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  try {
    const imported = await importZip(file);
    for (const f of imported) {
      await fsWriteFile(f.path, f.contents);
    }
    await renderTree();
    setStatus(`Imported ${imported.length} file(s) from ZIP`, "ok");
  } catch (err) {
    setStatus("ZIP import failed: " + err.message, "error");
  }
  zipImportInput.value = "";
});

exportZipBtn.addEventListener("click", async () => {
  try {
    const files = await collectExecutionFiles();
    const blob = await exportZip(files);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (moduleName || "workspace") + ".zip";
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
    setStatus("Exported ZIP", "ok");
  } catch (err) {
    setStatus("ZIP export failed: " + err.message, "error");
  }
});

/* ─── go.mod awareness ─── */
async function updateModuleName() {
  let hasGoMod = false;
  if (useFSAPI && dirHandle) {
    try {
      const text = await readFile(dirHandle, "go.mod");
      const m = text.match(/^module\s+(\S+)/m);
      moduleName = m ? m[1] : "";
      hasGoMod = true;
    } catch {
      moduleName = "";
    }
  } else {
    try {
      const text = memoryFS.readFile("go.mod");
      const m = text.match(/^module\s+(\S+)/m);
      moduleName = m ? m[1] : "";
      hasGoMod = true;
    } catch {
      moduleName = "";
    }
  }
  moduleNameEl.textContent = moduleName;
  initModuleBtn.hidden = hasGoMod;
}

initModuleBtn.addEventListener("click", async () => {
  const name = await promptModal("Module name (e.g., example.com/app):", "example.com/app");
  if (!name) return;
  const content = `module ${name}\n\ngo 1.22\n`;
  await fsWriteFile("go.mod", content);
  await renderTree();
  openFile("go.mod");
  setStatus("Initialized module " + name, "ok");
});

/* ─── Terminal input ─── */
terminalInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    const line = terminalInput.value;
    terminalInput.value = "";
    logOutput("$ " + line);
    stdinQueue.push(line + "\n");
    // In a full implementation, stdin would be forwarded to the WASM runtime.
    // The current engine worker model doesn't expose stdin directly,
    // so we queue it for future use.
  }
});

/* ─── Init ─── */
async function init() {
  initWorker();

  if (useFSAPI) {
    const saved = await restoreDirectory();
    if (saved) {
      dirHandle = saved;
      setStatus("Restored workspace");
    } else {
      setStatus('Click "Open Folder" to select a workspace');
      // Show an open-folder button in the file panel
      const openBtn = document.createElement("button");
      openBtn.textContent = "Open Folder";
      openBtn.style.margin = "12px";
      openBtn.addEventListener("click", async () => {
        const h = await pickDirectory();
        if (h) {
          dirHandle = h;
          openBtn.remove();
          await renderTree();
          setStatus("Workspace loaded");
        }
      });
      fileTree.appendChild(openBtn);
    }
  }

  await renderTree();
  syncControls();
}

init().catch((e) => {
  console.error(e);
  setStatus("Init error: " + e.message, "error");
});
