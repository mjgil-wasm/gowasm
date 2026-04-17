import { createArchiveDataUrl } from "./test-browser-archive-fixtures.js";
import { resetBrowserShellCacheDatabase } from "./test-browser-shell-cache-fixtures.js";
import {
  click,
  clickAndWaitFor,
  control,
  loadShellFrame,
  selectedSource,
  setInputValue,
  setPlainValue,
  setWorkspaceFileContents,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";
import { resetModuleCacheDatabase } from "./test-worker-modules.js";

const DEFAULT_CYCLE_COUNT = 6;
const MAX_CYCLE_COUNT = 12;

const results = document.querySelector("#results");
const summary = document.querySelector("#summary");
const artifact = document.querySelector("#artifact");
const frame = document.querySelector("#shell-frame");
const params = new URLSearchParams(window.location.search);
const ciMode = params.has("ci");

let passed = 0;
let failed = 0;

function log(message) {
  results.textContent += `${message}\n`;
}

function pass(name) {
  passed += 1;
  log(`  PASS  ${name}`);
}

function fail(name, reason) {
  failed += 1;
  log(`  FAIL  ${name}: ${reason}`);
}

function assert(condition, name, detail) {
  if (condition) {
    pass(name);
    return;
  }
  fail(name, detail || "assertion failed");
}

function publishCiElement(elementId, text, className = "") {
  if (!ciMode) {
    return;
  }
  void fetch("/__gowasm_ci_complete", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ elementId, className, text }),
  });
}

function setArtifactText(text) {
  artifact.textContent = text;
  publishCiElement("artifact", text);
}

function effectiveSeed() {
  const numeric = Number.parseInt(params.get("seed") || "", 10);
  if (Number.isFinite(numeric) && numeric > 0) {
    return numeric;
  }
  return Date.now();
}

function effectiveCycleCount() {
  const numeric = Number.parseInt(params.get("cycles") || "", 10);
  if (!Number.isFinite(numeric)) {
    return DEFAULT_CYCLE_COUNT;
  }
  return Math.max(1, Math.min(MAX_CYCLE_COUNT, numeric));
}

async function runAll() {
  log("running browser shell soak suite...");

  let suiteArtifact = null;
  try {
    suiteArtifact = await runSoakSuite();
    assert(
      suiteArtifact.status === "pass" && suiteArtifact.cycles_completed === suiteArtifact.cycle_count,
      "browser shell soak suite completed every planned cycle",
      JSON.stringify(
        {
          status: suiteArtifact.status,
          cycles_completed: suiteArtifact.cycles_completed,
          cycle_count: suiteArtifact.cycle_count,
        },
        null,
        2,
      ),
    );
  } catch (error) {
    suiteArtifact = error?.soakArtifact ?? fallbackArtifact(error);
    fail("browser shell soak suite", error?.stack || error?.message || String(error));
  }

  if (!artifact.textContent.trim()) {
    setArtifactText(JSON.stringify(suiteArtifact, null, 2));
  }

  if (failed === 0) {
    summary.textContent = `all browser shell soak tests passed (${passed} assertions)`;
    summary.className = "pass";
  } else {
    summary.textContent =
      `${failed} browser shell soak failure(s), ${passed} assertion(s) passed\n`
      + results.textContent.trim();
    summary.className = "fail";
  }
  publishCiElement("summary", summary.textContent, summary.className);
}

