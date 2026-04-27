import {
  MODULE_CACHE_DB_NAME,
  MODULE_CACHE_DB_VERSION,
  MODULE_CACHE_SCHEMA_VERSION,
  MODULE_CACHE_STORE_NAME,
} from "./engine-worker-module-cache.js";
import {
  moduleBundlesToWorkspaceFiles,
  moduleCacheSourcePath,
} from "./browser-workspace.js";

export { moduleBundlesToWorkspaceFiles, moduleCacheSourcePath };

export async function testPersistentModuleGraphPath({
  assert,
  createWorker,
  log,
  sendAndWait,
}) {
  log("\n--- persistent module host protocol path ---");
  await resetModuleCacheDatabase();
  const modulePath = `example.com/hello/persistent-${Date.now()}`;
  const version = "v1.2.3";
  let firstResult = null;
  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const bundle = {
      module: {
        module_path: modulePath,
        version,
      },
      files: [
        {
          path: "go.mod",
          contents: `module ${modulePath}\n\ngo 1.21\n`,
        },
        {
          path: "hello/hello.go",
          contents: "package hello\n",
        },
      ],
    };
    const fetchUrl = `data:application/json,${encodeURIComponent(JSON.stringify(bundle))}`;

    firstResult = await sendAndWait(
      worker,
      moduleGraphRequest(modulePath, version, fetchUrl),
      15000,
    );
    assert(
      firstResult.kind === "module_graph_result",
      "module graph request produces module_graph_result",
      `got: ${JSON.stringify(firstResult)}`,
    );
    assert(
      firstResult.modules?.[0]?.module?.module_path === modulePath &&
        firstResult.modules?.[0]?.files?.length === 2,
      "module graph request returns normalized bundle contents",
      `got: ${JSON.stringify(firstResult)}`,
    );
  } finally {
    worker.terminate();
  }

  const cachedWorker = createWorker();
  try {
    await sendAndWait(cachedWorker, { kind: "boot" });
    const cachedResult = await sendAndWait(
      cachedWorker,
      moduleGraphRequest(modulePath, version, "module-cache-invalid://should-not-fetch"),
      15000,
    );
    assert(
      cachedResult.kind === "module_graph_result",
      "cached module graph request still produces module_graph_result",
      `got: ${JSON.stringify(cachedResult)}`,
    );
    assert(
      cachedResult.modules?.[0]?.origin_url === firstResult.modules?.[0]?.origin_url,
      "cached module graph request reuses the filled bundle instead of refetching",
      `got: ${JSON.stringify(cachedResult)}`,
    );
  } finally {
    cachedWorker.terminate();
    await resetModuleCacheDatabase();
  }
}

export async function testStalePersistentModuleEntryInvalidation({
  assert,
  createWorker,
  log,
  sendAndWait,
}) {
  log("\n--- stale persistent module cache invalidation ---");
  await resetModuleCacheDatabase();

  const modulePath = `example.com/hello/stale-${Date.now()}`;
  const version = "v9.9.9";
  const staleOriginUrl = "https://stale.invalid/module.json";

  await seedModuleCacheRecord({
    cache_key: moduleCacheKey(modulePath, version),
    schema_version: MODULE_CACHE_SCHEMA_VERSION - 1,
    stored_at_ms: Date.now(),
    module: {
      module_path: modulePath,
      version,
    },
    origin_url: staleOriginUrl,
    files: [
      {
        path: "go.mod",
        contents: `module ${modulePath}\n\ngo 1.21\n`,
      },
      {
        path: "hello/hello.go",
        contents: "package hello\nconst Source = \"stale\"\n",
      },
    ],
  });

  const freshBundle = {
    module: {
      module_path: modulePath,
      version,
    },
    files: [
      {
        path: "go.mod",
        contents: `module ${modulePath}\n\ngo 1.21\n`,
      },
      {
        path: "hello/hello.go",
        contents: "package hello\nconst Source = \"fresh\"\n",
      },
    ],
  };
  const fetchUrl = `data:application/json,${encodeURIComponent(JSON.stringify(freshBundle))}`;

  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const result = await sendAndWait(
      worker,
      moduleGraphRequest(modulePath, version, fetchUrl),
      15000,
    );
    assert(
      result.kind === "module_graph_result",
      "stale persistent entry still resolves through module_graph_result",
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      result.modules?.[0]?.origin_url !== staleOriginUrl,
      "stale persistent entry is invalidated before reuse",
      `got: ${JSON.stringify(result.modules?.[0])}`,
    );
    assert(
      result.modules?.[0]?.files?.[1]?.contents?.includes("fresh"),
      "stale persistent entry is replaced with freshly fetched contents",
      `got: ${JSON.stringify(result.modules?.[0]?.files)}`,
    );
  } finally {
    worker.terminate();
    await resetModuleCacheDatabase();
  }
}

export async function loadModuleGraphBundles({
  worker,
  sendAndWait,
  modules,
  timeoutMs = 15000,
}) {
  const result = await sendAndWait(
    worker,
    {
      kind: "load_module_graph",
      modules,
    },
    timeoutMs,
  );
  if (result?.kind !== "module_graph_result" || !Array.isArray(result.modules)) {
    throw new Error(
      `expected module_graph_result from load_module_graph, got ${JSON.stringify(result)}`,
    );
  }
  return result.modules;
}

function moduleGraphRequest(modulePath, version, fetchUrl) {
  return {
    kind: "load_module_graph",
    modules: [
      {
        module_path: modulePath,
        version,
        fetch_url: fetchUrl,
      },
    ],
  };
}

function moduleCacheKey(modulePath, version) {
  return `${modulePath}@${version}`;
}

export async function resetModuleCacheDatabase() {
  await new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(MODULE_CACHE_DB_NAME);
    request.onsuccess = () => resolve();
    request.onerror = () => {
      reject(request.error ?? new Error("failed to delete module cache database"));
    };
    request.onblocked = () => {
      reject(new Error("module cache database deletion was blocked"));
    };
  });
}

async function seedModuleCacheRecord(record) {
  const db = await new Promise((resolve, reject) => {
    const request = indexedDB.open(MODULE_CACHE_DB_NAME, MODULE_CACHE_DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(MODULE_CACHE_STORE_NAME)) {
        db.createObjectStore(MODULE_CACHE_STORE_NAME, {
          keyPath: "cache_key",
        });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => {
      reject(request.error ?? new Error("failed to open module cache database"));
    };
    request.onblocked = () => {
      reject(new Error("module cache database open was blocked"));
    };
  });

  try {
    await new Promise((resolve, reject) => {
      const transaction = db.transaction(MODULE_CACHE_STORE_NAME, "readwrite");
      transaction.oncomplete = () => resolve();
      transaction.onerror = () => {
        reject(transaction.error ?? new Error("failed to seed module cache record"));
      };
      transaction.onabort = () => {
        reject(transaction.error ?? new Error("module cache seed transaction aborted"));
      };
      transaction.objectStore(MODULE_CACHE_STORE_NAME).put(record);
    });
  } finally {
    db.close();
  }
}
