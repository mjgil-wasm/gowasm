import {
  cloneWorkspaceFiles,
  configuredModuleRootsAreFresh,
  DEFAULT_BROWSER_WORKSPACE_FILES,
  editableWorkspaceFiles,
  findWorkspaceFile,
  moduleBundlesToWorkspaceFiles,
  moduleRootsConfigKey,
  normalizeWorkspacePath,
  parseModuleGraphRoots,
  upsertWorkspaceFile,
} from "./browser-workspace.js";
import { createBrowserShellControllers } from "./browser-shell-controllers.js";
import { createWorkerShellBackend } from "./browser-shell-backend.js";
import { createPackagedExampleController } from "./browser-shell-examples.js";
import { createShellActions } from "./browser-shell-actions.js";
import { createBrowserShellTestHooks } from "./browser-shell-test-hooks.js";
import {
  cancelledRequestView,
  cancellationPendingStatus,
  cancellationTimeoutView,
  isCancellableRequestKind,
} from "./browser-shell-cancellation.js";
import {
  createBrowserShellDiagnosticUi,
  createHostIssue,
  createProtocolIssue,
  createToolingIssue,
} from "./browser-shell-diagnostic-ui.js";
import { browserShellDom } from "./browser-shell-dom.js";
import {
  evaluateBrowserCompatibility,
  formatBrowserCompatibilityReport,
} from "./browser-compatibility.js";
import { startBrowserShellApp } from "./browser-shell-app-init.js";
import {
  describeModuleStatus,
  formatDiagnostics,
  formatLoadedModulesOutput,
  formatRunResult,
  formatTestResult,
  testRunnerLabel,
} from "./browser-shell-output.js";
import {
  renderWorkspaceChrome,
  resolveEntryPath,
  resolvePackageTargetPath,
  resolveSelectedDisplayFile,
  resolveSelectedWorkspaceFile,
} from "./browser-shell-workspace-ui.js";
import { selectionOffsetsForTarget } from "./browser-shell-source-links.js";
import { EditorView } from "https://esm.sh/@codemirror/view";
import { EditorSelection, Compartment } from "https://esm.sh/@codemirror/state";
import { basicSetup } from "https://esm.sh/codemirror";
import { go } from "https://esm.sh/@codemirror/lang-go";

const {
  addFileButton, archiveFileInput, archiveImportButton, archiveUrlImportButton, archiveUrlInput,
  bootUrlLoadButton, bootUrlPanel, bootUrlStatus, cacheStatus, cancelButton,
  clearAllCachesButton, clearModuleCacheButton, clearWorkspaceCacheButton, compatibilityStatus,
  diagnosticSummary, diagnosticSummaryPanel, editor, editorFileLabel, editorTabs, entryPathInput, exampleSelect,
  exampleSummary, fileList, filePathInput, formatButton, lintButton, loadExampleButton,
  loadModulesButton, moduleRootsInput, moduleStatus, output, packageTargetInput,
  refreshCacheButton, removeFileButton, renameFileButton, renamePathInput,
  restoreCachedWorkspaceButton, runButton, sidebar, sidebarToggle, snapshotExportButton, snapshotFileInput,
  snapshotImportButton, sourceLinks, sourceLinksPanel, status, testPackageButton,
  testSnippetButton, useSelectedEntryButton, useSelectedPackageButton, workspaceDirtyStatus,
  workspaceSelectionNote, workspaceShell, workspaceTree,
} = browserShellDom;

const EMPTY_MODULES_KEY = moduleRootsConfigKey([]);

entryPathInput.value = "main.go";
packageTargetInput.value = "";

let workspaceFiles = cloneWorkspaceFiles(DEFAULT_BROWSER_WORKSPACE_FILES);
let workspaceBaselineFiles = cloneWorkspaceFiles(DEFAULT_BROWSER_WORKSPACE_FILES);
let selectedFilePath = "main.go";
let loadedModuleBundles = [];
let loadedModuleRootsKey = EMPTY_MODULES_KEY;
let requestedModuleRootsKey = EMPTY_MODULES_KEY;
let loadedModuleBundlesStale = false;
let lastModuleLoadError = "";
let pendingModuleContinuation = null;
let requestedModuleCount = 0;
let archiveImportPending = false;
let cacheOperationPending = false;
const lastSuggestedRenamePath = { value: "" };
let exampleController;
let bootController;
let snapshotController;
let cacheController;

