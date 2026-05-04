import { createBrowserShellCacheController } from "./browser-shell-cache.js";
import { createBootUrlController } from "./browser-shell-boot.js";
import {
  applyRestoredSnapshotToShell,
  createSnapshotController,
} from "./browser-shell-snapshot.js";

export function createBrowserShellControllers(ctx) {
  let cacheController;

  const bootController = createBootUrlController({
    applyBootManifest: (bootManifest) => {
      ctx.setWorkspaceFiles(bootManifest.files, bootManifest.entryPath, {
        resetDirtyBaseline: true,
      });
      ctx.setSelectedExampleId?.("");
      ctx.entryPathInput.value = bootManifest.entryPath || "";
      ctx.packageTargetInput.value = bootManifest.packageTarget || "";
      ctx.moduleRootsInput.value = bootManifest.moduleRoots
        .map((module) => `${module.module_path} ${module.version} ${module.fetch_url}`)
        .join("\n");
      ctx.resetLoadedModules();
    },
    bootUrlPanel: ctx.bootUrlPanel,
    bootUrlStatus: ctx.bootUrlStatus,
    isBusy: ctx.isBusy,
    isWorkerReady: ctx.isWorkerReady,
    onBootManifestLoaded: async (bootManifest) => {
      await cacheController.rememberImportedWorkspace({
        sourceKind: "boot_manifest",
        sourceLabel: bootManifest.sourceLabel || "boot manifest",
      });
    },
    renderModuleStatus: ctx.renderModuleStatus,
    renderWorkspace: ctx.renderWorkspace,
    requestPackageTest: ctx.requestPackageTest,
    requestRun: ctx.requestRun,
    requestSnippetTest: ctx.requestSnippetTest,
    search: ctx.search,
    setOutputView: ctx.setOutputView,
    statusElement: ctx.statusElement,
    syncControls: ctx.syncControls,
  });

  const snapshotRestoreContext = {
    entryPathInput: ctx.entryPathInput,
    moduleRootsInput: ctx.moduleRootsInput,
    packageTargetInput: ctx.packageTargetInput,
    renderModuleStatus: ctx.renderModuleStatus,
    renderWorkspace: ctx.renderWorkspace,
    resetLoadedModules: ctx.resetLoadedModules,
    setSelectedExampleId: ctx.setSelectedExampleId,
    setOutputView: ctx.setOutputView,
    setWorkspaceFiles: ctx.setWorkspaceFiles,
    statusElement: ctx.statusElement,
    syncControls: ctx.syncControls,
  };

  const snapshotController = createSnapshotController({
    applyRestoredSnapshot: (restored, options) => {
      applyRestoredSnapshotToShell(snapshotRestoreContext, restored, options);
    },
    downloadSnapshotBlob: ctx.downloadSnapshotBlob,
    entryPathInput: ctx.entryPathInput,
    getLoadedModuleBundles: ctx.getLoadedModuleBundles,
    getSelectedFilePath: ctx.getSelectedFilePath,
    getWorkspaceFiles: ctx.getWorkspaceFiles,
    isBusy: ctx.isBusy,
    moduleRootsInput: ctx.moduleRootsInput,
    onImportedWorkspace: async (cacheSource) => {
      await cacheController.rememberImportedWorkspace(cacheSource);
    },
    packageTargetInput: ctx.packageTargetInput,
    renderModuleStatus: ctx.renderModuleStatus,
    renderWorkspace: ctx.renderWorkspace,
    resetLoadedModules: ctx.resetLoadedModules,
    setOutputView: ctx.setOutputView,
    setWorkspaceFiles: ctx.setWorkspaceFiles,
    statusElement: ctx.statusElement,
    syncControls: ctx.syncControls,
  });

  cacheController = createBrowserShellCacheController({
    applyRestoredSnapshot: (restored, options) => {
      applyRestoredSnapshotToShell(snapshotRestoreContext, restored, options);
    },
    cacheStatusElement: ctx.cacheStatusElement,
    entryPathInput: ctx.entryPathInput,
    getLoadedModuleBundles: ctx.getLoadedModuleBundles,
    getSelectedFilePath: ctx.getSelectedFilePath,
    getWorkspaceFiles: ctx.getWorkspaceFiles,
    isBusy: ctx.isBusy,
    moduleRootsInput: ctx.moduleRootsInput,
    packageTargetInput: ctx.packageTargetInput,
    setOutputView: ctx.setOutputView,
    statusElement: ctx.statusElement,
    syncControls: ctx.syncCacheControls,
  });

  return {
    bootController,
    cacheController,
    snapshotController,
  };
}
