import { cloneWorkspaceFiles, isModuleCachePath, normalizeWorkspacePath } from "./browser-workspace.js";

const SUPPORTED_ACTIONS = new Set(["run", "test_snippet", "test_package"]);

export function parseBootRequest(search) {
  const params = new URLSearchParams(search);
  const manifestUrl = String(params.get("boot_manifest_url") ?? "").trim();
  const expectedRevision = String(params.get("boot_manifest_revision") ?? "").trim();
  const entryPath = normalizeOptionalPath(params.get("boot_entry_path"));
  const packageTarget = normalizeOptionalPath(params.get("boot_package_target"));
  const action = normalizeBootAction(params.get("boot_action"));
  const consentGranted = params.get("boot_consent") === "1";
  const errors = [];

  if (!manifestUrl) {
    return {
      present: false,
      consentGranted: false,
      errors,
    };
  }
  if (params.has("boot_action") && !action) {
    errors.push('boot_action must be "run", "test_snippet", or "test_package".');
  }

  return {
    action,
    consentGranted,
    entryPath,
    errors,
    expectedRevision,
    manifestUrl,
    packageTarget,
    present: true,
  };
}

export async function fetchBootManifest(bootRequest) {
  if (!bootRequest?.manifestUrl) {
    throw new Error("Boot URL manifest is missing.");
  }

  const response = await fetch(bootRequest.manifestUrl, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`Boot manifest download failed: ${response.status} ${response.statusText}`.trim());
  }

  let manifest;
  try {
    manifest = await response.json();
  } catch {
    throw new Error("Boot manifest download failed: response was not valid JSON.");
  }

  const normalized = normalizeBootManifest(manifest, bootRequest);
  return {
    ...normalized,
    sourceLabel: bootRequest.manifestUrl,
  };
}

export function formatBootSummary(bootResult) {
  const lines = [
    `Loaded boot manifest from ${bootResult.sourceLabel}.`,
  ];
  if (bootResult.revision) {
    lines.push(`Revision: ${bootResult.revision}`);
  }
  if (bootResult.entryPath) {
    lines.push(`Entry path: ${bootResult.entryPath}`);
  }
  if (bootResult.packageTarget) {
    lines.push(`Package target: ${bootResult.packageTarget}`);
  }
  if (bootResult.action) {
    lines.push(`Auto action: ${bootResult.action}`);
  }
  if (bootResult.moduleRoots.length > 0) {
    lines.push(`Remote module roots: ${bootResult.moduleRoots.length}`);
  }
  lines.push("");
  lines.push("Workspace files:");
  for (const file of bootResult.files) {
    lines.push(`- ${file.path}`);
  }
  return lines.join("\n");
}

export function formatBootModuleRoots(moduleRoots) {
  return (moduleRoots ?? [])
    .map((module) => `${module.module_path} ${module.version} ${module.fetch_url}`)
    .join("\n");
}

export function createBootUrlController(ctx) {
  const bootRequest = parseBootRequest(ctx.search);
  let autoAction = "";
  let pending = false;

  function isPending() {
    return pending;
  }

  function renderBootUrlPanel() {
    if (!bootRequest.present) {
      ctx.bootUrlPanel.hidden = true;
      return;
    }

    ctx.bootUrlPanel.hidden = false;
    if (bootRequest.errors.length > 0) {
      ctx.bootUrlStatus.textContent = `Boot URL invalid:\n${bootRequest.errors.join("\n")}`;
      return;
    }

    const revisionLine = bootRequest.expectedRevision
      ? `\nExpected revision: ${bootRequest.expectedRevision}`
      : "";
    if (bootRequest.consentGranted) {
      ctx.bootUrlStatus.textContent =
        `Boot manifest URL ready:${revisionLine}\n${bootRequest.manifestUrl}`;
    } else {
      ctx.bootUrlStatus.textContent =
        `Boot manifest URL present but awaiting explicit load consent:${revisionLine}\n${bootRequest.manifestUrl}`;
    }
  }

  function maybeAutoloadBootManifest() {
    if (!bootRequest.present || bootRequest.errors.length > 0 || !bootRequest.consentGranted) {
      if (bootRequest.errors.length > 0) {
        ctx.statusElement.textContent = "Boot URL invalid";
        ctx.setOutputView(bootRequest.errors.join("\n"), []);
      }
      return;
    }
    void loadBootManifest("auto");
  }

  async function loadBootManifest(trigger) {
    if (!bootRequest.present || bootRequest.errors.length > 0 || pending || ctx.isBusy()) {
      return;
    }

    pending = true;
    ctx.statusElement.textContent =
      trigger === "auto" ? "Loading boot manifest…" : "Loading boot manifest after consent…";
    ctx.setOutputView("", []);
    ctx.syncControls();

    try {
      const bootManifest = await fetchBootManifest(bootRequest);
      ctx.applyBootManifest(bootManifest);
      await ctx.onBootManifestLoaded?.(bootManifest);
      autoAction = bootManifest.action || "";
      ctx.statusElement.textContent = autoAction
        ? `Boot manifest loaded; starting ${autoAction}…`
        : "Boot manifest loaded";
      ctx.setOutputView(formatBootSummary(bootManifest), []);
      ctx.renderModuleStatus();
      ctx.renderWorkspace();
    } catch (error) {
      ctx.statusElement.textContent = "Boot URL failed";
      ctx.setOutputView(error?.message || String(error), []);
    } finally {
      pending = false;
      ctx.syncControls();
      maybeRunBootAction();
    }
  }

  function maybeRunBootAction() {
    if (!autoAction || !ctx.isWorkerReady() || ctx.isBusy()) {
      return;
    }

    const action = autoAction;
    autoAction = "";
    switch (action) {
      case "run":
        ctx.requestRun();
        break;
      case "test_snippet":
        ctx.requestSnippetTest();
        break;
      case "test_package":
        ctx.requestPackageTest();
        break;
      default:
        break;
    }
  }

  return {
    bootRequest,
    isPending,
    loadBootManifest,
    maybeAutoloadBootManifest,
    maybeRunBootAction,
    renderBootUrlPanel,
  };
}