let editorView = null;
let lastRenderedPath = "";
let openTabs = [];
let activeTabPath = "";
const editableCompartment = new Compartment();

const browserCompatibility = evaluateBrowserCompatibility(window);
const browserShellTestHooks = createBrowserShellTestHooks(window);
const unsupportedBrowserMessage = "Browser compatibility check failed.\n\n" + formatBrowserCompatibilityReport(browserCompatibility);

function applyEditorSelectionCm(view, fileContents, target) {
  const selection = selectionOffsetsForTarget(fileContents, target);
  if (!selection) return false;
  view.focus();
  view.dispatch({
    selection: EditorSelection.create([EditorSelection.range(selection.start, selection.end)]),
    scrollIntoView: true,
  });
  return true;
}

const { setOutputView } = createBrowserShellDiagnosticUi({
  diagnosticSummaryElement: diagnosticSummary,
  diagnosticSummaryPanelElement: diagnosticSummaryPanel,
  getDisplayFiles: () => currentDisplayFiles(),
  linksElement: sourceLinks,
  outputElement: output,
  renderWorkspace,
  setSelectedFilePath(nextPath) {
    selectedFilePath = nextPath;
  },
  applyEditorSelection: (contents, link) => applyEditorSelectionCm(editorView, contents, link),
  sourceLinksPanelElement: sourceLinksPanel,
  statusElement: status,
});

const shellBackend = createWorkerShellBackend({
  cancellationTimeoutView,
  clearTimer(timerId) {
    self.clearTimeout(timerId);
  },
  createWorker() {
    return new Worker("./engine-worker.js", { type: "module" });
  },
  deferResponse(callback) {
    browserShellTestHooks.maybeDelayNextWorkerResponse(callback);
  },
  isCancellableRequestKind,
  onBooting({ statusMessage }) {
    status.textContent = statusMessage;
    renderModuleStatus();
    syncControls();
  },
  onCancellationPending({ requestKind }) {
    status.textContent = cancellationPendingStatus(requestKind);
    syncControls();
  },
  onCancellationTimeout({ timeoutView }) {
    setOutputView(timeoutView.outputText, []);
  },
  onReady({ info, readySuffix }) {
    status.textContent = readySuffix
      ? `Worker ready: ${info.engine_name} (${readySuffix})`
      : `Worker ready: ${info.engine_name}`;
    bootController?.maybeRunBootAction();
    renderModuleStatus();
    syncControls();
  },
  onResponse({ requestKind, response }) {
    switch (response.kind) {
    case "module_graph_result": {
      loadedModuleBundles = Array.isArray(response.modules) ? response.modules : [];
      loadedModuleRootsKey = requestedModuleRootsKey;
      loadedModuleBundlesStale = false;
      lastModuleLoadError = "";
      renderWorkspace();
      renderModuleStatus();
      void cacheController.refreshCacheStatus();

      const continuation = pendingModuleContinuation;
      clearPendingModuleContinuation();
      if (continuation) {
        status.textContent = `Loaded ${loadedModuleBundles.length} module bundle(s); continuing…`;
        syncControls();
        continuation();
        return;
      }

      status.textContent = loadedModuleBundles.length
        ? "Module bundles loaded"
        : "Module bundles cleared";
      setOutputView(formatLoadedModulesOutput(loadedModuleBundles), []);
      break;
    }
    case "run_result":
      status.textContent = "Worker responded";
      setOutputView(formatRunResult(response.stdout, response.diagnostics), response.diagnostics);
      break;
    case "test_result":
      status.textContent = `${testRunnerLabel(response.runner)} ${response.passed ? "passed" : "failed"}`;
      setOutputView(formatTestResult(
        response.runner,
        response.passed,
        response.stdout,
        response.diagnostics,
        response.details,
      ), response.diagnostics);
      break;
    case "lint_result":
      status.textContent = response.diagnostics?.length ? "Lint diagnostics received" : "Lint complete";
      setOutputView(formatDiagnostics(response.diagnostics) || "No lint findings.", response.diagnostics);
      break;
    case "format_result":
      setWorkspaceFiles(response.files, selectedFilePath);
      status.textContent = response.diagnostics?.length
        ? "Format diagnostics received"
        : "Format complete";
      setOutputView(formatDiagnostics(response.diagnostics) || "Formatting applied.", response.diagnostics);
      break;
    case "diagnostics":
      status.textContent = "Diagnostics received";
      setOutputView(formatDiagnostics(response.diagnostics), response.diagnostics);
      break;
    case "fatal":
      if (requestKind === "load_module_graph") {
        loadedModuleBundlesStale = loadedModuleBundles.length > 0;
        lastModuleLoadError = response.message;
      }
      clearPendingModuleContinuation();
      status.textContent = "Worker failed";
      setOutputView(response.message, [], [
        createHostIssue(response.message, {
          suggestedAction:
            "Retry the request. If the worker keeps failing, reload the page and rerun.",
        }),
      ]);
      break;
    case "cancelled": {
      const cancelledView = cancelledRequestView(requestKind);
      status.textContent = cancelledView.statusText;
      setOutputView(cancelledView.outputText, []);
      break;
    }
    default:
      clearPendingModuleContinuation();
      status.textContent = "Unknown worker message";
      setOutputView(JSON.stringify(response, null, 2), [], [
        createProtocolIssue("Unknown worker response kind received.", {
          suggestedAction:
            "Reload the page or inspect the raw worker payload below before retrying.",
        }),
      ]);
      break;
    }

    renderModuleStatus();
    syncControls();
  },
  onWorkerError({ requestKind, message, filename, lineno, colno, stackText }) {
    if (requestKind === "load_module_graph") {
      loadedModuleBundlesStale = loadedModuleBundles.length > 0;
      lastModuleLoadError = message || "Worker error";
    }
    status.textContent = "Worker failed";
    setOutputView(message || "Worker error", [], [
      createHostIssue(message || "Worker error", {
        filePath: filename,
        line: lineno,
        column: colno,
        stackText,
        suggestedAction:
          "Retry the request. If the worker keeps failing, reload the page and rerun.",
      }),
    ]);
    renderModuleStatus();
    syncControls();
  },
  onStateChange() {
    renderModuleStatus();
    syncControls();
  },
  setTimer(callback, durationMs) {
    return self.setTimeout(callback, durationMs);
  },
});

