import {
  cloneWorkspaceFiles,
  findWorkspaceFile,
  moduleRootsConfigKey,
  normalizeWorkspacePath,
  parseModuleGraphRoots,
} from "./browser-workspace.js";

const SNAPSHOT_VERSION = 1;

export function createWorkspaceSnapshot(state) {
  const workspaceFiles = cloneWorkspaceFiles(state.workspaceFiles);
  const selectedFilePath = normalizeWorkspacePath(state.selectedFilePath ?? "");
  const entryPath = normalizeWorkspacePath(state.entryPath ?? "");
  const packageTarget = normalizeWorkspacePath(state.packageTarget ?? "");
  const packagedExampleIds = [...(state.packagedExampleIds ?? [])].map((value) => String(value));
  const selectedExampleId = String(state.selectedExampleId ?? "");
  const { modules, errors } = parseModuleGraphRoots(state.moduleRootsText ?? "");
  if (errors.length > 0) {
    throw new Error(`Snapshot export failed: module roots config is invalid.\n${errors.join("\n")}`);
  }

  return {
    version: SNAPSHOT_VERSION,
    workspace: {
      files: workspaceFiles,
      selected_file_path: selectedFilePath,
      entry_path: entryPath,
      package_target: packageTarget,
    },
    module_roots: modules,
    loaded_module_refs: normalizeLoadedModuleRefs(state.loadedModuleBundles),
    settings: {
      lint_scope: "editable_workspace",
      projected_module_cache_paths_reserved: true,
      run_uses_loaded_module_bundles: true,
    },
    examples: {
      packaged_examples: packagedExampleIds,
      selected_example_id: selectedExampleId,
    },
  };
}

export function serializeWorkspaceSnapshot(snapshot) {
  return `${JSON.stringify(snapshot, null, 2)}\n`;
}

export function parseWorkspaceSnapshot(text) {
  let snapshot;
  try {
    snapshot = JSON.parse(String(text ?? ""));
  } catch {
    throw new Error("Snapshot import failed: snapshot was not valid JSON.");
  }

  if (!snapshot || typeof snapshot !== "object" || Array.isArray(snapshot)) {
    throw new Error("Snapshot import failed: snapshot must be a JSON object.");
  }
  if (snapshot.version !== SNAPSHOT_VERSION) {
    throw new Error(`Snapshot import failed: snapshot version ${JSON.stringify(snapshot.version)} is not supported.`);
  }

  const workspace = snapshot.workspace;
  if (!workspace || typeof workspace !== "object" || Array.isArray(workspace)) {
    throw new Error("Snapshot import failed: workspace must be a JSON object.");
  }

  const files = cloneWorkspaceFiles(workspace.files);
  if (files.length === 0) {
    throw new Error("Snapshot import failed: workspace files must be non-empty.");
  }

  const filePathSet = new Set(files.map((file) => file.path));
  const selectedFilePath = normalizeWorkspacePath(workspace.selected_file_path ?? "");
  const entryPath = normalizeWorkspacePath(workspace.entry_path ?? "");
  const packageTarget = normalizeWorkspacePath(workspace.package_target ?? "");
  if (selectedFilePath && !filePathSet.has(selectedFilePath)) {
    throw new Error(`Snapshot import failed: selected file ${JSON.stringify(selectedFilePath)} is missing from the workspace files.`);
  }
  if (entryPath && !filePathSet.has(entryPath)) {
    throw new Error(`Snapshot import failed: entry path ${JSON.stringify(entryPath)} is missing from the workspace files.`);
  }
  if (packageTarget && !filePathSet.has(packageTarget)) {
    throw new Error(`Snapshot import failed: package target ${JSON.stringify(packageTarget)} is missing from the workspace files.`);
  }

  const moduleRoots = normalizeModuleRoots(snapshot.module_roots);
  const moduleRootsText = moduleRoots
    .map((module) => `${module.module_path} ${module.version} ${module.fetch_url}`)
    .join("\n");

  return {
    examples: normalizeExamples(snapshot.examples),
    files,
    loadedModuleRefs: normalizeLoadedModuleRefs(snapshot.loaded_module_refs),
    moduleRoots,
    moduleRootsText,
    packageTarget,
    selectedFilePath: selectedFilePath || files[0]?.path || "",
    settings: normalizeSettings(snapshot.settings),
    snapshot,
    version: SNAPSHOT_VERSION,
    entryPath: entryPath || detectEntryPath(files),
  };
}

