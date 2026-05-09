import {
  MODULE_CACHE_DB_NAME,
  MODULE_CACHE_DB_VERSION,
  MODULE_CACHE_SCHEMA_VERSION,
  MODULE_CACHE_STORE_NAME,
} from "./engine-worker-module-cache.js";
import { isSafeModuleFileRecord } from "./browser-capability-security.js";
import {
  createWorkspaceSnapshot,
  formatWorkspaceSnapshotSummary,
  parseWorkspaceSnapshot,
} from "./browser-shell-snapshot.js";

export const BROWSER_SHELL_CACHE_DB_NAME = "gowasm-browser-shell-cache";
export const BROWSER_SHELL_CACHE_DB_VERSION = 1;
export const BROWSER_SHELL_CACHE_SCHEMA_VERSION = 1;
export const BROWSER_SHELL_WORKSPACE_STORE_NAME = "workspace_imports";
export const BROWSER_SHELL_EXAMPLE_STORE_NAME = "example_projects";
export const BROWSER_SHELL_METADATA_STORE_NAME = "metadata";
export const LATEST_IMPORTED_WORKSPACE_CACHE_KEY = "latest-imported-workspace";

const META_LAST_ALL_CLEAR_AT = "last_all_clear_at_ms";
const META_LAST_MODULE_CLEAR_AT = "last_module_clear_at_ms";
const META_LAST_WORKSPACE_CLEAR_AT = "last_workspace_clear_at_ms";

let browserShellCacheDbPromise = null;

export function createBrowserShellCacheController(ctx) {
  let pending = false;

  function isPending() {
    return pending;
  }

  async function refreshCacheStatus() {
    try {
      const summary = await collectCacheSummary();
      ctx.cacheStatusElement.textContent = formatCacheSummary(summary);
    } catch (error) {
      ctx.cacheStatusElement.textContent = `Browser cache status unavailable.\n${error?.message || String(error)}`;
    }
  }

  async function rememberImportedWorkspace(details = {}) {
    try {
      const snapshot = createWorkspaceSnapshot({
        entryPath: ctx.entryPathInput.value,
        loadedModuleBundles: ctx.getLoadedModuleBundles(),
        moduleRootsText: ctx.moduleRootsInput.value,
        packagedExampleIds: ctx.getPackagedExampleIds?.() ?? [],
        packageTarget: ctx.packageTargetInput.value,
        selectedExampleId: ctx.getSelectedExampleId?.() ?? "",
        selectedFilePath: ctx.getSelectedFilePath(),
        workspaceFiles: ctx.getWorkspaceFiles(),
      });
      await writeWorkspaceCacheRecord({
        cache_key: LATEST_IMPORTED_WORKSPACE_CACHE_KEY,
        schema_version: BROWSER_SHELL_CACHE_SCHEMA_VERSION,
        stored_at_ms: Date.now(),
        source_kind: String(details.sourceKind ?? "workspace_import"),
        source_label: String(details.sourceLabel ?? "").trim(),
        snapshot,
      });
      await refreshCacheStatus();
    } catch {
      // Cache persistence is best-effort only.
    }
  }

  async function restoreCachedWorkspace() {
    if (ctx.isBusy() || pending) {
      return;
    }
    await withPending(async () => {
      const record = await readWorkspaceCacheRecord(LATEST_IMPORTED_WORKSPACE_CACHE_KEY);
      if (!record) {
        ctx.statusElement.textContent = "No cached workspace";
        ctx.setOutputView("No imported workspace is cached yet.", []);
        await refreshCacheStatus();
        return;
      }

      const restored = restoreSnapshotFromCacheRecord(record);
      if (!restored) {
        ctx.statusElement.textContent = "Cached workspace is stale";
        ctx.setOutputView(
          "The latest imported workspace cache entry is stale or invalid. Clear the cache or re-import the workspace.",
          [],
        );
        await refreshCacheStatus();
        return;
      }

      ctx.applyRestoredSnapshot(restored, {
        outputText:
          `${formatWorkspaceSnapshotSummary(restored.snapshot)}\n\n` +
          `Restored from browser cache (${cacheSourceLabel(record)}).`,
        statusText: "Restored cached workspace",
      });
      await refreshCacheStatus();
    });
  }

  async function clearWorkspaceCache() {
    if (ctx.isBusy() || pending) {
      return;
    }
    await withPending(async () => {
      await clearBrowserShellStore(BROWSER_SHELL_WORKSPACE_STORE_NAME);
      await writeMetadataRecord(META_LAST_WORKSPACE_CLEAR_AT, Date.now());
      ctx.statusElement.textContent = "Workspace cache cleared";
      ctx.setOutputView("Imported workspace cache cleared.", []);
      await refreshCacheStatus();
    });
  }

  async function clearModuleCache() {
    if (ctx.isBusy() || pending) {
      return;
    }
    await withPending(async () => {
      await clearModuleCacheStore();
      await writeMetadataRecord(META_LAST_MODULE_CLEAR_AT, Date.now());
      ctx.statusElement.textContent = "Module cache cleared";
      ctx.setOutputView("Cached remote module bundles cleared.", []);
      await refreshCacheStatus();
    });
  }

  async function clearAllCaches() {
    if (ctx.isBusy() || pending) {
      return;
    }
    await withPending(async () => {
      await clearBrowserShellStore(BROWSER_SHELL_WORKSPACE_STORE_NAME);
      await clearBrowserShellStore(BROWSER_SHELL_EXAMPLE_STORE_NAME);
      await clearBrowserShellStore(BROWSER_SHELL_METADATA_STORE_NAME);
      await clearModuleCacheStore();
      await writeMetadataRecord(META_LAST_ALL_CLEAR_AT, Date.now());
      ctx.statusElement.textContent = "Browser caches cleared";
      ctx.setOutputView("Imported workspaces, packaged example cache entries, and module bundles cleared.", []);
      await refreshCacheStatus();
    });
  }

  async function seedPackagedExamples(records) {
    if (!Array.isArray(records) || records.length === 0) {
      await refreshCacheStatus();
      return;
    }
    try {
      await withBrowserShellStore(BROWSER_SHELL_EXAMPLE_STORE_NAME, "readwrite", (store) => {
        for (const record of records) {
          store.put(record);
        }
        return null;
      });
      await refreshCacheStatus();
    } catch {
      // Example seeding is best-effort only.
    }
  }

  async function withPending(operation) {
    pending = true;
    ctx.syncControls();
    try {
      await operation();
    } finally {
      pending = false;
      ctx.syncControls();
    }
  }

  return {
    cacheSchemaVersion: BROWSER_SHELL_CACHE_SCHEMA_VERSION,
    clearAllCaches,
    clearModuleCache,
    clearWorkspaceCache,
    isPending,
    refreshCacheStatus,
    rememberImportedWorkspace,
    restoreCachedWorkspace,
    seedPackagedExamples,
  };
}