const shellActions = createShellActions({
  archiveUrlInput,
  beginWorkerRequest: beginBackendRequest,
  configuredModulesMatchLoaded,
  currentEditableWorkspaceFiles,
  currentEntryPath,
  currentExecutionFiles,
  currentPackageTargetPath,
  clearSelectedExample: () => exampleController?.clearSelectedExample(),
  emptyModulesKey: EMPTY_MODULES_KEY,
  entryPathInput,
  filePathInput,
  renamePathInput,
  getLoadedModuleBundles: () => loadedModuleBundles,
  getWorkspaceFiles: () => workspaceFiles,
  isBusy,
  onImportedWorkspace: async (cacheSource) => {
    await cacheController.rememberImportedWorkspace(cacheSource);
  },
  packageTargetInput,
  parseConfiguredModules,
  renderModuleStatus,
  renderWorkspace,
  selectedWorkspaceFile,
  setArchiveImportPending: (value) => {
    archiveImportPending = value;
  },
  setLastModuleLoadError: (value) => {
    lastModuleLoadError = value;
  },
  setLoadedModuleBundles: (value) => {
    loadedModuleBundles = value;
  },
  setLoadedModuleBundlesStale: (value) => {
    loadedModuleBundlesStale = value;
  },
  setLoadedModuleRootsKey: (value) => {
    loadedModuleRootsKey = value;
  },
  setOutputView,
  setPendingModuleContinuation: (value) => {
    pendingModuleContinuation = value;
  },
  setRequestedModuleCount: (value) => {
    requestedModuleCount = value;
  },
  setRequestedModuleRootsKey: (value) => {
    requestedModuleRootsKey = value;
  },
  setWorkspaceFiles,
  statusElement: status,
  syncControls,
});

const {
  addWorkspaceFile,
  importArchiveFromFile,
  importArchiveFromUrl,
  removeSelectedWorkspaceFile,
  requestFormat,
  requestLint,
  requestPackageTest,
  requestRun,
  requestSnippetTest,
  renameSelectedWorkspaceFile,
  setSelectedAsEntryPath,
  setSelectedAsPackageTarget,
  startModuleLoad,
} = shellActions;

