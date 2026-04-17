import { isSafeModuleFileRecord } from "./browser-capability-security.js";

export const MODULE_CACHE_DB_NAME = "gowasm-module-cache";
export const MODULE_CACHE_DB_VERSION = 1;
export const MODULE_CACHE_STORE_NAME = "modules";
export const MODULE_CACHE_SCHEMA_VERSION = 1;

const hotModuleCache = new Map();

let moduleCacheDbPromise = null;

export async function lookupModuleCache(request) {
  const key = moduleCacheKey(request.module);
  const hotBundle = hydrateModuleBundle(hotModuleCache.get(key), request.module);
  if (hotBundle) {
    return {
      kind: "hit",
      module: hotBundle,
    };
  }
  hotModuleCache.delete(key);

  let persistedRecord = null;
  try {
    persistedRecord = await readPersistedModuleRecord(key);
  } catch {
    return {
      kind: "miss",
    };
  }
  const persistedBundle = hydrateModuleBundle(persistedRecord, request.module);
  if (!persistedBundle) {
    if (persistedRecord) {
      try {
        await deletePersistedModuleRecord(key);
      } catch {
        // Treat invalid-cache cleanup as best-effort only.
      }
    }
    return {
      kind: "miss",
    };
  }

  hotModuleCache.set(key, persistedRecord);
  return {
    kind: "hit",
    module: persistedBundle,
  };
}

export async function fillModuleCache(request) {
  const record = createModuleCacheRecord(request.module);
  hotModuleCache.set(record.cache_key, record);
  try {
    await writePersistedModuleRecord(record);
  } catch {
    // Persistence is an optimization over the hot in-worker cache.
  }
}

function createModuleCacheRecord(module) {
  return {
    cache_key: moduleCacheKey(module.module),
    schema_version: MODULE_CACHE_SCHEMA_VERSION,
    stored_at_ms: Date.now(),
    module: cloneModuleIdentity(module.module),
    origin_url: module.origin_url,
    files: cloneModuleFiles(module.files),
  };
}

function hydrateModuleBundle(record, expectedModule) {
  if (!record || !isValidModuleCacheRecord(record, expectedModule)) {
    return null;
  }
  return {
    module: cloneModuleIdentity(record.module),
    origin_url: record.origin_url,
    files: cloneModuleFiles(record.files),
  };
}

function isValidModuleCacheRecord(record, expectedModule) {
  return (
    typeof record?.cache_key === "string" &&
    record.cache_key === moduleCacheKey(expectedModule) &&
    record.schema_version === MODULE_CACHE_SCHEMA_VERSION &&
    Number.isFinite(record.stored_at_ms) &&
    matchesModuleIdentity(record.module, expectedModule) &&
    typeof record.origin_url === "string" &&
    record.origin_url.length > 0 &&
    Array.isArray(record.files) &&
    record.files.every(isValidModuleFile)
  );
}

function isValidModuleFile(file) {
  return isSafeModuleFileRecord(file);
}

function matchesModuleIdentity(actual, expected) {
  return (
    typeof actual?.module_path === "string" &&
    typeof actual?.version === "string" &&
    actual.module_path === expected.module_path &&
    actual.version === expected.version
  );
}

function cloneModuleIdentity(module) {
  return {
    module_path: module.module_path,
    version: module.version,
  };
}

function cloneModuleFiles(files) {
  return files.map((file) => ({
    path: file.path,
    contents: file.contents,
  }));
}

function moduleCacheKey(module) {
  return `${module.module_path}@${module.version}`;
}

async function readPersistedModuleRecord(cacheKey) {
  return withModuleCacheStore("readonly", (store) => store.get(cacheKey));
}

async function writePersistedModuleRecord(record) {
  await withModuleCacheStore("readwrite", (store) => store.put(record));
}

async function deletePersistedModuleRecord(cacheKey) {
  await withModuleCacheStore("readwrite", (store) => store.delete(cacheKey));
}

async function withModuleCacheStore(mode, operation) {
  const db = await openModuleCacheDb();
  if (!db) {
    return null;
  }

  return await new Promise((resolve, reject) => {
    const transaction = db.transaction(MODULE_CACHE_STORE_NAME, mode);
    const store = transaction.objectStore(MODULE_CACHE_STORE_NAME);
    let request;
    try {
      request = operation(store);
    } catch (error) {
      reject(error);
      return;
    }

    transaction.oncomplete = () => {
      resolve(request?.result ?? null);
    };
    transaction.onerror = () => {
      reject(transaction.error ?? new Error("module cache transaction failed"));
    };
    transaction.onabort = () => {
      reject(transaction.error ?? new Error("module cache transaction aborted"));
    };
  });
}

async function openModuleCacheDb() {
  if (typeof indexedDB === "undefined") {
    return null;
  }
  if (!moduleCacheDbPromise) {
    moduleCacheDbPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(MODULE_CACHE_DB_NAME, MODULE_CACHE_DB_VERSION);
      request.onupgradeneeded = () => {
        const db = request.result;
        let store;
        if (db.objectStoreNames.contains(MODULE_CACHE_STORE_NAME)) {
          store = request.transaction.objectStore(MODULE_CACHE_STORE_NAME);
          store.clear();
        } else {
          store = db.createObjectStore(MODULE_CACHE_STORE_NAME, {
            keyPath: "cache_key",
          });
        }
      };
      request.onsuccess = () => {
        resolve(request.result);
      };
      request.onerror = () => {
        reject(request.error ?? new Error("failed to open module cache database"));
      };
      request.onblocked = () => {
        reject(new Error("module cache database open was blocked"));
      };
    }).catch((error) => {
      moduleCacheDbPromise = null;
      throw error;
    });
  }
  return moduleCacheDbPromise;
}