function restoreSnapshotFromCacheRecord(record) {
  if (!isValidWorkspaceCacheRecord(record)) {
    return null;
  }
  try {
    return parseWorkspaceSnapshot(JSON.stringify(record.snapshot));
  } catch {
    return null;
  }
}

function isValidWorkspaceCacheRecord(record) {
  return (
    typeof record?.cache_key === "string" &&
    record.cache_key.length > 0 &&
    record.schema_version === BROWSER_SHELL_CACHE_SCHEMA_VERSION &&
    Number.isFinite(record.stored_at_ms) &&
    typeof record.source_kind === "string" &&
    typeof record.source_label === "string" &&
    record.snapshot &&
    typeof record.snapshot === "object" &&
    !Array.isArray(record.snapshot)
  );
}

function isValidExampleCacheRecord(record) {
  return (
    typeof record?.cache_key === "string" &&
    record.cache_key.length > 0 &&
    record.schema_version === BROWSER_SHELL_CACHE_SCHEMA_VERSION &&
    Number.isFinite(record.stored_at_ms) &&
    typeof record.example_id === "string" &&
    record.example_id.length > 0 &&
    record.snapshot &&
    typeof record.snapshot === "object" &&
    !Array.isArray(record.snapshot)
  );
}

function isValidModuleCacheRecord(record) {
  return (
    typeof record?.cache_key === "string" &&
    record.cache_key === `${record?.module?.module_path}@${record?.module?.version}` &&
    record.schema_version === MODULE_CACHE_SCHEMA_VERSION &&
    Number.isFinite(record.stored_at_ms) &&
    typeof record?.origin_url === "string" &&
    record.origin_url.length > 0 &&
    typeof record?.module?.module_path === "string" &&
    record.module.module_path.length > 0 &&
    typeof record?.module?.version === "string" &&
    record.module.version.length > 0 &&
    Array.isArray(record.files) &&
    record.files.every(isSafeModuleFileRecord)
  );
}