async function runSoakSuite() {
  const seed = effectiveSeed();
  const cycleCount = effectiveCycleCount();
  const remoteModulePath = `example.com/remote/browser-soak-long-${seed}`;
  const iterations = buildIterations(cycleCount);
  const suiteArtifact = {
    suite: "browser-shell-soak-v1",
    seed,
    cycle_count: cycleCount,
    cycles_completed: 0,
    remote_module_path: remoteModulePath,
    replay: {
      page: "web/test-browser-shell-soak.html",
      page_query: `seed=${seed}&cycles=${cycleCount}`,
      browser_page_runner_command:
        "python3 scripts/browser_page_runner.py "
        + `--page web/test-browser-shell-soak.html --page-query "seed=${seed}&cycles=${cycleCount}" `
        + '--element-id summary --artifact-element-id artifact '
        + '--artifact-output /tmp/gowasm-browser-shell-soak-artifact.json '
        + '--expect-substring "all browser shell soak tests passed" '
        + '--reject-substring "FAIL" --timeout-seconds 360',
    },
    cycles: [],
    status: "running",
    started_at: new Date().toISOString(),
  };

  await resetBrowserShellCacheDatabase();
  await resetModuleCacheDatabase();

  let doc = null;
  let activeCycle = null;
  try {
    doc = await loadShellFrame(frame);
    await waitForShellReady(doc);
    recordSuiteState(suiteArtifact, "shell-ready", doc);

    await addWorkspaceFile(doc, "data.txt", iterations[0].data);
    await addWorkspaceFile(doc, "calc/calc.go", calcSource(remoteModulePath));
    await addWorkspaceFile(
      doc,
      "calc/calc_test.go",
      calcTestSource(iterations[0].message, iterations[0].testName),
    );
    await setWorkspaceFileContents(doc, "main.go", formatDriftMainSource(remoteModulePath));
    await setPlainValue(doc, "entry-path-input", "main.go");
    await setPlainValue(doc, "package-target-input", "calc/calc.go");
    recordSuiteState(suiteArtifact, "workspace-prepared", doc);

    for (const iteration of iterations) {
      activeCycle = {
        index: iteration.index,
        version: iteration.version,
        message: iteration.message,
        data: iteration.data,
        test_name: iteration.testName,
        started_at: new Date().toISOString(),
        steps: [],
      };
      suiteArtifact.cycles.push(activeCycle);

      await setInputValue(
        doc,
        "module-roots",
        moduleRootLine(
          remoteModulePath,
          iteration.version,
          moduleBundleFetchUrl({
            modulePath: remoteModulePath,
            version: iteration.version,
            message: iteration.message,
          }),
        ),
      );
      await setWorkspaceFileContents(doc, "data.txt", iteration.data);
      await setWorkspaceFileContents(doc, "calc/calc_test.go", calcTestSource(iteration.message, iteration.testName));
      await setPlainValue(doc, "entry-path-input", "main.go");
      await setPlainValue(doc, "package-target-input", "calc/calc.go");

      await setWorkspaceFileContents(doc, "main.go", lintDriftMainSource(remoteModulePath));
      await clickAndWaitFor(
        doc,
        "lint-button",
        () =>
          control(doc, "status").textContent === "Lint diagnostics received"
            && control(doc, "output").textContent.includes("run Format")
            && control(doc, "output").textContent.includes("never references `strings`"),
        `soak lint cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "lint", doc);
      assert(
        control(doc, "output").textContent.includes("run Format")
          && control(doc, "output").textContent.includes("never references `strings`"),
        `browser shell soak cycle ${iteration.index + 1} reports formatter drift and unused import diagnostics`,
        shellSnapshot(doc),
      );

      await setWorkspaceFileContents(doc, "main.go", formatDriftMainSource(remoteModulePath));
      await clickAndWaitFor(
        doc,
        "format-button",
        () =>
          control(doc, "status").textContent === "Format complete"
            && selectedSource(doc).includes('fmt.Println(greeter.Message() + ":" + string(data))'),
        `soak format cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "format", doc);
      assert(
        selectedSource(doc).includes('\tdata, err := os.ReadFile("data.txt")'),
        `browser shell soak cycle ${iteration.index + 1} reformats the edited workspace source`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "load-modules-button",
        () =>
          control(doc, "module-status").textContent.includes(`${remoteModulePath}@${iteration.version}`)
            && control(doc, "output").textContent.includes("Loaded 1 remote module bundle(s)."),
        `soak module load cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "module-load", doc);
      assert(
        control(doc, "module-status").textContent.includes(`${remoteModulePath}@${iteration.version}`),
        `browser shell soak cycle ${iteration.index + 1} loads the remote module bundle explicitly`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes(`${iteration.message}:${iteration.data}`),
        `soak run cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "run", doc);
      assert(
        control(doc, "output").textContent.includes(`${iteration.message}:${iteration.data}`),
        `browser shell soak cycle ${iteration.index + 1} runs the real shell through the loaded module state`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "test-snippet-button",
        () =>
          control(doc, "status").textContent.includes("Snippet test passed")
            && control(doc, "output").textContent.includes(`${iteration.message}:${iteration.data}`),
        `soak snippet cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "snippet", doc);
      assert(
        control(doc, "output").textContent.includes("Snippet test passed.")
          && control(doc, "output").textContent.includes(`${iteration.message}:${iteration.data}`),
        `browser shell soak cycle ${iteration.index + 1} keeps snippet tests healthy`,
        shellSnapshot(doc),
      );

      await clickAndWaitFor(
        doc,
        "test-package-button",
        () =>
          control(doc, "status").textContent.includes("Package tests passed")
            && control(doc, "output").textContent.includes(iteration.testName),
        `soak package cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "package-test", doc);
      assert(
        control(doc, "output").textContent.includes(`Completed: ${iteration.testName}`),
        `browser shell soak cycle ${iteration.index + 1} keeps package tests healthy`,
        shellSnapshot(doc),
      );

      await setWorkspaceFileContents(doc, "main.go", sleepingMainSource(remoteModulePath, 60000, `cancel-${iteration.index + 1}`));
      click(doc, "run-button");
      await waitFor(
        () => !control(doc, "cancel-button").disabled,
        `soak cancellation cycle ${iteration.index + 1} became available`,
        doc,
      );
      await delay(150);
      click(doc, "cancel-button");
      await waitFor(
        () =>
          control(doc, "status").textContent === "Run cancelled"
            && control(doc, "output").textContent.includes("Execution cancelled."),
        `soak cancellation cycle ${iteration.index + 1} completed`,
        doc,
      );
      recordCycleState(activeCycle, "cancel", doc);
      assert(
        control(doc, "status").textContent === "Run cancelled",
        `browser shell soak cycle ${iteration.index + 1} cancels a paused host wait and keeps the shell responsive`,
        shellSnapshot(doc),
      );

      await setWorkspaceFileContents(doc, "main.go", sleepingMainSource(remoteModulePath, 5, `recovered-${iteration.index + 1}`));
      await clickAndWaitFor(
        doc,
        "run-button",
        () =>
          control(doc, "status").textContent === "Worker responded"
            && control(doc, "output").textContent.includes(`recovered-${iteration.index + 1} ${iteration.message}`),
        `soak recovery cycle ${iteration.index + 1}`,
      );
      recordCycleState(activeCycle, "cancel-recovery", doc);
      assert(
        control(doc, "output").textContent.includes(`recovered-${iteration.index + 1} ${iteration.message}`),
        `browser shell soak cycle ${iteration.index + 1} recovers after cancellation`,
        shellSnapshot(doc),
      );

      click(doc, "refresh-cache-button");
      await waitFor(
        () => moduleCacheEntries(control(doc, "cache-status").textContent) >= 1,
        `soak cache refresh cycle ${iteration.index + 1}`,
        doc,
      );
      recordCycleState(activeCycle, "cache-refresh", doc);
      assert(
        moduleCacheEntries(control(doc, "cache-status").textContent) >= 1,
        `browser shell soak cycle ${iteration.index + 1} refreshes module cache state after repeated module and shell activity`,
        shellSnapshot(doc),
      );

      activeCycle.finished_at = new Date().toISOString();
      suiteArtifact.cycles_completed = iteration.index + 1;
    }

    await runCacheReplayCycles(doc, remoteModulePath, suiteArtifact);

    suiteArtifact.status = "pass";
    suiteArtifact.finished_at = new Date().toISOString();
    setArtifactText(JSON.stringify(suiteArtifact, null, 2));
    return suiteArtifact;
  } catch (error) {
    suiteArtifact.status = "fail";
    suiteArtifact.finished_at = new Date().toISOString();
    suiteArtifact.failure = {
      cycle_index: activeCycle?.index ?? null,
      message: error?.message || String(error),
      stack: error?.stack || null,
      snapshot: doc ? snapshotSummary(doc) : null,
      raw_shell_snapshot: doc ? shellSnapshot(doc) : null,
    };
    setArtifactText(JSON.stringify(suiteArtifact, null, 2));
    error.soakArtifact = suiteArtifact;
    throw error;
  } finally {
    await unloadShellFrame(frame);
    await resetBrowserShellCacheDatabase();
    await resetModuleCacheDatabase();
  }
}

