import { createWorkspaceSnapshot } from "./browser-shell-snapshot.js";
import {
  formatPackagedExampleSummary,
  getPackagedExample,
  PACKAGED_EXAMPLE_IDS,
  PACKAGED_EXAMPLES,
} from "./browser-shell-example-catalog.js";

export function createPackagedExampleController(ctx) {
  let selectedCatalogId = PACKAGED_EXAMPLE_IDS[0] ?? "";
  let activeExampleId = "";

  function packagedExampleIds() {
    return [...PACKAGED_EXAMPLE_IDS];
  }

  function getSelectedExampleId() {
    return activeExampleId;
  }

  function setSelectedExampleId(nextId) {
    const normalizedId = getPackagedExample(nextId)?.id ?? "";
    activeExampleId = normalizedId;
    selectedCatalogId = normalizedId || PACKAGED_EXAMPLE_IDS[0] || "";
    renderExampleCatalog();
  }

  async function seedPackagedExamples() {
    await ctx.seedPackagedExamples(
      PACKAGED_EXAMPLES.map((example) => ({
        cache_key: `packaged-example:${example.id}`,
        example_id: example.id,
        schema_version: ctx.cacheSchemaVersion,
        snapshot: snapshotForExample(example),
        stored_at_ms: Date.now(),
      })),
    );
  }

  async function loadSelectedExample() {
    if (ctx.isBusy()) {
      return;
    }
    const example = getPackagedExample(selectedCatalogId);
    if (!example) {
      ctx.statusElement.textContent = "Packaged example unavailable";
      ctx.setOutputView("Select a packaged example before loading it.", []);
      return;
    }
    applyExample(example);
  }

  function handleExampleSelectionChange(nextValue) {
    selectedCatalogId = getPackagedExample(nextValue)?.id ?? selectedCatalogId;
    renderExampleCatalog();
  }

  function applyRestoredExampleSelection(restoredSelectedExampleId) {
    if (getPackagedExample(restoredSelectedExampleId)) {
      setSelectedExampleId(restoredSelectedExampleId);
      return;
    }
    setSelectedExampleId("");
  }

  function clearSelectedExample() {
    activeExampleId = "";
    selectedCatalogId = PACKAGED_EXAMPLE_IDS[0] ?? "";
    renderExampleCatalog();
  }

  function renderExampleCatalog() {
    const previousValue = ctx.exampleSelect.value;
    ctx.exampleSelect.replaceChildren();
    for (const example of PACKAGED_EXAMPLES) {
      const option = document.createElement("option");
      option.value = example.id;
      option.textContent = example.title;
      if (example.id === selectedCatalogId || (!selectedCatalogId && example.id === previousValue)) {
        option.selected = true;
      }
      ctx.exampleSelect.append(option);
    }
    if (selectedCatalogId) {
      ctx.exampleSelect.value = selectedCatalogId;
    }

    const selectedExample = getPackagedExample(ctx.exampleSelect.value || selectedCatalogId);
    const summaryLines = selectedExample
      ? [formatPackagedExampleSummary(selectedExample)]
      : ["No packaged example selected."];
    if (activeExampleId && activeExampleId === selectedExample?.id) {
      summaryLines.push("", "Current workspace source: this packaged example is active.");
    } else if (activeExampleId) {
      summaryLines.push("", `Current workspace source: ${activeExampleId}.`);
    } else {
      summaryLines.push("", "Current workspace source: custom/imported workspace.");
    }
    ctx.exampleSummary.textContent = summaryLines.join("\n");
  }

  function syncControls() {
    const hasExample = Boolean(getPackagedExample(selectedCatalogId));
    ctx.exampleSelect.disabled = ctx.isBusy();
    ctx.loadExampleButton.disabled = ctx.isBusy() || !hasExample;
  }

  function applyExample(example) {
    selectedCatalogId = example.id;
    activeExampleId = example.id;
    renderExampleCatalog();
    ctx.setWorkspaceFiles(example.files, example.selectedFilePath || example.entryPath, {
      resetDirtyBaseline: true,
    });
    ctx.entryPathInput.value = example.entryPath || "";
    ctx.packageTargetInput.value = example.packageTarget || "";
    ctx.moduleRootsInput.value = "";
    ctx.resetLoadedModules();
    ctx.statusElement.textContent = `Loaded packaged example: ${example.title}`;
    ctx.setOutputView(
      `${formatPackagedExampleSummary(example)}\n\nWorkspace files:\n${example.files
        .map((file) => `- ${file.path}`)
        .join("\n")}`,
      [],
    );
    ctx.renderModuleStatus();
    ctx.renderWorkspace();
    ctx.syncControls();
  }

  function snapshotForExample(example) {
    return createWorkspaceSnapshot({
      entryPath: example.entryPath,
      loadedModuleBundles: [],
      moduleRootsText: "",
      packagedExampleIds: PACKAGED_EXAMPLE_IDS,
      packageTarget: example.packageTarget,
      selectedExampleId: example.id,
      selectedFilePath: example.selectedFilePath || example.entryPath,
      workspaceFiles: example.files,
    });
  }

  return {
    applyRestoredExampleSelection,
    clearSelectedExample,
    getSelectedExampleId,
    handleExampleSelectionChange,
    loadSelectedExample,
    packagedExampleIds,
    renderExampleCatalog,
    seedPackagedExamples,
    setSelectedExampleId,
    syncControls,
  };
}
