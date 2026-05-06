import {
  defaultWorkspaceFileContents,
  findWorkspaceFile,
  isGoWorkspacePath,
  isModuleCachePath,
  moduleRootsConfigKey,
  normalizeWorkspacePath,
  renameWorkspaceFile,
  removeWorkspaceFile,
  upsertWorkspaceFile,
} from "./browser-workspace.js";
import {
  formatArchiveImportSummary,
  importProjectArchiveBytes,
  importProjectArchiveUrl,
} from "./browser-archive.js";

export function createShellActions(ctx) {
  function ensureWorkspacePathExists(path, label) {
    const normalizedPath = normalizeWorkspacePath(path);
    if (!normalizedPath) {
      ctx.statusElement.textContent = `${label} path is required`;
      ctx.setOutputView(`Set a ${label.toLowerCase()} path before sending the request.`, []);
      return false;
    }
    if (!findWorkspaceFile(ctx.getWorkspaceFiles(), normalizedPath)) {
      ctx.statusElement.textContent = `${label} path not found`;
      ctx.setOutputView(`The editable workspace does not contain ${normalizedPath}.`, []);
      return false;
    }
    return true;
  }

  function startModuleLoad({ modules, outputMessage, continuation }) {
    ctx.setRequestedModuleRootsKey(moduleRootsConfigKey(modules));
    ctx.setRequestedModuleCount(modules.length);
    ctx.setPendingModuleContinuation(continuation ?? null);
    ctx.setLoadedModuleBundlesStale(ctx.getLoadedModuleBundles().length > 0);
    ctx.setLastModuleLoadError("");
    ctx.beginWorkerRequest(
      "load_module_graph",
      `Loading ${modules.length} module bundle(s)…`,
      outputMessage,
      () => ({
        kind: "load_module_graph",
        modules,
      }),
    );
  }

  function ensureModulesLoadedForExecution(continuation) {
    const { modules, errors } = ctx.parseConfiguredModules();
    if (errors.length > 0) {
      ctx.statusElement.textContent = "Module roots config is invalid";
      ctx.setOutputView(errors.join("\n"), []);
      ctx.renderModuleStatus();
      ctx.syncControls();
      return false;
    }

    if (modules.length === 0) {
      ctx.setLoadedModuleBundles([]);
      ctx.setLoadedModuleRootsKey(ctx.emptyModulesKey);
      ctx.setLoadedModuleBundlesStale(false);
      ctx.setLastModuleLoadError("");
      ctx.renderWorkspace();
      ctx.renderModuleStatus();
      continuation();
      return true;
    }

    if (ctx.configuredModulesMatchLoaded(modules)) {
      continuation();
      return true;
    }

    startModuleLoad({
      modules,
      outputMessage: "",
      continuation,
    });
    return false;
  }

  function requestPackageTest() {
    ensureModulesLoadedForExecution(() => {
      ctx.beginWorkerRequest(
        "test_package",
        "Sending package test request…",
        "",
        () => {
          const targetPath = ctx.currentPackageTargetPath();
          if (!ensureWorkspacePathExists(targetPath, "Package test target")) {
            return null;
          }
          return {
            kind: "test_package",
            target_path: targetPath,
            files: ctx.currentExecutionFiles(),
          };
        },
      );
    });
  }

  function requestSnippetTest() {
    ensureModulesLoadedForExecution(() => {
      ctx.beginWorkerRequest(
        "test_snippet",
        "Sending snippet test request…",
        "",
        () => {
          const entryPath = ctx.currentEntryPath();
          if (!ensureWorkspacePathExists(entryPath, "Snippet entry")) {
            return null;
          }
          return {
            kind: "test_snippet",
            entry_path: entryPath,
            files: ctx.currentExecutionFiles(),
          };
        },
      );
    });
  }

  function requestRun() {
    ensureModulesLoadedForExecution(() => {
      ctx.beginWorkerRequest(
        "run",
        "Sending run request…",
        "",
        () => {
          const entryPath = ctx.currentEntryPath();
          if (!ensureWorkspacePathExists(entryPath, "Run entry")) {
            return null;
          }
          return {
            kind: "run",
            entry_path: entryPath,
            files: ctx.currentExecutionFiles(),
          };
        },
      );
    });
  }

  function requestLint() {
    ctx.beginWorkerRequest(
      "lint",
      "Sending lint request…",
      "",
      () => ({
        kind: "lint",
        files: ctx.currentEditableWorkspaceFiles(),
      }),
    );
  }

  function requestFormat() {
    ctx.beginWorkerRequest(
      "format",
      "Sending format request…",
      "",
      () => ({
        kind: "format",
        files: ctx.currentEditableWorkspaceFiles(),
      }),
    );
  }

  function addWorkspaceFile() {
    if (ctx.isBusy()) {
      return;
    }

    const path = normalizeWorkspacePath(ctx.filePathInput.value);
    if (!path) {
      ctx.statusElement.textContent = "File path required";
      ctx.setOutputView("Enter a relative workspace file path before adding a file.", []);
      return;
    }
    if (isModuleCachePath(path)) {
      ctx.statusElement.textContent = "Module cache paths are reserved";
      ctx.setOutputView(
        "Paths under __module_cache__/ are projected from loaded remote bundles and cannot be added as editable workspace files.",
        [],
      );
      return;
    }

    const workspaceFiles = ctx.getWorkspaceFiles();
    const alreadyExists = Boolean(findWorkspaceFile(workspaceFiles, path));
    const nextFiles = upsertWorkspaceFile(workspaceFiles, {
      path,
      contents: alreadyExists
        ? findWorkspaceFile(workspaceFiles, path)?.contents ?? ""
        : defaultWorkspaceFileContents(path),
    });
    ctx.setWorkspaceFiles(nextFiles, path);
    ctx.filePathInput.value = "";
    ctx.statusElement.textContent = alreadyExists ? `Selected ${path}` : `Added ${path}`;
    if (!ctx.packageTargetInput.value && path.endsWith(".go")) {
      ctx.packageTargetInput.value = path;
    }
  }

  function renameSelectedWorkspaceFile() {
    if (ctx.isBusy()) {
      return;
    }

    const selected = ctx.selectedWorkspaceFile();
    if (!selected) {
      ctx.statusElement.textContent = "Editable file required";
      ctx.setOutputView("Select an editable workspace file before renaming it.", []);
      return;
    }

    const renamedPath = normalizeWorkspacePath(ctx.renamePathInput.value);
    if (!renamedPath) {
      ctx.statusElement.textContent = "Rename path required";
      ctx.setOutputView("Enter a relative workspace path before renaming the selected file.", []);
      return;
    }
    if (isModuleCachePath(renamedPath)) {
      ctx.statusElement.textContent = "Module cache paths are reserved";
      ctx.setOutputView(
        "Paths under __module_cache__/ are projected from loaded remote bundles and cannot be used for editable workspace files.",
        [],
      );
      return;
    }

    try {
      const nextFiles = renameWorkspaceFile(ctx.getWorkspaceFiles(), selected.path, renamedPath);
      ctx.setWorkspaceFiles(nextFiles, renamedPath);
      if (normalizeWorkspacePath(ctx.entryPathInput.value) === selected.path) {
        ctx.entryPathInput.value = renamedPath;
      }
      if (normalizeWorkspacePath(ctx.packageTargetInput.value) === selected.path) {
        ctx.packageTargetInput.value = renamedPath;
      }
      ctx.statusElement.textContent = `Renamed ${selected.path} to ${renamedPath}`;
    } catch (error) {
      ctx.statusElement.textContent = "Rename failed";
      ctx.setOutputView(error?.message || String(error), []);
    }
  }

  function removeSelectedWorkspaceFile() {
    if (ctx.isBusy()) {
      return;
    }

    const selected = ctx.selectedWorkspaceFile();
    if (!selected) {
      return;
    }

    const nextFiles = removeWorkspaceFile(ctx.getWorkspaceFiles(), selected.path);
    ctx.setWorkspaceFiles(nextFiles, nextFiles[0]?.path ?? "");
    if (normalizeWorkspacePath(ctx.entryPathInput.value) === selected.path) {
      ctx.entryPathInput.value = nextFiles[0]?.path ?? "";
    }
    if (normalizeWorkspacePath(ctx.packageTargetInput.value) === selected.path) {
      ctx.packageTargetInput.value = "";
    }
    ctx.statusElement.textContent = `Removed ${selected.path}`;
  }

  function setSelectedAsEntryPath() {
    if (ctx.isBusy()) {
      return;
    }

    const selected = ctx.selectedWorkspaceFile();
    if (!selected || !isGoWorkspacePath(selected.path)) {
      ctx.statusElement.textContent = "Go source entry required";
      ctx.setOutputView("Select an editable Go source file before setting the run/snippet entry path.", []);
      return;
    }

    ctx.entryPathInput.value = selected.path;
    ctx.statusElement.textContent = `Set run/snippet entry to ${selected.path}`;
    ctx.renderWorkspace();
  }

  function setSelectedAsPackageTarget() {
    if (ctx.isBusy()) {
      return;
    }

    const selected = ctx.selectedWorkspaceFile();
    if (!selected || !isGoWorkspacePath(selected.path)) {
      ctx.statusElement.textContent = "Go source package target required";
      ctx.setOutputView("Select an editable Go source file before setting the package test target.", []);
      return;
    }

    ctx.packageTargetInput.value = selected.path;
    ctx.statusElement.textContent = `Set package test target to ${selected.path}`;
    ctx.renderWorkspace();
  }

  async function importArchiveFromFile(file) {
    if (ctx.isBusy()) {
      return;
    }
    if (!file) {
      ctx.statusElement.textContent = "Archive file required";
      ctx.setOutputView("Choose a .zip archive before importing.", []);
      return;
    }

    await importArchiveBuffer(await file.arrayBuffer(), file.name || "archive.zip", {
      sourceKind: "archive_file",
      sourceLabel: file.name || "archive.zip",
    });
  }

  async function importArchiveFromUrl() {
    if (ctx.isBusy()) {
      return;
    }

    const archiveUrl = String(ctx.archiveUrlInput.value ?? "").trim();
    if (!archiveUrl) {
      ctx.statusElement.textContent = "Archive URL required";
      ctx.setOutputView("Enter an archive URL before importing.", []);
      return;
    }

    ctx.statusElement.textContent = "Downloading archive…";
    ctx.setArchiveImportPending(true);
    ctx.setOutputView("", []);
    ctx.syncControls();

    try {
      const imported = await importProjectArchiveUrl(archiveUrl);
      await applyImportedArchive(imported, {
        sourceKind: "archive_url",
        sourceLabel: archiveUrl,
      });
    } catch (error) {
      ctx.statusElement.textContent = "Archive import failed";
      ctx.setOutputView(error?.message || String(error), []);
    } finally {
      ctx.setArchiveImportPending(false);
      ctx.syncControls();
    }
  }

  async function importArchiveBuffer(buffer, sourceLabel, cacheSource) {
    ctx.statusElement.textContent = "Importing archive…";
    ctx.setArchiveImportPending(true);
    ctx.setOutputView("", []);
    ctx.syncControls();

    try {
      const imported = await importProjectArchiveBytes(buffer, { sourceLabel });
      await applyImportedArchive(imported, cacheSource);
    } catch (error) {
      ctx.statusElement.textContent = "Archive import failed";
      ctx.setOutputView(error?.message || String(error), []);
    } finally {
      ctx.setArchiveImportPending(false);
      ctx.syncControls();
    }
  }

  async function applyImportedArchive(imported, cacheSource) {
    ctx.setWorkspaceFiles(imported.files, imported.entryPath, { resetDirtyBaseline: true });
    ctx.clearSelectedExample?.();
    if (imported.entryPath) {
      ctx.entryPathInput.value = imported.entryPath;
    }
    if (imported.packageTargetPath) {
      ctx.packageTargetInput.value = imported.packageTargetPath;
    } else {
      ctx.packageTargetInput.value = "";
    }
    await ctx.onImportedWorkspace?.(cacheSource);
    ctx.statusElement.textContent = `Imported ${imported.files.length} archive file(s)`;
    ctx.setOutputView(formatArchiveImportSummary(imported), []);
  }

  return {
    addWorkspaceFile,
    importArchiveFromFile,
    importArchiveFromUrl,
    removeSelectedWorkspaceFile,
    renameSelectedWorkspaceFile,
    requestFormat,
    requestLint,
    requestPackageTest,
    requestRun,
    requestSnippetTest,
    setSelectedAsEntryPath,
    setSelectedAsPackageTarget,
    startModuleLoad,
  };
}
