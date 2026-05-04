import {
  BROWSER_SHELL_CACHE_SCHEMA_VERSION,
  LATEST_IMPORTED_WORKSPACE_CACHE_KEY,
} from "./browser-shell-cache.js";
import { MODULE_CACHE_SCHEMA_VERSION } from "./engine-worker-module-cache.js";
import { createArchiveDataUrl } from "./test-browser-archive-fixtures.js";
import {
  resetBrowserShellCacheDatabase,
  seedBrowserShellWorkspaceCacheRecord,
  seedModuleCacheRecord,
} from "./test-browser-shell-cache-fixtures.js";
import {
  click,
  control,
  loadShellFrame,
  selectedSource,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
  setWorkspaceFileContents,
} from "./test-browser-shell-harness.js";
import { resetModuleCacheDatabase } from "./test-worker-modules.js";

export async function testBrowserShellCaches({ assert, frame, log }) {
  log("\n--- browser shell cache harness ---");

  await resetBrowserShellCacheDatabase();
  await resetModuleCacheDatabase();

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);
    await waitForCacheStatus(doc, (text) => text.includes("Imported workspace cache: 0 entries valid"));
    assert(
      control(doc, "cache-status").textContent.includes("Module bundle cache: 0 entries valid"),
      "browser shell cache panel reports an initial miss for workspace and module caches",
      shellSnapshot(doc),
    );

    control(doc, "archive-url-input").value = createArchiveDataUrl([
      {
        path: "cache-root/go.mod",
        contents: "module example.com/cache\n\ngo 1.21\n",
      },
      {
        path: "cache-root/main.go",
        contents: `package main

import "fmt"

func main() {
\tfmt.Println("cache-hit")
}
`,
      },
    ]);
    click(doc, "archive-url-import-button");
    await waitFor(
      () =>
        control(doc, "status").textContent.startsWith("Imported")
          && control(doc, "cache-status").textContent.includes("Imported workspace cache: 1 entry valid"),
      "workspace import cache population",
      doc,
    );
    assert(
      control(doc, "cache-status").textContent.includes("Latest cached workspace: archive url"),
      "browser shell cache panel records the latest imported workspace source",
      shellSnapshot(doc),
    );

    await setWorkspaceFileContents(
      doc,
      "main.go",
      `package main

func main() {}
`,
    );
    click(doc, "restore-cached-workspace-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Restored cached workspace"
          && selectedSource(doc).includes("cache-hit"),
      "cached workspace restore completed",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("Restored from browser cache")
        && selectedSource(doc).includes("cache-hit"),
      "browser shell cache restore rehydrates the latest imported workspace on a cache hit",
      shellSnapshot(doc),
    );

    await seedBrowserShellWorkspaceCacheRecord({
      cache_key: LATEST_IMPORTED_WORKSPACE_CACHE_KEY,
      schema_version: BROWSER_SHELL_CACHE_SCHEMA_VERSION - 1,
      stored_at_ms: Date.now(),
      source_kind: "archive_url",
      source_label: "stale-cache",
      snapshot: {
        version: 1,
      },
    });
    click(doc, "refresh-cache-button");
    await waitForCacheStatus(doc, (text) => text.includes("Imported workspace cache: 0 entries valid, 1 entry stale"));
    assert(
      control(doc, "cache-status").textContent.includes("Warnings:")
        && control(doc, "cache-status").textContent.includes("workspace cache contains stale entries"),
      "browser shell cache panel surfaces stale workspace warnings",
      shellSnapshot(doc),
    );

    click(doc, "restore-cached-workspace-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Cached workspace is stale"
          && control(doc, "output").textContent.includes("stale or invalid"),
      "stale cache restore rejected",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("stale or invalid"),
      "browser shell cache restore rejects stale cached workspaces",
      shellSnapshot(doc),
    );

    await seedModuleCacheRecord({
      cache_key: "example.com/cache/module@v1.2.3",
      schema_version: MODULE_CACHE_SCHEMA_VERSION,
      stored_at_ms: Date.now(),
      module: {
        module_path: "example.com/cache/module",
        version: "v1.2.3",
      },
      origin_url: "https://example.invalid/module.json",
      files: [
        {
          path: "go.mod",
          contents: "module example.com/cache/module\n\ngo 1.21\n",
        },
      ],
    });
    click(doc, "refresh-cache-button");
    await waitForCacheStatus(doc, (text) => text.includes("Module bundle cache: 1 entry valid"));
    assert(
      control(doc, "cache-status").textContent.includes("Module bundle cache: 1 entry valid"),
      "browser shell cache panel reports module cache hits",
      shellSnapshot(doc),
    );

    click(doc, "clear-module-cache-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Module cache cleared"
          && control(doc, "cache-status").textContent.includes("Module bundle cache: 0 entries valid"),
      "module cache clear completed",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("Cached remote module bundles cleared."),
      "browser shell cache clear action empties the module cache surface",
      shellSnapshot(doc),
    );

    click(doc, "clear-all-caches-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Browser caches cleared"
          && control(doc, "cache-status").textContent.includes("Imported workspace cache: 0 entries valid"),
      "clear-all caches completed",
      doc,
    );
    click(doc, "restore-cached-workspace-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "No cached workspace"
          && control(doc, "output").textContent.includes("No imported workspace is cached yet."),
      "restore after clear-all completed",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("No imported workspace is cached yet."),
      "browser shell cache miss is explicit after clearing caches",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
    await resetBrowserShellCacheDatabase();
    await resetModuleCacheDatabase();
  }
}

async function waitForCacheStatus(doc, predicate) {
  await waitFor(
    () => predicate(control(doc, "cache-status").textContent),
    "browser cache status update",
    doc,
  );
}
