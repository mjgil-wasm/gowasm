import {
  loadModuleGraphBundles,
  resetModuleCacheDatabase,
} from "./test-worker-modules.js";
import {
  moduleBundlesToWorkspaceFiles,
  moduleCacheSourcePath,
  parseModuleGraphRoots,
} from "./browser-workspace.js";

export async function testModuleAndToolingEndToEnd({
  assert,
  createWorker,
  log,
  sendAndWait,
}) {
  log("\n--- module and tooling end-to-end harness ---");
  await resetModuleCacheDatabase();

  const seed = Date.now();
  const remoteModulePath = `example.com/remote/tooling-${seed}`;
  const remoteModuleVersion = "v1.2.3";
  const unusedModulePath = `example.com/unused/tooling-${seed}`;
  const unusedModuleVersion = "v0.9.0";
  const parsedRoots = parseModuleGraphRoots(`
# comment
${remoteModulePath} ${remoteModuleVersion} https://example.invalid/remote.json
${unusedModulePath} ${unusedModuleVersion} https://example.invalid/unused.json
`);
  assert(
    parsedRoots.errors.length === 0 && parsedRoots.modules.length === 2,
    "shared module-root parser accepts two configured module roots",
    `got: ${JSON.stringify(parsedRoots)}`,
  );
  assert(
    parsedRoots.modules[0]?.module_path === remoteModulePath &&
      parsedRoots.modules[1]?.version === unusedModuleVersion,
    "shared module-root parser preserves module order and fields",
    `got: ${JSON.stringify(parsedRoots.modules)}`,
  );

  const duplicateRoots = parseModuleGraphRoots(`
${remoteModulePath} ${remoteModuleVersion} https://example.invalid/remote.json
${remoteModulePath} ${remoteModuleVersion} https://example.invalid/remote-duplicate.json
`);
  assert(
    duplicateRoots.errors.length === 1 && duplicateRoots.errors[0]?.includes("duplicate module root"),
    "shared module-root parser rejects duplicate module roots",
    `got: ${JSON.stringify(duplicateRoots)}`,
  );

  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });

    const bundles = await loadModuleGraphBundles({
      worker,
      sendAndWait,
      modules: [
        {
          module_path: remoteModulePath,
          version: remoteModuleVersion,
          fetch_url: moduleBundleFetchUrl({
            modulePath: remoteModulePath,
            version: remoteModuleVersion,
            files: [
              {
                path: "go.mod",
                contents: `module ${remoteModulePath}\n\ngo 1.21\n`,
              },
              {
                path: "greeter/greeter.go",
                contents: `package greeter

func Message() string {
\treturn "remote"
}
`,
              },
            ],
          }),
        },
        {
          module_path: unusedModulePath,
          version: unusedModuleVersion,
          fetch_url: moduleBundleFetchUrl({
            modulePath: unusedModulePath,
            version: unusedModuleVersion,
            files: [
              {
                path: "go.mod",
                contents: `module ${unusedModulePath}\n\ngo 1.21\n`,
              },
              {
                path: "ghost/ghost.go",
                contents: `package ghost

func Label() string {
\treturn "ghost"
}
`,
              },
            ],
          }),
        },
      ],
    });
    assert(
      bundles.length === 2,
      "module graph load returns both requested bundles",
      `got: ${JSON.stringify(bundles)}`,
    );

    let files = moduleBundlesToWorkspaceFiles(baseWorkspaceFiles(remoteModulePath), bundles);
    assert(
      files.some(
        (file) =>
          file.path ===
          moduleCacheSourcePath(remoteModulePath, remoteModuleVersion, "greeter/greeter.go"),
      ),
      "shared module projection rewrites remote sources into __module_cache__ paths",
      `got: ${JSON.stringify(files)}`,
    );

    const lintResult = await sendAndWait(worker, { kind: "lint", files }, 15000);
    assert(
      lintResult.kind === "lint_result",
      "combined harness lint step produces lint_result",
      `got: ${JSON.stringify(lintResult)}`,
    );
    assert(
      Array.isArray(lintResult.diagnostics) && lintResult.diagnostics.length === 1,
      "combined harness lint step reports one formatter-drift warning",
      `got: ${JSON.stringify(lintResult.diagnostics)}`,
    );
    assert(
      lintResult.diagnostics?.[0]?.file_path === "main.go",
      "combined harness lint step reports drift on main.go",
      `got: ${JSON.stringify(lintResult.diagnostics?.[0])}`,
    );
    assert(
      lintResult.diagnostics?.[0]?.position?.line === 10
        && lintResult.diagnostics?.[0]?.position?.column === 1,
      "combined harness lint step reports the first differing source position",
      `got: ${JSON.stringify(lintResult.diagnostics?.[0])}`,
    );

    const formatResult = await sendAndWait(worker, { kind: "format", files }, 15000);
    assert(
      formatResult.kind === "format_result",
      "combined harness format step produces format_result",
      `got: ${JSON.stringify(formatResult)}`,
    );
    assert(
      Array.isArray(formatResult.diagnostics) && formatResult.diagnostics.length === 0,
      "combined harness format step returns no diagnostics",
      `got: ${JSON.stringify(formatResult.diagnostics)}`,
    );
    const formattedMain = formatResult.files?.find((file) => file.path === "main.go")?.contents;
    assert(
      formattedMain?.includes("func joinValue[T any](value T) string {"),
      "combined harness format step preserves supported generic helpers",
      `got: ${JSON.stringify(formattedMain)}`,
    );
    assert(
      formattedMain?.includes("\t// normalized comment"),
      "combined harness format step keeps supported line comments while reindenting generic helpers",
      `got: ${JSON.stringify(formattedMain)}`,
    );
    assert(
      formattedMain?.includes('\tdata, err := os.ReadFile("data.txt")'),
      "combined harness format step rewrites main.go indentation",
      `got: ${JSON.stringify(formattedMain)}`,
    );

    files = formatResult.files;

    const snippetResult = await sendAndWait(
      worker,
      {
        kind: "test_snippet",
        entry_path: "main.go",
        files,
      },
      15000,
    );
    assert(
      snippetResult.kind === "test_result",
      "combined harness snippet step produces test_result",
      `got: ${JSON.stringify(snippetResult)}`,
    );
    assert(
      snippetResult.runner === "snippet" && snippetResult.passed === true,
      "combined harness snippet step passes",
      `got: ${JSON.stringify(snippetResult)}`,
    );
    assert(
      snippetResult.stdout === "remote:alpha\n",
      "combined harness snippet step sees remote module and workspace data",
      `got: ${JSON.stringify(snippetResult.stdout)}`,
    );

    const packageResult = await sendAndWait(
      worker,
      {
        kind: "test_package",
        target_path: "calc/calc.go",
        files,
      },
      15000,
    );
    assert(
      packageResult.kind === "test_result",
      "combined harness package step produces test_result",
      `got: ${JSON.stringify(packageResult)}`,
    );
    assert(
      packageResult.runner === "package" && packageResult.passed === true,
      "combined harness package step passes",
      `got: ${JSON.stringify(packageResult)}`,
    );
    assert(
      packageResult.stdout?.includes("PASS TestRemoteGreeting"),
      "combined harness package step runs the generated test runner",
      `got: ${JSON.stringify(packageResult.stdout)}`,
    );

    const reusedFiles = files.map((file) => {
      if (file.path === "data.txt") {
        return {
          ...file,
          contents: "beta",
        };
      }
      if (
        file.path ===
        moduleCacheSourcePath(unusedModulePath, unusedModuleVersion, "ghost/ghost.go")
      ) {
        return {
          ...file,
          contents: "package ghost\n\nfunc Broken(\n",
        };
      }
      return file;
    });

    const runResult = await sendAndWait(
      worker,
      {
        kind: "run",
        entry_path: "main.go",
        files: reusedFiles,
      },
      15000,
    );
    assert(
      runResult.kind === "run_result",
      "combined harness run step produces run_result",
      `got: ${JSON.stringify(runResult)}`,
    );
    assert(
      Array.isArray(runResult.diagnostics) && runResult.diagnostics.length === 0,
      "combined harness incremental run still returns no diagnostics",
      `got: ${JSON.stringify(runResult.diagnostics)}`,
    );
    assert(
      runResult.stdout === "remote:beta\n",
      "combined harness incremental run reuses the cached compile while seeing updated runtime data",
      `got: ${JSON.stringify(runResult)}`,
    );
  } finally {
    worker.terminate();
    await resetModuleCacheDatabase();
  }
}

