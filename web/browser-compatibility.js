export const MIN_SUPPORTED_CHROMIUM_MAJOR = 146;

export const BROWSER_COMPATIBILITY_MATRIX = [
  {
    browserClass: "chromium",
    minimumVersion: `${MIN_SUPPORTED_CHROMIUM_MAJOR}+`,
    status: "supported",
    checkedBy: "browser worker, browser shell, and browser performance gates",
    notes:
      "Chromium-derived desktop/headless browsers at or above the checked floor are the only supported worker-backed shell class.",
  },
  {
    browserClass: "firefox",
    minimumVersion: "128+",
    status: "unsupported",
    checkedBy: "none",
    notes:
      "Firefox-like browsers are not part of the checked worker/module-cache/browser-shell contract and stay outside the supported release surface.",
  },
  {
    browserClass: "safari",
    minimumVersion: "17.6+",
    status: "unsupported",
    checkedBy: "none",
    notes:
      "Safari/WebKit-like browsers are not part of the checked worker/module-cache/browser-shell contract and stay outside the supported release surface.",
  },
];

const REQUIRED_FEATURES = [
  {
    id: "webassembly",
    label: "WebAssembly instantiate",
    check: (target) => typeof target?.WebAssembly?.instantiate === "function",
  },
  {
    id: "module_worker",
    label: "Dedicated module Worker",
    check: (target) =>
      typeof target?.Worker === "function" &&
      typeof target?.Blob === "function" &&
      typeof target?.URL?.createObjectURL === "function",
  },
  {
    id: "fetch",
    label: "fetch",
    check: (target) => typeof target?.fetch === "function",
  },
  {
    id: "text_encoding",
    label: "TextEncoder/TextDecoder",
    check: (target) =>
      typeof target?.TextEncoder === "function" && typeof target?.TextDecoder === "function",
  },
  {
    id: "url_apis",
    label: "URL and URLSearchParams",
    check: (target) =>
      typeof target?.URL === "function" && typeof target?.URLSearchParams === "function",
  },
];

const LIMITED_FEATURES = [
  {
    id: "indexeddb",
    label: "IndexedDB",
    check: (target) => typeof target?.indexedDB !== "undefined",
  },
  {
    id: "storage_estimate",
    label: "navigator.storage.estimate",
    check: (target) => typeof target?.navigator?.storage?.estimate === "function",
  },
];

export function detectBrowserClass(userAgent) {
  const normalized = String(userAgent || "");
  if (/(Chrome|Chromium|HeadlessChrome|Edg)\//.test(normalized)) {
    return "chromium";
  }
  if (/Firefox\//.test(normalized)) {
    return "firefox";
  }
  if (/Safari\//.test(normalized) && !/(Chrome|Chromium|HeadlessChrome|Edg)\//.test(normalized)) {
    return "safari";
  }
  return "unknown";
}

export function detectBrowserMajorVersion(browserClass, userAgent) {
  const normalized = String(userAgent || "");
  const patterns = {
    chromium: /(?:Chrome|Chromium|HeadlessChrome|Edg)\/(\d+)/,
    firefox: /Firefox\/(\d+)/,
    safari: /Version\/(\d+)/,
  };
  const match = normalized.match(patterns[browserClass] || /$^/);
  return match ? Number.parseInt(match[1], 10) : null;
}

export function evaluateBrowserCompatibility(target = globalThis) {
  const userAgent = String(target?.navigator?.userAgent || "");
  const browserClass = detectBrowserClass(userAgent);
  const browserMajorVersion = detectBrowserMajorVersion(browserClass, userAgent);

  const requiredFeatures = REQUIRED_FEATURES.map((feature) => ({
    id: feature.id,
    label: feature.label,
    supported: Boolean(feature.check(target)),
    required: true,
  }));
  const limitedFeatures = LIMITED_FEATURES.map((feature) => ({
    id: feature.id,
    label: feature.label,
    supported: Boolean(feature.check(target)),
    required: false,
  }));

  const missingRequired = requiredFeatures.filter((feature) => !feature.supported);
  const chromiumClassSupported =
    browserClass === "chromium" &&
    Number.isFinite(browserMajorVersion) &&
    browserMajorVersion >= MIN_SUPPORTED_CHROMIUM_MAJOR;
  const workerExecutionSupported = chromiumClassSupported && missingRequired.length === 0;
  const cachePersistenceSupported = limitedFeatures.find((feature) => feature.id === "indexeddb")?.supported === true;
  const storageEstimateSupported =
    limitedFeatures.find((feature) => feature.id === "storage_estimate")?.supported === true;

  const warnings = [];
  if (browserClass !== "chromium") {
    warnings.push(
      `The checked support class is Chromium ${MIN_SUPPORTED_CHROMIUM_MAJOR}+ only; detected ${browserClass || "unknown"} stays outside the supported shell contract.`,
    );
  } else if (
    !Number.isFinite(browserMajorVersion) ||
    browserMajorVersion < MIN_SUPPORTED_CHROMIUM_MAJOR
  ) {
    warnings.push(
      `Chromium ${MIN_SUPPORTED_CHROMIUM_MAJOR}+ is required for the checked browser contract; detected ${browserMajorVersion ?? "unknown"}.`,
    );
  }
  if (!cachePersistenceSupported) {
    warnings.push(
      "IndexedDB is unavailable, so imported-workspace, example-project, and module caches stay disabled.",
    );
  }
  if (!storageEstimateSupported) {
    warnings.push(
      "Browser quota estimates are unavailable; the cache panel omits usage/quota reporting but the shell can still run.",
    );
  }

  const status = !workerExecutionSupported
    ? "unsupported"
    : cachePersistenceSupported
      ? "supported"
      : "limited";

  return {
    browserClass,
    browserMajorVersion,
    cachePersistenceSupported,
    features: [...requiredFeatures, ...limitedFeatures],
    missingRequired,
    status,
    storageEstimateSupported,
    supported: workerExecutionSupported,
    warnings,
  };
}

export function formatBrowserCompatibilityReport(report) {
  const lines = [
    `Support profile: Chromium ${MIN_SUPPORTED_CHROMIUM_MAJOR}+ (${report.status})`,
    `Detected browser class: ${report.browserClass}${Number.isFinite(report.browserMajorVersion) ? ` ${report.browserMajorVersion}` : ""}`,
    `Worker-backed shell actions: ${report.supported ? "enabled" : "disabled"}.`,
    `Required runtime features: ${formatFeatureList(report.features.filter((feature) => feature.required))}.`,
    `Cache persistence: ${report.cachePersistenceSupported ? "enabled via IndexedDB" : "disabled (IndexedDB unavailable)"}.`,
    `Storage estimate: ${report.storageEstimateSupported ? "available" : "unavailable (quota line omitted)"}.`,
    "Fallback: when the checked Chromium floor or a required runtime feature is missing, the shell stays in workspace-edit mode and keeps run/format/lint/test/module-load actions disabled.",
  ];

  if (report.warnings.length > 0) {
    lines.push("");
    lines.push("Warnings:");
    for (const warning of report.warnings) {
      lines.push(`- ${warning}`);
    }
  }

  return lines.join("\n");
}

function formatFeatureList(features) {
  return features
    .map((feature) => `${feature.label}: ${feature.supported ? "yes" : "no"}`)
    .join("; ");
}