function normalizeBootManifest(manifest, bootRequest) {
  if (!manifest || typeof manifest !== "object" || Array.isArray(manifest)) {
    throw new Error("Boot manifest download failed: manifest must be a JSON object.");
  }
  if (manifest.version !== 1) {
    throw new Error(`Boot manifest version ${JSON.stringify(manifest.version)} is not supported.`);
  }

  const revision = typeof manifest.revision === "string" ? manifest.revision.trim() : "";
  if (bootRequest.expectedRevision && revision !== bootRequest.expectedRevision) {
    throw new Error(
      `Boot manifest is stale: expected revision ${bootRequest.expectedRevision}, got ${revision || "(missing revision)"}.`,
    );
  }

  const files = normalizeBootFiles(manifest.files);
  const moduleRoots = normalizeBootModuleRoots(manifest.module_roots);
  const manifestEntryPath = normalizeOptionalPath(manifest.entry_path);
  const manifestPackageTarget = normalizeOptionalPath(manifest.package_target);
  const manifestAction = normalizeBootAction(manifest.action);
  if (manifest.action !== undefined && manifestAction === "") {
    throw new Error('Boot manifest action must be "run", "test_snippet", or "test_package".');
  }

  return {
    action: bootRequest.action || manifestAction,
    entryPath: bootRequest.entryPath || manifestEntryPath || detectBootEntryPath(files),
    files,
    moduleRoots,
    packageTarget: bootRequest.packageTarget || manifestPackageTarget || "",
    revision,
  };
}

function normalizeBootFiles(files) {
  if (!Array.isArray(files) || files.length === 0) {
    throw new Error("Boot manifest must include a non-empty files array.");
  }

  const normalizedFiles = [];
  const seenPaths = new Set();
  for (const file of files) {
    const path = normalizeRequiredPath(file?.path, "Boot manifest file path");
    if (isModuleCachePath(path)) {
      throw new Error(`Boot manifest path ${JSON.stringify(path)} is reserved for projected module-cache files.`);
    }
    if (seenPaths.has(path)) {
      throw new Error(`Boot manifest contains duplicate file path ${JSON.stringify(path)}.`);
    }
    seenPaths.add(path);
    normalizedFiles.push({
      path,
      contents: String(file?.contents ?? ""),
    });
  }
  return cloneWorkspaceFiles(normalizedFiles);
}

function normalizeBootModuleRoots(moduleRoots) {
  if (moduleRoots === undefined) {
    return [];
  }
  if (!Array.isArray(moduleRoots)) {
    throw new Error("Boot manifest module_roots must be an array.");
  }

  const normalizedRoots = [];
  const seen = new Set();
  for (const module of moduleRoots) {
    const modulePath = String(module?.module_path ?? "").trim();
    const version = String(module?.version ?? "").trim();
    const fetchUrl = String(module?.fetch_url ?? "").trim();
    if (!modulePath || !version || !fetchUrl) {
      throw new Error("Boot manifest module roots must include module_path, version, and fetch_url.");
    }
    const dedupeKey = `${modulePath}@${version}`;
    if (seen.has(dedupeKey)) {
      throw new Error(`Boot manifest module roots contain duplicate ${dedupeKey}.`);
    }
    seen.add(dedupeKey);
    normalizedRoots.push({
      fetch_url: fetchUrl,
      module_path: modulePath,
      version,
    });
  }
  return normalizedRoots;
}

function detectBootEntryPath(files) {
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
  const goCandidates = files
    .filter((file) => file.path.endsWith(".go"))
    .map((file) => file.path)
    .sort();
  return goCandidates[0] ?? files[0]?.path ?? "";
}

function normalizeRequiredPath(path, label) {
  const normalized = normalizeOptionalPath(path);
  if (!normalized) {
    throw new Error(`${label} is required.`);
  }
  return normalized;
}

function normalizeOptionalPath(path) {
  const normalized = normalizeWorkspacePath(path ?? "");
  if (!normalized) {
    return "";
  }
  if (normalized.split("/").includes("..")) {
    return "";
  }
  return normalized;
}

function normalizeBootAction(action) {
  const normalized = String(action ?? "").trim();
  if (!normalized) {
    return "";
  }
  return SUPPORTED_ACTIONS.has(normalized) ? normalized : "";
}