const resetLoadedModules = () => {
  loadedModuleBundles = []; loadedModuleRootsKey = EMPTY_MODULES_KEY;
  loadedModuleBundlesStale = false; lastModuleLoadError = "";
};

({ bootController, snapshotController, cacheController } = createBrowserShellControllers({
  bootUrlPanel,
  bootUrlStatus,
  cacheStatusElement: cacheStatus,
  downloadSnapshotBlob: (blob, fileName) => {
    const anchor = document.createElement("a");
    const objectUrl = URL.createObjectURL(blob);
    anchor.href = objectUrl;
    anchor.download = fileName;
    anchor.click();
    URL.revokeObjectURL(objectUrl);
  },
  entryPathInput,
  getLoadedModuleBundles: () => loadedModuleBundles,
  getPackagedExampleIds: () => exampleController?.packagedExampleIds() ?? [],
  getSelectedExampleId: () => exampleController?.getSelectedExampleId() ?? "",
  getSelectedFilePath: () => selectedFilePath,
  getWorkspaceFiles: () => workspaceFiles,
  isBusy,
  isWorkerReady: () => shellBackend.currentState().ready,
  moduleRootsInput,
  packageTargetInput,
  renderModuleStatus,
  renderWorkspace,
  requestPackageTest,
  requestRun,
  requestSnippetTest,
  resetLoadedModules,
  setSelectedExampleId: (value) => exampleController?.setSelectedExampleId(value),
  search: window.location.search,
  setOutputView,
  setWorkspaceFiles,
  statusElement: status,
  syncCacheControls: () => {
    cacheOperationPending = cacheController.isPending();
    syncControls();
  },
  syncControls,
}));

exampleController = createPackagedExampleController({
  cacheSchemaVersion: cacheController.cacheSchemaVersion ?? 1,
  entryPathInput,
  exampleSelect,
  exampleSummary,
  isBusy,
  loadExampleButton,
  moduleRootsInput,
  packageTargetInput,
  renderModuleStatus,
  renderWorkspace,
  resetLoadedModules,
  seedPackagedExamples: (records) => cacheController.seedPackagedExamples(records),
  setOutputView,
  setWorkspaceFiles,
  statusElement: status,
  syncControls,
});

startBrowserShellApp({
  addFileButton,
  addWorkspaceFile,
  archiveFileInput,
  archiveImportButton,
  archiveUrlImportButton,
  archiveUrlInput,
  bootController,
  bootUrlLoadButton,
  bootWorker: (statusMessage, readySuffix = "") => shellBackend.boot(statusMessage, readySuffix),
  browserCompatibility,
  cacheController,
  cancelButton,
  cancelRun: cancelRequest,
  clearAllCachesButton,
  clearModuleCacheButton,
  clearWorkspaceCacheButton,
  entryPathInput,
  exampleController,
  exampleSelect,
  fileList,
  filePathInput,
  formatButton,
  handleFileListChange,
  handleLoadModules,
  handleModuleRootsInput,
  handleWorkspaceTargetInput,
  importArchiveFromFile,
  importArchiveFromUrl,
  isBusy,
  lintButton,
  loadExampleButton,
  loadModulesButton,
  moduleRootsInput,
  packageTargetInput,
  refreshCacheButton,
  removeFileButton,
  removeSelectedWorkspaceFile,
  renameFileButton,
  renamePathInput,
  renameSelectedWorkspaceFile,
  renderCompatibilityStatus,
  renderModuleStatus,
  renderWorkspace,
  requestFormat,
  requestLint,
  requestPackageTest,
  requestRun,
  requestSnippetTest,
  restoreCachedWorkspaceButton,
  runButton,
  setOutputView,
  setSelectedAsEntryPath,
  setSelectedAsPackageTarget,
  snapshotController,
  snapshotExportButton,
  snapshotFileInput,
  snapshotImportButton,
  statusElement: status,
  syncControls,
  testPackageButton,
  testSnippetButton,
  unsupportedBrowserMessage,
  useSelectedEntryButton,
  useSelectedPackageButton,
});

function isBusy() {
  return shellBackend.currentState().activeRequestKind !== null || archiveImportPending || cacheOperationPending || bootController.isPending();
}

function canCancelActiveRequest() {
  return isCancellableRequestKind(shellBackend.currentState().activeRequestKind);
}

function currentEditableWorkspaceFiles() {
  return editableWorkspaceFiles(workspaceFiles);
}