async function collectCacheSummary() {
  const [workspaceRecords, exampleRecords, metadataRecords, moduleRecords, storageEstimate] = await Promise.all([
    readBrowserShellStoreRecords(BROWSER_SHELL_WORKSPACE_STORE_NAME),
    readBrowserShellStoreRecords(BROWSER_SHELL_EXAMPLE_STORE_NAME),
    readBrowserShellStoreRecords(BROWSER_SHELL_METADATA_STORE_NAME),
    readModuleCacheRecords(),
    readStorageEstimate(),
  ]);

  const workspaceValidRecords = workspaceRecords.filter(isValidWorkspaceCacheRecord);
  const exampleValidRecords = exampleRecords.filter(isValidExampleCacheRecord);
  const moduleValidRecords = moduleRecords.filter(isValidModuleCacheRecord);
  const metadata = new Map(
    metadataRecords
      .filter((record) => typeof record?.key === "string")
      .map((record) => [record.key, record.value]),
  );

  return {
    exampleStaleCount: exampleRecords.length - exampleValidRecords.length,
    exampleValidCount: exampleValidRecords.length,
    lastAllClearAtMs: finiteMetadataValue(metadata.get(META_LAST_ALL_CLEAR_AT)),
    lastModuleClearAtMs: finiteMetadataValue(metadata.get(META_LAST_MODULE_CLEAR_AT)),
    lastWorkspaceClearAtMs: finiteMetadataValue(metadata.get(META_LAST_WORKSPACE_CLEAR_AT)),
    latestWorkspaceRecord: workspaceValidRecords.find(
      (record) => record.cache_key === LATEST_IMPORTED_WORKSPACE_CACHE_KEY,
    ) || null,
    moduleStaleCount: moduleRecords.length - moduleValidRecords.length,
    moduleValidCount: moduleValidRecords.length,
    storageEstimate,
    workspaceStaleCount: workspaceRecords.length - workspaceValidRecords.length,
    workspaceValidCount: workspaceValidRecords.length,
  };
}

function formatCacheSummary(summary) {
  const lines = [
    `Imported workspace cache: ${countLabel(summary.workspaceValidCount)} valid, ${countLabel(summary.workspaceStaleCount)} stale.`,
    `Example project cache: ${countLabel(summary.exampleValidCount)} valid, ${countLabel(summary.exampleStaleCount)} stale.`,
    `Module bundle cache: ${countLabel(summary.moduleValidCount)} valid, ${countLabel(summary.moduleStaleCount)} stale.`,
  ];

  if (summary.latestWorkspaceRecord) {
    lines.push(
      `Latest cached workspace: ${cacheSourceLabel(summary.latestWorkspaceRecord)} at ${new Date(summary.latestWorkspaceRecord.stored_at_ms).toLocaleString()}.`,
    );
  } else {
    lines.push("Latest cached workspace: none yet.");
  }

  const storageEstimateLine = formatStorageEstimate(summary.storageEstimate);
  if (storageEstimateLine) {
    lines.push(storageEstimateLine);
  }

  const clearSummary = formatClearSummary(summary);
  if (clearSummary) {
    lines.push(clearSummary);
  }

  const warnings = [];
  if (summary.workspaceStaleCount > 0) {
    warnings.push("Imported workspace cache contains stale entries; clear it or re-import the workspace.");
  }
  if (summary.exampleStaleCount > 0) {
    warnings.push("Example project cache contains stale packaged examples; clear browser caches before reloading the example catalog.");
  }
  if (summary.moduleStaleCount > 0) {
    warnings.push("Module bundle cache contains stale entries; reload modules or clear the module cache.");
  }
  if (warnings.length > 0) {
    lines.push("");
    lines.push("Warnings:");
    for (const warning of warnings) {
      lines.push(`- ${warning}`);
    }
  }

  return lines.join("\n");
}

function formatStorageEstimate(estimate) {
  if (!Number.isFinite(estimate?.usage) || !Number.isFinite(estimate?.quota)) {
    return "";
  }
  return `Browser storage estimate: ${formatBytes(estimate.usage)} used of ${formatBytes(estimate.quota)} quota.`;
}

function formatClearSummary(summary) {
  const parts = [];
  if (summary.lastWorkspaceClearAtMs) {
    parts.push(`workspace cache cleared ${new Date(summary.lastWorkspaceClearAtMs).toLocaleString()}`);
  }
  if (summary.lastModuleClearAtMs) {
    parts.push(`module cache cleared ${new Date(summary.lastModuleClearAtMs).toLocaleString()}`);
  }
  if (summary.lastAllClearAtMs) {
    parts.push(`all caches cleared ${new Date(summary.lastAllClearAtMs).toLocaleString()}`);
  }
  if (parts.length === 0) {
    return "";
  }
  return `Recent clear actions: ${parts.join("; ")}.`;
}

function cacheSourceLabel(record) {
  const sourceKind = String(record?.source_kind ?? "workspace_import").replaceAll("_", " ");
  const sourceLabel = String(record?.source_label ?? "").trim();
  if (!sourceLabel) {
    return sourceKind;
  }
  return `${sourceKind} (${sourceLabel})`;
}

