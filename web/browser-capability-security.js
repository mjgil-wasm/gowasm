const EXECUTION_REQUEST_KINDS = new Set([
  "format",
  "lint",
  "run",
  "test_package",
  "test_snippet",
]);
const ALLOWED_WORKER_REQUEST_KINDS = new Set([
  "boot",
  "cancel",
  "format",
  "lint",
  "load_module_graph",
  "run",
  "test_package",
  "test_snippet",
]);

export function validateWorkerRequestEnvelope(request) {
  if (!request || typeof request !== "object" || Array.isArray(request)) {
    throw new Error("worker request must be a JSON object");
  }

  const kind = typeof request.kind === "string" ? request.kind : "";
  if (!ALLOWED_WORKER_REQUEST_KINDS.has(kind)) {
    throw new Error(`unsupported worker request kind: ${JSON.stringify(kind)}`);
  }

  if (EXECUTION_REQUEST_KINDS.has(kind)) {
    validateWorkspaceFilesInput(request.files, kind);
  }

  switch (kind) {
    case "run":
    case "test_snippet":
      validateRequiredString(request.entry_path, `${kind}.entry_path`);
      break;
    case "test_package":
      validateRequiredString(request.target_path, "test_package.target_path");
      if (request.filter !== undefined && typeof request.filter !== "string") {
        throw new Error("test_package.filter must be a string when present");
      }
      break;
    case "load_module_graph":
      validateModuleRootRequest(request.modules);
      break;
    default:
      break;
  }
}

export function normalizeModuleRelativePath(rawPath) {
  const trimmedPath = String(rawPath ?? "").trim().replace(/\\/g, "/");
  if (!trimmedPath) {
    throw new Error("module file paths cannot be empty");
  }
  if (/^[A-Za-z]:\//.test(trimmedPath) || trimmedPath.startsWith("/")) {
    throw new Error(`absolute module file path ${JSON.stringify(trimmedPath)} is not allowed`);
  }

  const normalizedSegments = [];
  for (const segment of trimmedPath.split("/")) {
    if (!segment || segment === ".") {
      continue;
    }
    if (segment === "..") {
      throw new Error(`module file path traversal ${JSON.stringify(trimmedPath)} is not allowed`);
    }
    normalizedSegments.push(segment);
  }

  const normalizedPath = normalizedSegments.join("/");
  if (!normalizedPath) {
    throw new Error("module file paths cannot normalize to an empty path");
  }
  if (normalizedPath.startsWith("__module_cache__/")) {
    throw new Error(`reserved projected module path ${JSON.stringify(normalizedPath)} is not allowed`);
  }
  return normalizedPath;
}

export function isSafeModuleFileRecord(file) {
  if (typeof file?.contents !== "string") {
    return false;
  }
  try {
    return normalizeModuleRelativePath(file?.path ?? "").length > 0;
  } catch {
    return false;
  }
}

export function validateWasmBufferWindow(memory, ptr, len, label) {
  if (!(memory?.buffer instanceof ArrayBuffer)) {
    throw new Error(`${label} memory was unavailable`);
  }
  if (!Number.isSafeInteger(ptr) || ptr < 0) {
    throw new Error(`${label} pointer was invalid`);
  }
  if (!Number.isSafeInteger(len) || len < 0) {
    throw new Error(`${label} length was invalid`);
  }
  const end = ptr + len;
  if (!Number.isSafeInteger(end) || end > memory.buffer.byteLength) {
    throw new Error(`${label} exceeded wasm memory bounds`);
  }
}

function validateWorkspaceFilesInput(files, kind) {
  if (!Array.isArray(files)) {
    throw new Error(`${kind}.files must be an array`);
  }
  for (const [index, file] of files.entries()) {
    if (typeof file?.path !== "string" || typeof file?.contents !== "string") {
      throw new Error(`${kind}.files[${index}] was malformed`);
    }
  }
}

function validateModuleRootRequest(modules) {
  if (!Array.isArray(modules)) {
    throw new Error("load_module_graph.modules must be an array");
  }
  for (const [index, module] of modules.entries()) {
    validateRequiredString(module?.module_path, `load_module_graph.modules[${index}].module_path`);
    validateRequiredString(module?.version, `load_module_graph.modules[${index}].version`);
    validateRequiredString(module?.fetch_url, `load_module_graph.modules[${index}].fetch_url`);
  }
}

function validateRequiredString(value, label) {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
}