function buildIterations(count) {
  const names = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta"];
  return Array.from({ length: count }, (_, index) => {
    const base = names[index % names.length];
    return {
      index,
      version: `v3.1.${index}`,
      message: `soak-${base}-${index + 1}`,
      data: `${base}-${index + 1}`,
      testName: `TestRemoteGreetingCycle${index + 1}`,
    };
  });
}

async function importBaselineWorkspace(doc, remoteModulePath) {
  await setInputValue(doc, "archive-url-input", createBaselineArchiveDataUrl(remoteModulePath));
  click(doc, "archive-url-import-button");
  await waitFor(
    () =>
      control(doc, "status").textContent.startsWith("Imported")
        && selectedSource(doc).includes("shell soak baseline"),
    "baseline archive import completed",
    doc,
  );
}

async function addWorkspaceFile(doc, path, contents) {
  await setInputValue(doc, "file-path-input", path);
  click(doc, "add-file-button");
  await waitFor(
    () => Array.from(control(doc, "file-list").options).some((option) => option.value === path),
    `workspace file ${path} added`,
    doc,
  );
  await setWorkspaceFileContents(doc, path, contents);
}

async function restoreCachedWorkspace(doc) {
  click(doc, "restore-cached-workspace-button");
  await waitFor(
    () =>
      control(doc, "status").textContent === "Restored cached workspace"
        && control(doc, "output").textContent.includes("Restored from browser cache"),
    "cached workspace restore completed",
    doc,
  );
}

