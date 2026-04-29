const MODULE_CACHE_PREFIX = "__module_cache__/";
const SUPPORTED_EDITABLE_EXTENSIONS = new Set([
  ".css",
  ".csv",
  ".env",
  ".go",
  ".gql",
  ".graphql",
  ".html",
  ".js",
  ".json",
  ".jsx",
  ".md",
  ".mod",
  ".proto",
  ".sh",
  ".sql",
  ".sum",
  ".toml",
  ".ts",
  ".tsx",
  ".txt",
  ".xml",
  ".yaml",
  ".yml",
]);
const SUPPORTED_EDITABLE_BASENAMES = new Set([
  "Dockerfile",
  "LICENSE",
  "LICENSE.txt",
  "Makefile",
  "README",
  "README.md",
  "go.mod",
  "go.sum",
]);

export const DEFAULT_BROWSER_WORKSPACE_FILES = [
  {
    path: "go.mod",
    contents: "module example.com/app\n\ngo 1.21\n",
  },
  {
    path: "main.go",
    contents: `package main
import "fmt"

func helper() {
\tfmt.Println("worker shell")
}

func main() {
\thelper()
\tfmt.Println("Rust engine next")
}
`,
  },
];

export function normalizeWorkspacePath(path) {
  let normalized = String(path ?? "").trim();
  normalized = normalized.replace(/^\/+/, "");
  while (normalized.startsWith("./")) {
    normalized = normalized.slice(2);
  }
  return normalized;
}

export function cloneWorkspaceFiles(files) {
  return (files ?? []).map((file) => ({
    path: normalizeWorkspacePath(file?.path ?? ""),
    contents: String(file?.contents ?? ""),
  }));
}

export function isModuleCachePath(path) {
  return normalizeWorkspacePath(path).startsWith(MODULE_CACHE_PREFIX);
}

export function editableWorkspaceFiles(files) {
  return cloneWorkspaceFiles(files).filter((file) => !isModuleCachePath(file.path));
}

export function findWorkspaceFile(files, path) {
  const normalizedPath = normalizeWorkspacePath(path);
  return (files ?? []).find((file) => normalizeWorkspacePath(file?.path) === normalizedPath) ?? null;
}

export function upsertWorkspaceFile(files, file) {
  const normalizedPath = normalizeWorkspacePath(file?.path ?? "");
  if (!normalizedPath) {
    throw new Error("workspace file paths cannot be empty");
  }

  const nextFile = {
    path: normalizedPath,
    contents: String(file?.contents ?? ""),
  };
  const existingIndex = (files ?? []).findIndex(
    (candidate) => normalizeWorkspacePath(candidate?.path) === normalizedPath,
  );
  if (existingIndex < 0) {
    return [...cloneWorkspaceFiles(files), nextFile];
  }

  return cloneWorkspaceFiles(files).map((candidate, index) =>
    index === existingIndex ? nextFile : candidate,
  );
}

export function removeWorkspaceFile(files, path) {
  const normalizedPath = normalizeWorkspacePath(path);
  return cloneWorkspaceFiles(files).filter((file) => file.path !== normalizedPath);
}

export function defaultWorkspaceFileContents(path) {
  if (path === "go.mod") {
    return "module example.com/app\n\ngo 1.21\n";
  }
  if (!path.endsWith(".go")) {
    return "";
  }

  const segments = path.split("/");
  const fileName = segments[segments.length - 1] ?? "file.go";
  let packageName = segments.length > 1 ? segments[segments.length - 2] : "main";
  if (fileName.endsWith("_test.go")) {
    packageName = segments.length > 1 ? segments[segments.length - 2] : "main";
  }
  packageName = packageName.replace(/[^A-Za-z0-9_]/g, "") || "main";
  return `package ${packageName}\n`;
}

export function isGoWorkspacePath(path) {
  return normalizeWorkspacePath(path).endsWith(".go");
}