function currentExecutionFiles() {
  return moduleBundlesToWorkspaceFiles(currentEditableWorkspaceFiles(), loadedModuleBundles);
}

function currentDisplayFiles() {
  return currentExecutionFiles();
}

function currentEntryPath() {
  return resolveEntryPath(entryPathInput.value);
}

function currentPackageTargetPath() {
  return resolvePackageTargetPath(
    packageTargetInput.value,
    selectedWorkspaceFile(),
    entryPathInput.value,
  );
}

function selectedDisplayFile() {
  return resolveSelectedDisplayFile(currentDisplayFiles(), selectedFilePath);
}

function selectedWorkspaceFile() {
  return resolveSelectedWorkspaceFile(workspaceFiles, selectedFilePath);
}

function selectedFileIsEditable() {
  return Boolean(selectedWorkspaceFile());
}

function parseConfiguredModules() {
  return parseModuleGraphRoots(moduleRootsInput.value);
}

function configuredModulesMatchLoaded(modules) {
  return configuredModuleRootsAreFresh(modules, loadedModuleRootsKey, loadedModuleBundlesStale);
}

function renderCompatibilityStatus() {
  compatibilityStatus.textContent = formatBrowserCompatibilityReport(browserCompatibility);
}

function isFileDirty(path) {
  const baseline = workspaceBaselineFiles.find((f) => f.path === path);
  const current = workspaceFiles.find((f) => f.path === path);
  if (!current) return false;
  if (!baseline) return true;
  return baseline.contents !== current.contents;
}

function openOrFocusTab(path) {
  if (!openTabs.some((t) => t.path === path)) {
    openTabs.push({ path });
  }
  activeTabPath = path;
}

function closeTab(path) {
  openTabs = openTabs.filter((t) => t.path !== path);
  if (activeTabPath === path) {
    activeTabPath = openTabs.length > 0 ? openTabs[openTabs.length - 1].path : "";
    if (activeTabPath) {
      selectedFilePath = activeTabPath;
    } else if (workspaceFiles.length > 0) {
      selectedFilePath = workspaceFiles[0].path;
      activeTabPath = selectedFilePath;
      openTabs.push({ path: selectedFilePath });
    } else {
      selectedFilePath = "";
      activeTabPath = "";
    }
  }
  renderWorkspace();
}

function renderTabs() {
  editorTabs.replaceChildren();
  for (const tab of openTabs) {
    const btn = document.createElement("button");
    btn.className = "editor-tab" + (tab.path === activeTabPath ? " active" : "");
    btn.type = "button";

    const label = document.createElement("span");
    label.textContent = tab.path.split("/").pop();
    btn.append(label);

    if (isFileDirty(tab.path)) {
      const dot = document.createElement("span");
      dot.className = "tab-dirty";
      btn.append(dot);
    }

    const close = document.createElement("button");
    close.className = "tab-close";
    close.type = "button";
    close.textContent = "×";
    close.addEventListener("click", (e) => {
      e.stopPropagation();
      closeTab(tab.path);
    });
    btn.append(close);

    btn.addEventListener("click", () => {
      activeTabPath = tab.path;
      selectedFilePath = tab.path;
      renderWorkspace();
    });

    editorTabs.append(btn);
  }
}

function createEditor(initialContent) {
  editorView = new EditorView({
    doc: initialContent,
    extensions: [
      basicSetup,
      go(),
      editableCompartment.of(EditorView.editable.of(true)),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          handleSourceInput(update.state.doc.toString());
        }
      }),
      EditorView.theme({
        "&": { height: "100%" },
        ".cm-scroller": { overflow: "auto" },
      }),
    ],
    parent: editor,
  });
  window._codeEditorView = editorView;
}

function setWorkspaceFiles(nextFiles, preferredSelectedPath = selectedFilePath, options = {}) {
  workspaceFiles = cloneWorkspaceFiles(nextFiles);
  if (options.resetDirtyBaseline) {
    workspaceBaselineFiles = cloneWorkspaceFiles(nextFiles);
  }

  const preferredPath = normalizeWorkspacePath(preferredSelectedPath);
  if (preferredPath && findWorkspaceFile(workspaceFiles, preferredPath)) {
    selectedFilePath = preferredPath;
  } else if (workspaceFiles.length > 0) {
    selectedFilePath = workspaceFiles[0].path;
  } else {
    selectedFilePath = "";
  }

  openTabs = openTabs.filter((t) => workspaceFiles.some((f) => f.path === t.path));
  if (selectedFilePath && !openTabs.some((t) => t.path === selectedFilePath)) {
    openTabs.push({ path: selectedFilePath });
  }
  activeTabPath = selectedFilePath;

  renderWorkspace();
}