function createBaselineArchiveDataUrl(remoteModulePath) {
  return createArchiveDataUrl([
    {
      path: "browser-shell-soak/go.mod",
      contents: "module example.com/browser-shell-soak\n\ngo 1.21\n",
    },
    {
      path: "browser-shell-soak/main.go",
      contents: baselineMainSource(remoteModulePath),
    },
    {
      path: "browser-shell-soak/data.txt",
      contents: "baseline\n",
    },
    {
      path: "browser-shell-soak/calc/calc.go",
      contents: calcSource(remoteModulePath),
    },
    {
      path: "browser-shell-soak/calc/calc_test.go",
      contents: calcTestSource("soak-alpha-1", "TestRemoteGreetingCycle1"),
    },
  ]);
}

async function runCacheReplayCycles(doc, remoteModulePath, suiteArtifact) {
  const cacheCycles = ["cache-replay-one", "cache-replay-two"];
  suiteArtifact.cache_replay_cycles = [];

  for (const [index, label] of cacheCycles.entries()) {
    const cycleRecord = {
      index,
      label,
      steps: [],
      started_at: new Date().toISOString(),
    };
    suiteArtifact.cache_replay_cycles.push(cycleRecord);

    await setInputValue(doc, "archive-url-input", createCacheArchiveDataUrl(remoteModulePath, label));
    click(doc, "archive-url-import-button");
    await waitFor(
      () =>
        control(doc, "status").textContent.startsWith("Imported")
          && selectedSource(doc).includes(label),
      `cache replay import ${index + 1} completed`,
      doc,
    );
    recordCycleState(cycleRecord, "archive-import", doc);
    assert(
      control(doc, "status").textContent.startsWith("Imported")
        && selectedSource(doc).includes(label),
      `browser shell cache replay cycle ${index + 1} imports a replayable archived workspace`,
      shellSnapshot(doc),
    );

    click(doc, "refresh-cache-button");
    await waitFor(
      () => control(doc, "cache-status").textContent.includes("Imported workspace cache: 1 entry valid"),
      `cache replay refresh ${index + 1}`,
      doc,
    );
    recordCycleState(cycleRecord, "cache-refresh", doc);

    await setWorkspaceFileContents(doc, "main.go", `package main\n\nfunc main() {}\n`);
    await restoreCachedWorkspace(doc);
    recordCycleState(cycleRecord, "cache-restore", doc);
    assert(
      control(doc, "status").textContent === "Restored cached workspace"
        && control(doc, "output").textContent.includes("Restored from browser cache"),
      `browser shell cache replay cycle ${index + 1} restores the imported workspace from browser cache`,
      shellSnapshot(doc),
    );

    cycleRecord.finished_at = new Date().toISOString();
  }
}

function createCacheArchiveDataUrl(remoteModulePath, label) {
  return createArchiveDataUrl([
    {
      path: "browser-shell-soak/go.mod",
      contents: "module example.com/browser-shell-soak\n\ngo 1.21\n",
    },
    {
      path: "browser-shell-soak/main.go",
      contents: baselineMainSource(remoteModulePath, label),
    },
    {
      path: "browser-shell-soak/data.txt",
      contents: `${label}\n`,
    },
    {
      path: "browser-shell-soak/calc/calc.go",
      contents: calcSource(remoteModulePath),
    },
    {
      path: "browser-shell-soak/calc/calc_test.go",
      contents: calcTestSource(label, `Test${label.replace(/[^a-zA-Z0-9]/g, "")}`),
    },
  ]);
}

