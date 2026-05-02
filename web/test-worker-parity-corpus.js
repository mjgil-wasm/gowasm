function corpusIndexUrl() {
  return new URL("../testdata/parity-corpus/index.json", import.meta.url);
}

function workspaceFileUrl(caseDef, filePath) {
  return new URL(
    `../testdata/parity-corpus/${caseDef.id}/workspace/${filePath}`,
    import.meta.url,
  );
}

async function loadCorpusIndex() {
  const response = await fetch(corpusIndexUrl());
  if (!response.ok) {
    throw new Error(
      `failed to load parity corpus index: ${response.status} ${response.statusText}`,
    );
  }
  return response.json();
}

async function loadWorkspaceFiles(caseDef) {
  return Promise.all(
    caseDef.workspace_files.map(async (path) => {
      const response = await fetch(workspaceFileUrl(caseDef, path));
      if (!response.ok) {
        throw new Error(
          `failed to load parity corpus file ${caseDef.id}/${path}: ${response.status} ${response.statusText}`,
        );
      }
      return {
        path,
        contents: await response.text(),
      };
    }),
  );
}

export async function testParityCorpusAcrossWorker({
  assert,
  createWorker,
  log,
  sendAndWait,
}) {
  log("\n--- parity corpus across worker ---");

  const index = await loadCorpusIndex();
  assert(index.schema_version === 2, "parity corpus index uses schema version 2");
  assert(
    Array.isArray(index.cases) && index.cases.length > 0,
    "parity corpus index contains representative cases",
    `got: ${JSON.stringify(index)}`,
  );

  for (const caseDef of index.cases) {
    await runCase({ assert, caseDef, createWorker, sendAndWait });
  }
}

async function runCase({ assert, caseDef, createWorker, sendAndWait }) {
  const expected = caseDef.expected_outcomes?.browser_worker;
  if (!expected || expected.status !== "pass") {
    throw new Error(
      `browser_worker parity case ${caseDef.name} is not tracked as passing`,
    );
  }

  const worker = createWorker();
  try {
    await sendAndWait(worker, { kind: "boot" });
    const files = await loadWorkspaceFiles(caseDef);
    const result = await sendAndWait(
      worker,
      {
        kind: "run",
        entry_path: caseDef.entry_path,
        files,
        host_time_unix_millis: caseDef.host_time_unix_millis,
      },
      15000,
    );

    assert(
      result.kind === "run_result",
      `browser worker parity case ${caseDef.name} produces run_result`,
      `got: ${JSON.stringify(result)}`,
    );
    assert(
      Array.isArray(result.diagnostics) && result.diagnostics.length === 0,
      `browser worker parity case ${caseDef.name} returns no diagnostics`,
      `got: ${JSON.stringify(result.diagnostics)}`,
    );
    assert(
      result.stdout === caseDef.expected_stdout,
      `browser worker parity case ${caseDef.name} matches expected stdout`,
      `got: ${JSON.stringify(result.stdout)}`,
    );
  } finally {
    worker.terminate();
  }
}