function baseWorkspaceFiles(remoteModulePath) {
  return [
    {
      path: "go.mod",
      contents: "module example.com/app\n\ngo 1.21\n",
    },
    {
      path: "main.go",
      contents: `package main

import (
\t"fmt"
\t"os"
\t"${remoteModulePath}/greeter"
)

func joinValue[T any](value T) string {
// normalized comment
return fmt.Sprint(value)
}

func main() {
data, err := os.ReadFile("data.txt")
if err != nil {
panic(err)
}
fmt.Println(joinValue(greeter.Message() + ":" + string(data)))
}
`,
    },
    {
      path: "data.txt",
      contents: "alpha",
    },
    {
      path: "calc/calc.go",
      contents: `package calc

import "${remoteModulePath}/greeter"

func Greeting() string {
\treturn greeter.Message()
}
`,
    },
    {
      path: "calc/calc_test.go",
      contents: `package calc

func TestRemoteGreeting() {
\tif Greeting() != "remote" {
\t\tpanic("expected remote greeting")
\t}
}
`,
    },
  ];
}

function moduleBundleFetchUrl({ modulePath, version, files }) {
  return `data:application/json,${encodeURIComponent(
    JSON.stringify({
      module: {
        module_path: modulePath,
        version,
      },
      files,
    }),
  )}`;
}
