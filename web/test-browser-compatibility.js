import {
  detectBrowserClass,
  detectBrowserMajorVersion,
  evaluateBrowserCompatibility,
  formatBrowserCompatibilityReport,
  MIN_SUPPORTED_CHROMIUM_MAJOR,
} from "./browser-compatibility.js";

export async function runBrowserCompatibilityTests({ assert }) {
  const actual = evaluateBrowserCompatibility(window);
  assert(
    actual.supported,
    "actual browser is supported",
    formatBrowserCompatibilityReport(actual),
  );
  assert(
    actual.browserClass === "chromium",
    "actual browser class is chromium",
    JSON.stringify(actual),
  );
  assert(
    actual.browserMajorVersion >= MIN_SUPPORTED_CHROMIUM_MAJOR,
    "actual browser meets checked chromium floor",
    JSON.stringify(actual),
  );

  const fakeChromium = createFakeEnvironment(
    `Mozilla/5.0 Chrome/${MIN_SUPPORTED_CHROMIUM_MAJOR}.0.0.0 Safari/537.36`,
  );
  const fakeChromiumReport = evaluateBrowserCompatibility(fakeChromium);
  assert(
    fakeChromiumReport.status === "supported",
    "chromium environment is supported",
    formatBrowserCompatibilityReport(fakeChromiumReport),
  );

  const fakeChromiumWithoutCache = createFakeEnvironment(
    `Mozilla/5.0 Chrome/${MIN_SUPPORTED_CHROMIUM_MAJOR}.0.0.0 Safari/537.36`,
    { indexedDB: undefined },
  );
  const fakeChromiumWithoutCacheReport = evaluateBrowserCompatibility(fakeChromiumWithoutCache);
  assert(
    fakeChromiumWithoutCacheReport.status === "limited" &&
      !fakeChromiumWithoutCacheReport.cachePersistenceSupported,
    "missing indexeddb degrades to limited support",
    formatBrowserCompatibilityReport(fakeChromiumWithoutCacheReport),
  );

  const fakeFirefox = createFakeEnvironment("Mozilla/5.0 Firefox/128.0");
  const fakeFirefoxReport = evaluateBrowserCompatibility(fakeFirefox);
  assert(
    fakeFirefoxReport.status === "unsupported" && !fakeFirefoxReport.supported,
    "firefox class stays unsupported",
    formatBrowserCompatibilityReport(fakeFirefoxReport),
  );

  const fakeChromiumWithoutWorkers = createFakeEnvironment(
    `Mozilla/5.0 Chrome/${MIN_SUPPORTED_CHROMIUM_MAJOR}.0.0.0 Safari/537.36`,
    { Worker: undefined },
  );
  const fakeChromiumWithoutWorkersReport = evaluateBrowserCompatibility(fakeChromiumWithoutWorkers);
  assert(
    fakeChromiumWithoutWorkersReport.missingRequired.some(
      (feature) => feature.id === "module_worker",
    ) && !fakeChromiumWithoutWorkersReport.supported,
    "missing module worker disables shell support",
    formatBrowserCompatibilityReport(fakeChromiumWithoutWorkersReport),
  );

  assert(
    detectBrowserClass("Mozilla/5.0 Version/17.6 Safari/605.1.15") === "safari" &&
      detectBrowserMajorVersion("safari", "Mozilla/5.0 Version/17.6 Safari/605.1.15") === 17,
    "browser detection parses safari class and major version",
    "safari detection failed",
  );
}

function createFakeEnvironment(userAgent, overrides = {}) {
  function UrlConstructor() {}
  UrlConstructor.createObjectURL = () => "blob:gowasm";

  return {
    Blob: function Blob() {},
    TextDecoder: function TextDecoder() {},
    TextEncoder: function TextEncoder() {},
    URL: UrlConstructor,
    URLSearchParams: function URLSearchParams() {},
    WebAssembly: { instantiate() {} },
    Worker: function Worker() {},
    fetch: async () => ({ ok: true }),
    indexedDB: {},
    navigator: {
      storage: {
        estimate: async () => ({ usage: 1, quota: 2 }),
      },
      userAgent,
    },
    ...overrides,
  };
}