function countLabel(count) {
  return `${count} ${count === 1 ? "entry" : "entries"}`;
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return "unknown";
  }
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = -1;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[unitIndex]}`;
}

function finiteMetadataValue(value) {
  return Number.isFinite(value) ? value : 0;
}

async function readStorageEstimate() {
  if (typeof navigator?.storage?.estimate !== "function") {
    return null;
  }
  try {
    return await navigator.storage.estimate();
  } catch {
    return null;
  }
}

async function readWorkspaceCacheRecord(cacheKey) {
  return await withBrowserShellStore(BROWSER_SHELL_WORKSPACE_STORE_NAME, "readonly", (store) => store.get(cacheKey));
}

async function writeWorkspaceCacheRecord(record) {
  await withBrowserShellStore(BROWSER_SHELL_WORKSPACE_STORE_NAME, "readwrite", (store) => store.put(record));
}

async function readBrowserShellStoreRecords(storeName) {
  const records = await withBrowserShellStore(storeName, "readonly", (store) => store.getAll());
  return Array.isArray(records) ? records : [];
}

async function clearBrowserShellStore(storeName) {
  await withBrowserShellStore(storeName, "readwrite", (store) => store.clear());
}

async function writeMetadataRecord(key, value) {
  await withBrowserShellStore(BROWSER_SHELL_METADATA_STORE_NAME, "readwrite", (store) => store.put({ key, value }));
}

async function withBrowserShellStore(storeName, mode, operation) {
  const db = await openBrowserShellCacheDb();
  return await transactionResult(db, storeName, mode, operation);
}

async function openBrowserShellCacheDb() {
  if (typeof indexedDB === "undefined") {
    throw new Error("IndexedDB is unavailable in this browser.");
  }
  if (!browserShellCacheDbPromise) {
    browserShellCacheDbPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(BROWSER_SHELL_CACHE_DB_NAME, BROWSER_SHELL_CACHE_DB_VERSION);
      request.onupgradeneeded = () => {
        const db = request.result;
        ensureObjectStore(db, BROWSER_SHELL_WORKSPACE_STORE_NAME, { keyPath: "cache_key" });
        ensureObjectStore(db, BROWSER_SHELL_EXAMPLE_STORE_NAME, { keyPath: "cache_key" });
        ensureObjectStore(db, BROWSER_SHELL_METADATA_STORE_NAME, { keyPath: "key" });
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => {
        reject(request.error ?? new Error("failed to open browser shell cache database"));
      };
      request.onblocked = () => {
        reject(new Error("browser shell cache database open was blocked"));
      };
    }).catch((error) => {
      browserShellCacheDbPromise = null;
      throw error;
    });
  }
  return browserShellCacheDbPromise;
}

function ensureObjectStore(db, storeName, options) {
  if (!db.objectStoreNames.contains(storeName)) {
    db.createObjectStore(storeName, options);
  }
}

async function readModuleCacheRecords() {
  if (typeof indexedDB === "undefined") {
    return [];
  }
  let db = null;
  try {
    db = await openModuleCacheDb();
    const records = await transactionResult(db, MODULE_CACHE_STORE_NAME, "readonly", (store) => store.getAll());
    return Array.isArray(records) ? records : [];
  } catch {
    return [];
  } finally {
    db?.close();
  }
}

async function clearModuleCacheStore() {
  if (typeof indexedDB === "undefined") {
    return;
  }
  let db = null;
  try {
    db = await openModuleCacheDb();
    await transactionResult(db, MODULE_CACHE_STORE_NAME, "readwrite", (store) => store.clear());
  } finally {
    db?.close();
  }
}

async function openModuleCacheDb() {
  return await new Promise((resolve, reject) => {
    const request = indexedDB.open(MODULE_CACHE_DB_NAME, MODULE_CACHE_DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      ensureObjectStore(db, MODULE_CACHE_STORE_NAME, { keyPath: "cache_key" });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => {
      reject(request.error ?? new Error("failed to open module cache database"));
    };
    request.onblocked = () => {
      reject(new Error("module cache database open was blocked"));
    };
  });
}

async function transactionResult(db, storeName, mode, operation) {
  return await new Promise((resolve, reject) => {
    const transaction = db.transaction(storeName, mode);
    const store = transaction.objectStore(storeName);
    let request;
    try {
      request = operation(store);
    } catch (error) {
      reject(error);
      return;
    }
    transaction.oncomplete = () => resolve(request?.result ?? null);
    transaction.onerror = () => {
      reject(transaction.error ?? new Error(`${storeName} transaction failed`));
    };
    transaction.onabort = () => {
      reject(transaction.error ?? new Error(`${storeName} transaction aborted`));
    };
  });
}