export function formatWorkspaceSnapshotSummary(snapshot) {
  const lines = [
    `Workspace snapshot v${snapshot.version}.`,
    `Workspace files: ${snapshot.workspace.files.length}`,
    `Configured module roots: ${snapshot.module_roots.length}`,
    `Loaded module refs: ${snapshot.loaded_module_refs.length}`,
    `Selected file: ${snapshot.workspace.selected_file_path || "(none)"}`,
    `Entry path: ${snapshot.workspace.entry_path || "(none)"}`,
    `Package target: ${snapshot.workspace.package_target || "(none)"}`,
    "",
    "Workspace files:",
  ];
  for (const file of snapshot.workspace.files) {
    lines.push(`- ${file.path}`);
  }
  return lines.join("\n");
}

export function snapshotDownloadFileName() {
  return "gowasm-workspace.snapshot.json";
}

export async function readSnapshotFile(file) {
  if (!file) {
    throw new Error("Snapshot import failed: choose a snapshot file first.");
  }
  return parseWorkspaceSnapshot(await file.text());
}

export function createSnapshotController(ctx) {
  async function exportSnapshot() {
    if (ctx.isBusy()) {
      return;
    }

    try {
      const snapshot = createWorkspaceSnapshot({
        entryPath: ctx.entryPathInput.value,
        loadedModuleBundles: ctx.getLoadedModuleBundles(),
        moduleRootsText: ctx.moduleRootsInput.value,
        packageTarget: ctx.packageTargetInput.value,
        selectedFilePath: ctx.getSelectedFilePath(),
        workspaceFiles: ctx.getWorkspaceFiles(),
      });
      const contents = serializeWorkspaceSnapshot(snapshot);
      const blob = new Blob([contents], { type: "application/json" });
      ctx.downloadSnapshotBlob(blob, snapshotDownloadFileName());
      ctx.statusElement.textContent = "Workspace snapshot exported";
      ctx.setOutputView(formatWorkspaceSnapshotSummary(snapshot), []);
    } catch (error) {
      ctx.statusElement.textContent = "Snapshot export failed";
      ctx.setOutputView(error?.message || String(error), []);
    }
  }

  async function importSnapshotFromFile(file) {
    if (ctx.isBusy()) {
      return;
    }

    try {
      const restored = await readSnapshotFile(file);
      applyRestoredSnapshotToShell(ctx, restored, {
        outputText: formatWorkspaceSnapshotSummary(restored.snapshot),
        statusText: "Workspace snapshot imported",
      });
      await ctx.onImportedWorkspace?.({
        sourceKind: "snapshot_import",
        sourceLabel: file?.name || "gowasm-workspace.snapshot.json",
      });
    } catch (error) {
      ctx.statusElement.textContent = "Snapshot import failed";
      ctx.setOutputView(error?.message || String(error), []);
    }
  }

  return {
    exportSnapshot,
    importSnapshotFromFile,
  };
}

export function applyRestoredSnapshotToShell(ctx, restored, options = {}) {
  ctx.setWorkspaceFiles(restored.files, restored.selectedFilePath, {
    resetDirtyBaseline: true,
  });
  ctx.setSelectedExampleId?.(restored.examples?.selected_example_id ?? "");
  ctx.entryPathInput.value = restored.entryPath || "";
  ctx.packageTargetInput.value = restored.packageTarget || "";
  ctx.moduleRootsInput.value = restored.moduleRootsText;
  ctx.resetLoadedModules();
  ctx.statusElement.textContent = options.statusText || "Workspace snapshot imported";
  ctx.setOutputView(
    options.outputText || formatWorkspaceSnapshotSummary(restored.snapshot),
    [],
  );
  ctx.renderModuleStatus();
  ctx.renderWorkspace();
  ctx.syncControls();
}

