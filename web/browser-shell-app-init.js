import { bindBrowserShellEvents } from "./browser-shell-events.js";

export function startBrowserShellApp(ctx) {
  if (ctx.browserCompatibility.supported) {
    ctx.bootWorker("Booting worker...");
  } else {
    ctx.statusElement.textContent = "Browser unsupported";
    ctx.setOutputView(ctx.unsupportedBrowserMessage, []);
  }

  ctx.renderWorkspace();
  ctx.renderModuleStatus();
  ctx.renderCompatibilityStatus();
  ctx.bootController.renderBootUrlPanel();
  ctx.exampleController.renderExampleCatalog();
  if (ctx.browserCompatibility.cachePersistenceSupported) {
    void ctx.exampleController.seedPackagedExamples();
  } else {
    void ctx.cacheController.refreshCacheStatus();
  }
  ctx.syncControls();
  if (ctx.browserCompatibility.supported) {
    ctx.bootController.maybeAutoloadBootManifest();
  }

  bindBrowserShellEvents({
    addFileButton: ctx.addFileButton,
    addWorkspaceFile: ctx.addWorkspaceFile,
    archiveFileInput: ctx.archiveFileInput,
    archiveImportButton: ctx.archiveImportButton,
    archiveUrlImportButton: ctx.archiveUrlImportButton,
    archiveUrlInput: ctx.archiveUrlInput,
    bootUrlLoadButton: ctx.bootUrlLoadButton,
    cancelButton: ctx.cancelButton,
    cancelRun: ctx.cancelRun,
    clearAllCaches: () => ctx.cacheController.clearAllCaches(),
    clearAllCachesButton: ctx.clearAllCachesButton,
    clearModuleCache: () => ctx.cacheController.clearModuleCache(),
    clearModuleCacheButton: ctx.clearModuleCacheButton,
    clearWorkspaceCache: () => ctx.cacheController.clearWorkspaceCache(),
    clearWorkspaceCacheButton: ctx.clearWorkspaceCacheButton,
    entryPathInput: ctx.entryPathInput,
    exampleSelect: ctx.exampleSelect,
    exportSnapshot: () => ctx.snapshotController.exportSnapshot(),
    fileList: ctx.fileList,
    filePathInput: ctx.filePathInput,
    formatButton: ctx.formatButton,
    handleExampleSelectionChange: (value) => ctx.exampleController.handleExampleSelectionChange(value),
    handleFileListChange: ctx.handleFileListChange,
    handleLoadModules: ctx.handleLoadModules,
    handleModuleRootsInput: ctx.handleModuleRootsInput,
    handleWorkspaceTargetInput: ctx.handleWorkspaceTargetInput,
    importArchiveFromFile: ctx.importArchiveFromFile,
    importArchiveFromUrl: ctx.importArchiveFromUrl,
    importSnapshotFromFile: (file) => ctx.snapshotController.importSnapshotFromFile(file),
    isBusy: ctx.isBusy,
    lintButton: ctx.lintButton,
    loadBootManifest: () => ctx.bootController.loadBootManifest("manual"),
    loadExampleButton: ctx.loadExampleButton,
    loadModulesButton: ctx.loadModulesButton,
    loadSelectedExample: () => ctx.exampleController.loadSelectedExample(),
    moduleRootsInput: ctx.moduleRootsInput,
    packageTargetInput: ctx.packageTargetInput,
    refreshCacheButton: ctx.refreshCacheButton,
    refreshCacheStatus: () => ctx.cacheController.refreshCacheStatus(),
    removeFileButton: ctx.removeFileButton,
    removeSelectedWorkspaceFile: ctx.removeSelectedWorkspaceFile,
    renameFileButton: ctx.renameFileButton,
    renamePathInput: ctx.renamePathInput,
    renameSelectedWorkspaceFile: ctx.renameSelectedWorkspaceFile,
    requestFormat: ctx.requestFormat,
    requestLint: ctx.requestLint,
    requestPackageTest: ctx.requestPackageTest,
    requestRun: ctx.requestRun,
    requestSnippetTest: ctx.requestSnippetTest,
    restoreCachedWorkspace: () => ctx.cacheController.restoreCachedWorkspace(),
    restoreCachedWorkspaceButton: ctx.restoreCachedWorkspaceButton,
    runButton: ctx.runButton,
    setSelectedAsEntryPath: ctx.setSelectedAsEntryPath,
    setSelectedAsPackageTarget: ctx.setSelectedAsPackageTarget,
    snapshotExportButton: ctx.snapshotExportButton,
    snapshotFileInput: ctx.snapshotFileInput,
    snapshotImportButton: ctx.snapshotImportButton,
    testPackageButton: ctx.testPackageButton,
    testSnippetButton: ctx.testSnippetButton,
    useSelectedEntryButton: ctx.useSelectedEntryButton,
    useSelectedPackageButton: ctx.useSelectedPackageButton,
  });
}
