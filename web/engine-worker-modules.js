import { normalizeModuleRelativePath } from "./browser-capability-security.js";
import { formatError } from "./engine-worker-runtime.js";
import { fillModuleCache, lookupModuleCache } from "./engine-worker-module-cache.js";

export async function buildModuleResumeRequest(response) {
  if (response.module?.kind === "cache_lookup") {
    return {
      kind: "resume_module",
      request_id: response.request_id,
      module: {
        kind: "cache_lookup",
        result: await lookupModuleCache(response.module.request),
      },
    };
  }
  if (response.module?.kind === "fetch") {
    return {
      kind: "resume_module",
      request_id: response.request_id,
      module: {
        kind: "fetch",
        result: await performModuleFetch(response.module.request),
      },
    };
  }
  if (response.module?.kind === "cache_fill") {
    await fillModuleCache(response.module.request);
    return {
      kind: "resume_module",
      request_id: response.request_id,
      module: {
        kind: "cache_fill",
      },
    };
  }
  throw new Error(`unsupported engine module request: ${JSON.stringify(response)}`);
}

async function performModuleFetch(request) {
  try {
    const response = await fetch(request.fetch_url);
    if (!response.ok) {
      return {
        kind: "error",
        message: `fetch failed with ${response.status} ${response.statusText}`,
      };
    }
    const payload = await response.json();
    const files = normalizeModuleFiles(payload?.files);
    return {
      kind: "module",
      module: {
        module: {
          module_path: payload?.module?.module_path ?? request.module.module_path,
          version: payload?.module?.version ?? request.module.version,
        },
        origin_url: normalizeOriginUrl(response.url, request.fetch_url),
        files,
      },
    };
  } catch (error) {
    return {
      kind: "error",
      message: `fetch failed for ${request.module.module_path}@${request.module.version}: ${formatError(error)}`,
    };
  }
}

function normalizeModuleFiles(files) {
  if (!Array.isArray(files)) {
    throw new Error("module fetch payload must include a files array");
  }
  return files.map((file, index) => {
    if (typeof file?.path !== "string" || typeof file?.contents !== "string") {
      throw new Error(`module fetch payload file ${index} was malformed`);
    }
    return {
      path: normalizeModuleRelativePath(file.path),
      contents: file.contents,
    };
  });
}

function normalizeOriginUrl(responseUrl, fallbackUrl) {
  if (typeof responseUrl === "string" && responseUrl.length > 0) {
    return responseUrl;
  }
  return String(fallbackUrl ?? "");
}
