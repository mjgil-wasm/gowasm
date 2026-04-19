import { describeModuleStatus, formatDiagnostics, formatTestResult } from "./browser-shell-output.js";
import {
  collectStructuredIssues,
  createHostIssue,
  createToolingIssue,
  renderStructuredIssuePanel,
} from "./browser-shell-diagnostic-ui.js";

export function testStructuredTestResultFormatting({ assert }) {
  const rendered = formatTestResult(
    "package",
    false,
    "RUN TestAdd\nPASS TestAdd\nRUN TestFail\n",
    [],
    {
      subject_path: "calc.go",
      planned_tests: ["TestAdd", "TestFail", "TestAfter"],
      completed_tests: ["TestAdd"],
      active_test: "TestFail",
    },
  );

  assert(
    rendered.includes("Target: calc.go"),
    "formatted test result includes the package target path",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Completed: TestAdd"),
    "formatted test result includes completed tests",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Stopped in: TestFail"),
    "formatted test result includes the active failing test",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Not run: TestAfter"),
    "formatted test result includes not-run tests",
    `got: ${JSON.stringify(rendered)}`,
  );
}

export function testSnippetResultFormatting({ assert }) {
  const rendered = formatTestResult("snippet", false, "", [], {
    subject_path: "main.go",
    planned_tests: ["main.go"],
    completed_tests: [],
    active_test: "main.go",
  });

  assert(
    rendered.includes("Snippet test failed."),
    "formatted snippet result includes the snippet summary",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Entry: main.go"),
    "formatted snippet result includes the snippet entry path",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Stopped in: main.go"),
    "formatted snippet result includes the active snippet entry",
    `got: ${JSON.stringify(rendered)}`,
  );
}

export function testModuleStatusRetryFormatting({ assert }) {
  const rendered = describeModuleStatus({
    modules: [
      {
        module_path: "example.com/remote/tooling",
        version: "v1.2.3",
        fetch_url: "https://example.invalid/module.json",
      },
    ],
    errors: [],
    isLoading: false,
    requestedModuleCount: 1,
    loadedBundles: [
      {
        module: {
          module_path: "example.com/remote/tooling",
          version: "v1.2.3",
        },
        files: [],
      },
    ],
    loadedBundlesStale: true,
    configuredModulesMatchLoaded: false,
    lastLoadError: "fetch failed with 500 Internal Server Error",
  });

  assert(
    rendered.includes("Last module load failed: fetch failed with 500 Internal Server Error"),
    "module status includes the last load failure",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("Previously loaded bundles are stale; retry Load Modules or run/test to refresh them."),
    "module status includes the retry hint for stale bundles",
    `got: ${JSON.stringify(rendered)}`,
  );
}

export function testDiagnosticExcerptAndSuggestionFormatting({ assert }) {
  const rendered = formatDiagnostics([
    {
      message: "undefined: undeclaredFunction\n--> main.go:3:3\n3 |   undeclaredFunction()\n      ^^^^^^^^^^^^^^^^^^",
      severity: "error",
      source_excerpt: {
        line: 3,
        text: "  undeclaredFunction()",
        highlight_start_column: 3,
        highlight_end_column: 20,
      },
      suggested_action: "Fix the source error and compile again.",
    },
  ]);

  assert(
    rendered.includes("undefined: undeclaredFunction"),
    "formatted diagnostics keep the leading compile message",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("3 |   undeclaredFunction()"),
    "formatted diagnostics render the structured source excerpt",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("suggestion: Fix the source error and compile again."),
    "formatted diagnostics render suggested actions",
    `got: ${JSON.stringify(rendered)}`,
  );
}