function normalizeModuleRoots(moduleRoots) {
  if (moduleRoots === undefined) {
    return [];
  }
  if (!Array.isArray(moduleRoots)) {
    throw new Error("Snapshot import failed: module_roots must be an array.");
  }

  const normalized = [];
  const seen = new Set();
  for (const module of moduleRoots) {
    const modulePath = String(module?.module_path ?? "").trim();
    const version = String(module?.version ?? "").trim();
    const fetchUrl = String(module?.fetch_url ?? "").trim();
    if (!modulePath || !version || !fetchUrl) {
      throw new Error("Snapshot import failed: every module root must include module_path, version, and fetch_url.");
    }
    const dedupeKey = `${modulePath}@${version}`;
    if (seen.has(dedupeKey)) {
      throw new Error(`Snapshot import failed: duplicate module root ${dedupeKey}.`);
    }
    seen.add(dedupeKey);
    normalized.push({
      fetch_url: fetchUrl,
      module_path: modulePath,
      version,
    });
  }
  return normalized;
}

function normalizeLoadedModuleRefs(moduleRefsOrBundles) {
  const normalized = [];
  const seen = new Set();
  for (const value of moduleRefsOrBundles ?? []) {
    const module = value?.module ?? value;
    const modulePath = String(module?.module_path ?? "").trim();
    const version = String(module?.version ?? "").trim();
    if (!modulePath || !version) {
      continue;
    }
    const dedupeKey = `${modulePath}@${version}`;
    if (seen.has(dedupeKey)) {
      continue;
    }
    seen.add(dedupeKey);
    const projectedFileCount = Array.isArray(value?.files) ? value.files.length : 0;
    normalized.push({
      module_path: modulePath,
      projected_file_count: projectedFileCount,
      version,
    });
  }
  normalized.sort((left, right) => moduleRootsConfigKey([left]).localeCompare(moduleRootsConfigKey([right])));
  return normalized;
}

function normalizeSettings(settings) {
  if (settings === undefined) {
    return {
      lint_scope: "editable_workspace",
      projected_module_cache_paths_reserved: true,
      run_uses_loaded_module_bundles: true,
    };
  }
  if (!settings || typeof settings !== "object" || Array.isArray(settings)) {
    throw new Error("Snapshot import failed: settings must be a JSON object when present.");
  }
  return {
    lint_scope: String(settings.lint_scope ?? "editable_workspace"),
    projected_module_cache_paths_reserved: Boolean(
      settings.projected_module_cache_paths_reserved ?? true,
    ),
    run_uses_loaded_module_bundles: Boolean(settings.run_uses_loaded_module_bundles ?? true),
  };
}

function normalizeExamples(examples) {
  if (examples === undefined) {
    return {
      packaged_examples: [],
      selected_example_id: "",
    };
  }
  if (!examples || typeof examples !== "object" || Array.isArray(examples)) {
    throw new Error("Snapshot import failed: examples must be a JSON object when present.");
  }
  if (examples.packaged_examples !== undefined && !Array.isArray(examples.packaged_examples)) {
    throw new Error("Snapshot import failed: examples.packaged_examples must be an array when present.");
  }
  return {
    packaged_examples: [...(examples.packaged_examples ?? [])].map((value) => String(value)),
    selected_example_id: String(examples.selected_example_id ?? ""),
  };
}

function detectEntryPath(files) {
  if (files.some((file) => file.path === "main.go")) {
    return "main.go";
  }
  const mainCandidates = files
    .filter((file) => file.path.endsWith("/main.go"))
    .map((file) => file.path)
    .sort();
  if (mainCandidates.length > 0) {
    return mainCandidates[0];
  }
  return findWorkspaceFile(files, files[0]?.path ?? "")?.path ?? "";
}