export function isSupportedEditableWorkspacePath(path) {
  const normalizedPath = normalizeWorkspacePath(path);
  if (!normalizedPath || isModuleCachePath(normalizedPath)) {
    return false;
  }

  const basename = normalizedPath.split("/").pop() ?? normalizedPath;
  const extensionIndex = basename.lastIndexOf(".");
  const extension = extensionIndex >= 0 ? basename.slice(extensionIndex) : "";
  return SUPPORTED_EDITABLE_BASENAMES.has(basename) || SUPPORTED_EDITABLE_EXTENSIONS.has(extension);
}

export function renameWorkspaceFile(files, oldPath, newPath) {
  const normalizedOldPath = normalizeWorkspacePath(oldPath);
  const normalizedNewPath = normalizeWorkspacePath(newPath);
  if (!normalizedOldPath || !normalizedNewPath) {
    throw new Error("workspace file rename paths cannot be empty");
  }
  if (normalizedOldPath === normalizedNewPath) {
    return cloneWorkspaceFiles(files);
  }

  const nextFiles = cloneWorkspaceFiles(files);
  const existingIndex = nextFiles.findIndex((file) => file.path === normalizedOldPath);
  if (existingIndex < 0) {
    throw new Error(`workspace file ${JSON.stringify(normalizedOldPath)} does not exist`);
  }
  if (nextFiles.some((file) => file.path === normalizedNewPath)) {
    throw new Error(`workspace file ${JSON.stringify(normalizedNewPath)} already exists`);
  }

  nextFiles[existingIndex] = {
    path: normalizedNewPath,
    contents: nextFiles[existingIndex].contents,
  };
  return nextFiles;
}

export function moduleCacheSourcePath(modulePath, version, relativePath) {
  return `__module_cache__/${modulePath}/@${version}/${normalizeWorkspacePath(relativePath)}`;
}

export function moduleBundlesToWorkspaceFiles(workspaceFiles, bundles) {
  const byPath = new Map();
  const combined = [];

  for (const file of cloneWorkspaceFiles(workspaceFiles)) {
    byPath.set(file.path, combined.length);
    combined.push(file);
  }

  for (const bundle of bundles ?? []) {
    for (const file of bundle?.files ?? []) {
      const projectedPath = moduleCacheSourcePath(
        bundle?.module?.module_path ?? "",
        bundle?.module?.version ?? "",
        file?.path ?? "",
      );
      const nextFile = {
        path: projectedPath,
        contents: String(file?.contents ?? ""),
      };
      const existingIndex = byPath.get(projectedPath);
      if (existingIndex === undefined) {
        byPath.set(projectedPath, combined.length);
        combined.push(nextFile);
      } else {
        combined[existingIndex] = nextFile;
      }
    }
  }

  return combined;
}

export function parseModuleGraphRoots(configText) {
  const modules = [];
  const errors = [];
  const seen = new Set();

  for (const [index, rawLine] of String(configText ?? "").split(/\r?\n/).entries()) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }

    const [modulePath = "", version = "", ...fetchUrlParts] = line.split(/\s+/);
    const fetchUrl = fetchUrlParts.join(" ").trim();
    if (!modulePath || !version || !fetchUrl) {
      errors.push(
        `line ${index + 1}: expected "module_path version fetch_url", got ${JSON.stringify(rawLine)}`,
      );
      continue;
    }

    const dedupeKey = `${modulePath}@${version}`;
    if (seen.has(dedupeKey)) {
      errors.push(`line ${index + 1}: duplicate module root ${dedupeKey}`);
      continue;
    }
    seen.add(dedupeKey);
    modules.push({
      module_path: modulePath,
      version,
      fetch_url: fetchUrl,
    });
  }

  return { modules, errors };
}

export function moduleRootsConfigKey(modules) {
  return JSON.stringify(
    (modules ?? []).map((module) => ({
      module_path: module?.module_path ?? "",
      version: module?.version ?? "",
      fetch_url: module?.fetch_url ?? "",
    })),
  );
}

export function configuredModuleRootsAreFresh(
  modules,
  loadedModuleRootsKey,
  loadedBundlesStale = false,
) {
  return !loadedBundlesStale && moduleRootsConfigKey(modules) === loadedModuleRootsKey;
}