function renderWorkspace() {
  const displayFiles = currentDisplayFiles();
  const nextSelected = selectedDisplayFile();
  editorFileLabel.textContent = !nextSelected
    ? "No file selected"
    : selectedFileIsEditable()
      ? nextSelected.path
      : `${nextSelected.path} (projected read-only)`;

  const previousValue = fileList.value;
  fileList.replaceChildren();
  for (const file of displayFiles) {
    const option = document.createElement("option");
    option.value = file.path;
    option.textContent = file.path;
    if (file.path === previousValue || file.path === selectedFilePath) {
      option.selected = true;
    }
    fileList.append(option);
  }

  if (!nextSelected && displayFiles.length > 0) {
    selectedFilePath = displayFiles[0].path;
  }
  const selected = selectedDisplayFile();

  if (selected) {
    if (!editorView) {
      createEditor(selected.contents);
    } else {
      const currentDoc = editorView.state.doc.toString();
      const changedFile = selected.path !== lastRenderedPath;
      if (changedFile || currentDoc !== selected.contents) {
        editorView.dispatch({
          changes: { from: 0, to: editorView.state.doc.length, insert: selected.contents },
          selection: EditorSelection.cursor(0),
        });
      }
    }
    lastRenderedPath = selected.path;
    if (selected.path) {
      fileList.value = selected.path;
    }
    openOrFocusTab(selected.path);
  } else {
    if (editorView) {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: "" },
        selection: EditorSelection.cursor(0),
      });
    }
    lastRenderedPath = "";
  }

  renderTabs();
  renderWorkspaceSidebarState();
  syncControls();
}

function renderWorkspaceSidebarState() {
  renderWorkspaceChrome({
    activeElement: document.activeElement,
    baselineFiles: workspaceBaselineFiles,
    disableSelection: isBusy(),
    dirtyStatusElement: workspaceDirtyStatus,
    displayFiles: currentDisplayFiles(),
    editableFiles: currentEditableWorkspaceFiles(),
    entryPath: currentEntryPath(),
    lastSuggestedRenamePath,
    onSelectPath: handleFileListChange,
    packageTargetPath: currentPackageTargetPath(),
    renamePathInput,
    selectedDisplayFile: selectedDisplayFile(),
    selectedPath: selectedFilePath,
    selectedWorkspaceFile: selectedWorkspaceFile(),
    selectionNoteElement: workspaceSelectionNote,
    selectElement: fileList,
    treeElement: workspaceTree,
  });
}

function renderModuleStatus() {
  const { modules, errors } = parseConfiguredModules();
  moduleStatus.textContent = describeModuleStatus({
    modules,
    errors,
    isLoading: shellBackend.currentState().activeRequestKind === "load_module_graph",
    requestedModuleCount,
    loadedBundles: loadedModuleBundles,
    loadedBundlesStale: loadedModuleBundlesStale,
    configuredModulesMatchLoaded: configuredModulesMatchLoaded(modules),
    lastLoadError: lastModuleLoadError,
  });
}

