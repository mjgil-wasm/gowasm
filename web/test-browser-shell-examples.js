import { PACKAGED_EXAMPLES } from "./browser-shell-example-catalog.js";
import {
  click,
  control,
  loadShellFrame,
  selectedSource,
  shellSnapshot,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testPackagedExamples({ assert, frame, log }) {
  log("\n--- packaged browser examples harness ---");

  try {
    const doc = await loadShellFrame(frame);
    await waitForShellReady(doc);
    await waitFor(
      () =>
        control(doc, "cache-status").textContent.includes(
          `Example project cache: ${PACKAGED_EXAMPLES.length} entries valid`,
        ),
      "packaged example cache seeded",
      doc,
    );
    assert(
      control(doc, "cache-status").textContent.includes(
        `Example project cache: ${PACKAGED_EXAMPLES.length} entries valid`,
      ),
      "packaged examples seed the mirrored example-project cache store",
      shellSnapshot(doc),
    );

    for (const example of PACKAGED_EXAMPLES) {
      await loadPackagedExample(doc, example);
      assert(
        control(doc, "status").textContent === `Loaded packaged example: ${example.title}`
          && control(doc, "workspace-dirty-status").textContent.includes("matches the last imported or bootstrapped baseline"),
        `packaged example ${example.id} loads through the real browser shell`,
        shellSnapshot(doc),
      );

      switch (example.verify.action) {
        case "run":
          await clickAndVerifyRun(doc, example);
          break;
        case "test_package":
          await clickAndVerifyPackageTests(doc, example);
          break;
        case "format":
          await clickAndVerifyFormat(doc, example);
          break;
        default:
          throw new Error(`unsupported packaged example verify action ${example.verify.action}`);
      }
    }
  } finally {
    await unloadShellFrame(frame);
  }
}

async function loadPackagedExample(doc, example) {
  const select = control(doc, "packaged-example-select");
  select.value = example.id;
  select.dispatchEvent(new doc.defaultView.Event("change", { bubbles: true }));
  await waitFor(
    () => control(doc, "packaged-example-summary").textContent.includes(example.title),
    `packaged example ${example.id} selected`,
    doc,
  );
  click(doc, "load-example-button");
  await waitFor(
    () =>
      control(doc, "status").textContent === `Loaded packaged example: ${example.title}`
        && control(doc, "editor-file-label").textContent === (example.selectedFilePath || example.entryPath),
    `packaged example ${example.id} loaded`,
    doc,
  );
}

async function clickAndVerifyRun(doc, example) {
  click(doc, "run-button");
  await waitFor(
    () =>
      control(doc, "status").textContent === "Worker responded"
        && example.verify.outputIncludes.every((value) => control(doc, "output").textContent.includes(value)),
    `packaged example ${example.id} run completed`,
    doc,
  );
}

async function clickAndVerifyPackageTests(doc, example) {
  click(doc, "test-package-button");
  await waitFor(
    () =>
      control(doc, "status").textContent.includes("Package tests passed")
        && example.verify.outputIncludes.every((value) => control(doc, "output").textContent.includes(value)),
    `packaged example ${example.id} package test completed`,
    doc,
  );
}

async function clickAndVerifyFormat(doc, example) {
  click(doc, "format-button");
  await waitFor(
    () =>
      control(doc, "status").textContent === "Format complete"
        && (example.verify.sourceIncludes ?? []).every((value) => selectedSource(doc).includes(value)),
    `packaged example ${example.id} formatting completed`,
    doc,
  );
}
