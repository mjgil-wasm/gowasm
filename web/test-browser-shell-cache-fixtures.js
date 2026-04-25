import {
  BROWSER_SHELL_CACHE_DB_NAME,
  BROWSER_SHELL_CACHE_DB_VERSION,
  BROWSER_SHELL_EXAMPLE_STORE_NAME,
  BROWSER_SHELL_METADATA_STORE_NAME,
  BROWSER_SHELL_WORKSPACE_STORE_NAME,
} from "./browser-shell-cache.js";
import {
  MODULE_CACHE_DB_NAME,
  MODULE_CACHE_DB_VERSION,
  MODULE_CACHE_STORE_NAME,
} from "./engine-worker-module-cache.js";

export async function resetBrowserShellCacheDatabase() {
  await deleteDatabase(BROWSER_SHELL_CACHE_DB_NAME);
}

export async function seedBrowserShellWorkspaceCacheRecord(record) {
  await seedIndexedDbRecord(
    BROWSER_SHELL_CACHE_DB_NAME,
    BROWSER_SHELL_CACHE_DB_VERSION,
    BROWSER_SHELL_WORKSPACE_STORE_NAME,
    { keyPath: "cache_key" },
    record,
  );
}

export async function seedBrowserShellExampleCacheRecord(record) {
  await seedIndexedDbRecord(
    BROWSER_SHELL_CACHE_DB_NAME,
    BROWSER_SHELL_CACHE_DB_VERSION,
    BROWSER_SHELL_EXAMPLE_STORE_NAME,
    { keyPath: "cache_key" },
    record,
  );
}

export async function seedBrowserShellMetadataRecord(record) {
  await seedIndexedDbRecord(
    BROWSER_SHELL_CACHE_DB_NAME,
    BROWSER_SHELL_CACHE_DB_VERSION,
    BROWSER_SHELL_METADATA_STORE_NAME,
    { keyPath: "key" },
    record,
  );
}

export async function seedModuleCacheRecord(record) {
  await seedIndexedDbRecord(
    MODULE_CACHE_DB_NAME,
    MODULE_CACHE_DB_VERSION,
    MODULE_CACHE_STORE_NAME,
    { keyPath: "cache_key" },
    record,
  );
}

async function deleteDatabase(name) {
  await new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(name);
    request.onsuccess = () => resolve();
    request.onerror = () => {
      reject(request.error ?? new Error(`failed to delete ${name}`));
    };
    request.onblocked = () => {
      reject(new Error(`${name} deletion was blocked`));
    };
  });
}

async function seedIndexedDbRecord(dbName, dbVersion, storeName, storeOptions, record) {
  const db = await openIndexedDb(dbName, dbVersion, storeName, storeOptions);
  try {
    await new Promise((resolve, reject) => {
      const transaction = db.transaction(storeName, "readwrite");
      transaction.oncomplete = () => resolve();
      transaction.onerror = () => {
        reject(transaction.error ?? new Error(`failed to seed ${storeName}`));
      };
      transaction.onabort = () => {
        reject(transaction.error ?? new Error(`${storeName} seed transaction aborted`));
      };
      transaction.objectStore(storeName).put(record);
    });
  } finally {
    db.close();
  }
}

async function openIndexedDb(dbName, dbVersion, storeName, storeOptions) {
  return await new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, dbVersion);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(storeName)) {
        db.createObjectStore(storeName, storeOptions);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => {
      reject(request.error ?? new Error(`failed to open ${dbName}`));
    };
    request.onblocked = () => {
      reject(new Error(`${dbName} open was blocked`));
    };
  });
}