function syncControls() {
  const busy = isBusy();
  const backendState = shellBackend.currentState();
  const selected = selectedDisplayFile();
  const workerActionsAvailable = browserCompatibility.supported && backendState.ready && !busy;
  const editableSelection = selectedFileIsEditable();
  const goEditableSelection = editableSelection && Boolean(selectedWorkspaceFile()?.path.endsWith(".go"));
  const cacheAvailable = browserCompatibility.cachePersistenceSupported;
  const urlFetchAvailable = browserCompatibility.features.some(
    (feature) => feature.id === "fetch" && feature.supported,
  );

  fileList.disabled = busy;
  filePathInput.disabled = busy;
  renamePathInput.disabled = busy || !editableSelection;
  archiveFileInput.disabled = busy;
  archiveImportButton.disabled = busy;
  archiveUrlInput.disabled = busy || !urlFetchAvailable;
  archiveUrlImportButton.disabled = busy || !urlFetchAvailable;
  bootUrlLoadButton.disabled =
    busy ||
    !urlFetchAvailable ||
    !bootController.bootRequest.present ||
    bootController.bootRequest.errors.length > 0;
  snapshotExportButton.disabled = busy;
  snapshotImportButton.disabled = busy;
  snapshotFileInput.disabled = busy;
  entryPathInput.disabled = busy;
  packageTargetInput.disabled = busy;
  moduleRootsInput.disabled = busy;
  addFileButton.disabled = busy;
  removeFileButton.disabled = busy || !editableSelection;
  renameFileButton.disabled = busy || !editableSelection;
  useSelectedEntryButton.disabled = busy || !goEditableSelection;
  useSelectedPackageButton.disabled = busy || !goEditableSelection;
  exampleController?.syncControls();
  refreshCacheButton.disabled = busy || !cacheAvailable;
  restoreCachedWorkspaceButton.disabled = busy || !cacheAvailable;
  clearWorkspaceCacheButton.disabled = busy || !cacheAvailable;
  clearModuleCacheButton.disabled = busy || !cacheAvailable;
  clearAllCachesButton.disabled = busy || !cacheAvailable;
  loadModulesButton.disabled = !workerActionsAvailable;
  testPackageButton.disabled = !workerActionsAvailable;
  testSnippetButton.disabled = !workerActionsAvailable;
  lintButton.disabled = !workerActionsAvailable;
  formatButton.disabled = !workerActionsAvailable;
  runButton.disabled = !workerActionsAvailable;
  cancelButton.disabled = !canCancelActiveRequest() || backendState.cancelPending;

  if (editorView) {
    const editable = !busy && selected && editableSelection;
    editorView.dispatch({
      effects: editableCompartment.reconfigure(EditorView.editable.of(editable)),
    });
  }
}

function clearPendingModuleContinuation() {
  pendingModuleContinuation = null;
}
function cancelRequest() {
  shellBackend.cancel();
}

function beginBackendRequest(kind, statusMessage, outputMessage, buildRequest) {
  if (!shellBackend.currentState().ready || isBusy()) {
    return;
  }

  const request = buildRequest();
  if (!request) {
    syncControls();
    return;
  }

  status.textContent = statusMessage;
  if (outputMessage !== null) {
    setOutputView(outputMessage, []);
  }
  renderModuleStatus();
  syncControls();
  shellBackend.send(kind, request);
}

function handleFileListChange(nextPathValue) {
  const nextPath = normalizeWorkspacePath(nextPathValue);
  if (!nextPath) {
    return;
  }
  selectedFilePath = nextPath;
  openOrFocusTab(nextPath);
  renderWorkspace();
}

function handleSourceInput(nextValue) {
  const selected = selectedWorkspaceFile();
  if (!selected) {
    return;
  }
  workspaceFiles = upsertWorkspaceFile(workspaceFiles, {
    path: selected.path,
    contents: nextValue,
  });
  renderWorkspaceSidebarState();
  renderTabs();
  syncControls();
}

function handleModuleRootsInput() {
  lastModuleLoadError = "";
  renderModuleStatus();
  syncControls();
}

function handleWorkspaceTargetInput() {
  renderWorkspaceSidebarState();
  syncControls();
}

function handleLoadModules() {
  if (!shellBackend.currentState().ready || isBusy()) {
    return;
  }

  const { modules, errors } = parseConfiguredModules();
  if (errors.length > 0) {
    status.textContent = "Module roots config is invalid";
    setOutputView(
      errors.join("\n"),
      [],
      errors.map((message) =>
        createToolingIssue(message, {
          suggestedAction: "Fix the module root list and retry Load Modules.",
        }),
      ),
    );
    renderModuleStatus();
    syncControls();
    return;
  }

  if (modules.length === 0) {
    loadedModuleBundles = [];
    loadedModuleRootsKey = EMPTY_MODULES_KEY;
    loadedModuleBundlesStale = false;
    lastModuleLoadError = "";
    status.textContent = "Module bundles cleared";
    renderWorkspace();
    setOutputView("No remote modules configured.", []);
    renderModuleStatus();
    void cacheController.refreshCacheStatus();
    syncControls();
    return;
  }

  startModuleLoad({
    modules,
    outputMessage: "",
    continuation: null,
  });
}

sidebarToggle.addEventListener("click", () => {
  workspaceShell.classList.toggle("collapsed");
});