export function testRuntimeDiagnosticFormatting({ assert }) {
  const rendered = formatDiagnostics([
    {
      message: "division by zero in function `explode`",
      severity: "error",
      source_excerpt: {
        line: 6,
        text: "    _ = 1 / value",
        highlight_start_column: 5,
        highlight_end_column: 16,
      },
      suggested_action: "Increase the instruction budget or reduce work per run.",
      runtime: {
        root_message: "division by zero in function `explode`",
        stack_trace: [
          {
            function: "explode",
            source_location: { path: "main.go", line: 6, column: 5 },
          },
          {
            function: "main",
            source_location: { path: "main.go", line: 10, column: 2 },
          },
        ],
      },
    },
  ]);

  assert(
    rendered.includes("division by zero in function `explode`"),
    "runtime diagnostics render the root message",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("stack trace:\n  at explode (main.go:6:5)\n  at main (main.go:10:2)"),
    "runtime diagnostics render structured stack traces",
    `got: ${JSON.stringify(rendered)}`,
  );
  assert(
    rendered.includes("suggestion: Increase the instruction budget or reduce work per run."),
    "runtime diagnostics render suggestions before the stack trace",
    `got: ${JSON.stringify(rendered)}`,
  );
}

export function testStructuredIssueCollection({ assert }) {
  const issues = collectStructuredIssues({
    diagnostics: [
      {
        message: "undefined: undeclaredFunction",
        severity: "error",
        category: "compile_error",
        file_path: "main.go",
        position: { line: 4, column: 2 },
        source_excerpt: {
          line: 4,
          text: "\tundeclaredFunction()",
          highlight_start_column: 2,
          highlight_end_column: 19,
        },
        suggested_action: "Fix the source error and compile again.",
      },
      {
        message: "division by zero",
        severity: "error",
        category: "runtime_trap",
        file_path: "main.go",
        position: { line: 6, column: 5 },
        runtime: {
          root_message: "division by zero",
          category: "runtime_trap",
          stack_trace: [
            {
              function: "explode",
              source_location: { path: "main.go", line: 6, column: 5 },
            },
          ],
        },
      },
    ],
    auxiliaryIssues: [
      createToolingIssue("module roots require <module_path> <version> <fetch_url>", {
        suggestedAction: "Fix the module root list and retry Load Modules.",
      }),
    ],
  });

  assert(
    issues.length === 3,
    "structured issue collection keeps diagnostics plus auxiliary issues",
    `got: ${JSON.stringify(issues)}`,
  );
  assert(
    issues[0]?.category === "compile_error" && issues[0]?.location === "main.go:4:2",
    "structured issue collection keeps compile category and location",
    `got: ${JSON.stringify(issues[0])}`,
  );
  assert(
    issues[1]?.stackLines?.[0] === "explode (main.go:6:5)",
    "structured issue collection keeps runtime stack frames",
    `got: ${JSON.stringify(issues[1])}`,
  );
  assert(
    issues[2]?.category === "tooling"
      && issues[2]?.suggestedAction === "Fix the module root list and retry Load Modules.",
    "structured issue collection keeps shell-side suggestions",
    `got: ${JSON.stringify(issues[2])}`,
  );
}

export function testStructuredIssuePanelRendering({ assert }) {
  const panel = document.createElement("section");
  const list = document.createElement("div");
  renderStructuredIssuePanel({
    diagnostics: [
      {
        message: "undefined: undeclaredFunction",
        severity: "error",
        category: "compile_error",
        file_path: "main.go",
        position: { line: 4, column: 2 },
        suggested_action: "Fix the source error and compile again.",
      },
    ],
    auxiliaryIssues: [
      createHostIssue("Worker bridge failed", {
        filePath: "web/engine-worker.js",
        line: 18,
        column: 3,
        stackText: "Error: Worker bridge failed\nat postMessage (web/engine-worker.js:18:3)",
        suggestedAction: "Reload the page and rerun the request.",
      }),
    ],
    listElement: list,
    panelElement: panel,
  });

  assert(
    panel.hidden === false && list.querySelectorAll(".diagnostic-card").length === 2,
    "structured issue panel renders one card per diagnostic or auxiliary issue",
    list.innerHTML,
  );
  assert(
    list.textContent.includes("compile_error") && list.textContent.includes("host_error"),
    "structured issue panel renders explicit category badges",
    list.textContent,
  );
  assert(
    list.textContent.includes("main.go:4:2")
      && list.textContent.includes("web/engine-worker.js:18:3"),
    "structured issue panel renders explicit locations",
    list.textContent,
  );
  assert(
    list.textContent.includes("Reload the page and rerun the request.")
      && list.textContent.includes("postMessage (web/engine-worker.js:18:3)"),
    "structured issue panel renders suggested next steps and stack lines",
    list.textContent,
  );
}
