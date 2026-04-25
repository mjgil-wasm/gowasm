export function bindBrowserShellEvents(ctx) {
  ctx.fileList.addEventListener("change", () => {
    ctx.handleFileListChange(ctx.fileList.value);
  });

  ctx.source.addEventListener("input", () => {
    ctx.handleSourceInput(ctx.source.value);
  });

  ctx.exampleSelect.addEventListener("change", () => {
    ctx.handleExampleSelectionChange(ctx.exampleSelect.value);
  });

  ctx.entryPathInput.addEventListener("input", () => {
    ctx.handleWorkspaceTargetInput();
  });

  ctx.packageTargetInput.addEventListener("input", () => {
    ctx.handleWorkspaceTargetInput();
  });

  ctx.filePathInput.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") {
      return;
    }
    event.preventDefault();
    ctx.addWorkspaceFile();
  });

  ctx.archiveUrlInput.addEventListener("keydown", async (event) => {
    if (event.key !== "Enter") {
      return;
    }
    event.preventDefault();
    await ctx.importArchiveFromUrl();
  });

  ctx.moduleRootsInput.addEventListener("input", () => {
    ctx.handleModuleRootsInput();
  });

  ctx.addFileButton.addEventListener("click", () => {
    ctx.addWorkspaceFile();
  });

  ctx.renamePathInput.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") {
      return;
    }
    event.preventDefault();
    ctx.renameSelectedWorkspaceFile();
  });

  ctx.renameFileButton.addEventListener("click", () => {
    ctx.renameSelectedWorkspaceFile();
  });

  ctx.archiveImportButton.addEventListener("click", () => {
    if (ctx.isBusy()) {
      return;
    }
    ctx.archiveFileInput.click();
  });

  ctx.archiveFileInput.addEventListener("change", async () => {
    const file = ctx.archiveFileInput.files?.[0] ?? null;
    ctx.archiveFileInput.value = "";
    await ctx.importArchiveFromFile(file);
  });

  ctx.archiveUrlImportButton.addEventListener("click", async () => {
    await ctx.importArchiveFromUrl();
  });

  ctx.bootUrlLoadButton.addEventListener("click", async () => {
    await ctx.loadBootManifest();
  });

  ctx.loadExampleButton.addEventListener("click", async () => {
    await ctx.loadSelectedExample();
  });

  ctx.snapshotExportButton.addEventListener("click", async () => {
    await ctx.exportSnapshot();
  });

  ctx.snapshotImportButton.addEventListener("click", () => {
    if (ctx.isBusy()) {
      return;
    }
    ctx.snapshotFileInput.click();
  });

  ctx.snapshotFileInput.addEventListener("change", async () => {
    const file = ctx.snapshotFileInput.files?.[0] ?? null;
    ctx.snapshotFileInput.value = "";
    await ctx.importSnapshotFromFile(file);
  });

  ctx.refreshCacheButton.addEventListener("click", async () => {
    await ctx.refreshCacheStatus();
  });

  ctx.restoreCachedWorkspaceButton.addEventListener("click", async () => {
    await ctx.restoreCachedWorkspace();
  });

  ctx.clearWorkspaceCacheButton.addEventListener("click", async () => {
    await ctx.clearWorkspaceCache();
  });

  ctx.clearModuleCacheButton.addEventListener("click", async () => {
    await ctx.clearModuleCache();
  });

  ctx.clearAllCachesButton.addEventListener("click", async () => {
    await ctx.clearAllCaches();
  });

  ctx.removeFileButton.addEventListener("click", () => {
    ctx.removeSelectedWorkspaceFile();
  });

  ctx.useSelectedEntryButton.addEventListener("click", () => {
    ctx.setSelectedAsEntryPath();
  });

  ctx.useSelectedPackageButton.addEventListener("click", () => {
    ctx.setSelectedAsPackageTarget();
  });

  ctx.loadModulesButton.addEventListener("click", () => {
    ctx.handleLoadModules();
  });

  ctx.testPackageButton.addEventListener("click", () => {
    ctx.requestPackageTest();
  });

  ctx.testSnippetButton.addEventListener("click", () => {
    ctx.requestSnippetTest();
  });

  ctx.lintButton.addEventListener("click", () => {
    ctx.requestLint();
  });

  ctx.formatButton.addEventListener("click", () => {
    ctx.requestFormat();
  });

  ctx.runButton.addEventListener("click", () => {
    ctx.requestRun();
  });

  ctx.cancelButton.addEventListener("click", () => {
    ctx.cancelRun();
  });
}