function baselineMainSource(remoteModulePath, label = "shell soak baseline") {
  return `package main

import (
\t"fmt"
\t"os"
\t"${remoteModulePath}/greeter"
)

func main() {
\tdata, err := os.ReadFile("data.txt")
\tif err != nil {
\t\tpanic(err)
\t}
\tfmt.Println("${label}", greeter.Message(), string(data))
}
`;
}

function lintDriftMainSource(remoteModulePath) {
  return `package main

import (
"fmt"
"os"
\t"strings"
"${remoteModulePath}/greeter"
)

func main() {
data, err := os.ReadFile("data.txt")
if err != nil {
panic(err)
}
fmt.Println(greeter.Message() + ":" + string(data))
}
`;
}

function formatDriftMainSource(remoteModulePath) {
  return `package main

import (
"fmt"
"os"
"${remoteModulePath}/greeter"
)

func main() {
data, err := os.ReadFile("data.txt")
if err != nil {
panic(err)
}
fmt.Println(greeter.Message() + ":" + string(data))
}
`;
}

function sleepingMainSource(remoteModulePath, durationMs, label) {
  const pauseExpr = durationMs > 1000 ? "time.Second" : "time.Millisecond";
  return `package main

import (
\t"fmt"
\t"time"
\t"${remoteModulePath}/greeter"
)

func main() {
\ttime.Sleep(${pauseExpr})
\tfmt.Println("${label}", greeter.Message())
}
`;
}

function calcSource(remoteModulePath) {
  return `package calc

import "${remoteModulePath}/greeter"

func Greeting() string {
\treturn greeter.Message()
}
`;
}

function calcTestSource(expectedGreeting, testName) {
  return `package calc

func ${testName}() {
\tif Greeting() != "${expectedGreeting}" {
\t\tpanic("expected ${expectedGreeting} greeting")
\t}
}
`;
}

function moduleBundleFetchUrl({ modulePath, version, message }) {
  return `data:application/json,${encodeURIComponent(
    JSON.stringify({
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
          path: "greeter/greeter.go",
          contents: `package greeter

func Message() string {
\treturn "${message}"
}
`,
        },
      ],
    }),
  )}`;
}

function moduleRootLine(modulePath, version, fetchUrl) {
  return `${modulePath} ${version} ${fetchUrl}`;
}

function recordSuiteState(suiteArtifact, step, doc) {
  if (!suiteArtifact.suite_steps) {
    suiteArtifact.suite_steps = [];
  }
  suiteArtifact.suite_steps.push({ step, snapshot: snapshotSummary(doc) });
}

function recordCycleState(cycleArtifact, step, doc) {
  cycleArtifact.steps.push({ step, snapshot: snapshotSummary(doc) });
}

function snapshotSummary(doc) {
  return {
    status: control(doc, "status").textContent,
    module_status: control(doc, "module-status").textContent,
    cache_status: control(doc, "cache-status").textContent,
    output_excerpt: excerpt(control(doc, "output").textContent),
    selected_file: control(doc, "file-list").value,
    entry_path: control(doc, "entry-path-input").value,
    package_target: control(doc, "package-target-input").value,
  };
}

function excerpt(text) {
  const normalized = text.replace(/\s+/g, " ").trim();
  return normalized.length <= 240 ? normalized : `${normalized.slice(0, 237)}...`;
}

function moduleCacheEntries(text) {
  const match = text.match(/Module bundle cache: (\d+) entr(?:y|ies) valid/);
  if (!match) {
    return 0;
  }
  return Number.parseInt(match[1], 10);
}

function delay(durationMs) {
  return new Promise((resolve) => self.setTimeout(resolve, durationMs));
}

function fallbackArtifact(error) {
  return {
    suite: "browser-shell-soak-v1",
    status: "fail",
    started_at: new Date().toISOString(),
    finished_at: new Date().toISOString(),
    failure: {
      message: error?.message || String(error),
      stack: error?.stack || null,
    },
  };
}

void runAll();
