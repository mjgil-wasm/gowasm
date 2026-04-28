import {
  findWorkspaceFile,
  isGoWorkspacePath,
  isSupportedEditableWorkspacePath,
  normalizeWorkspacePath,
} from "./browser-workspace.js";

export function resolveEntryPath(entryPathValue) {
  return normalizeWorkspacePath(entryPathValue) || "main.go";
}

export function resolvePackageTargetPath(packageTargetValue, selectedWorkspaceFile, entryPathValue) {
  const configuredPath = normalizeWorkspacePath(packageTargetValue);
  if (configuredPath) {
    return configuredPath;
  }
  if (selectedWorkspaceFile?.path.endsWith(".go")) {
    return selectedWorkspaceFile.path;
  }
  return resolveEntryPath(entryPathValue);
}

export function resolveSelectedDisplayFile(displayFiles, selectedPath) {
  return findWorkspaceFile(displayFiles, selectedPath);
}

export function resolveSelectedWorkspaceFile(workspaceFiles, selectedPath) {
  return findWorkspaceFile(workspaceFiles, selectedPath);
}

export function buildDirtyWorkspaceState(files, baselineFiles) {
  const currentFiles = files ?? [];
  const baselineMap = new Map((baselineFiles ?? []).map((file) => [file.path, file.contents]));
  const currentMap = new Map(currentFiles.map((file) => [file.path, file.contents]));
  const dirtyPaths = new Set();
  const removedPaths = [];

  for (const file of currentFiles) {
    if (!baselineMap.has(file.path) || baselineMap.get(file.path) !== file.contents) {
      dirtyPaths.add(file.path);
    }
  }
  for (const [path] of baselineMap) {
    if (!currentMap.has(path)) {
      removedPaths.push(path);
    }
  }

  return {
    currentDirtyCount: dirtyPaths.size,
    dirtyPaths,
    removedDirtyCount: removedPaths.length,
    removedPaths,
    totalDirtyCount: dirtyPaths.size + removedPaths.length,
  };
}

export function describeWorkspaceDirtyState(dirtyState) {
  if (!dirtyState || dirtyState.totalDirtyCount === 0) {
    return "Workspace matches the last imported or bootstrapped baseline.";
  }

  const parts = [];
  if (dirtyState.currentDirtyCount > 0) {
    parts.push(`${dirtyState.currentDirtyCount} current file(s) differ from baseline`);
  }
  if (dirtyState.removedDirtyCount > 0) {
    parts.push(`${dirtyState.removedDirtyCount} baseline file(s) were removed`);
  }
  return `Dirty workspace: ${parts.join("; ")}.`;
}

export function describeWorkspaceSelectionNotice({ selectedFile, selectedWorkspaceFile }) {
  if (!selectedFile) {
    return "Select a workspace file to edit, rename, or target for run and package-test actions.";
  }
  if (!selectedWorkspaceFile) {
    return "Projected __module_cache__/... files are read-only host-owned module views and cannot be renamed or removed.";
  }

  const warnings = [];
  if (!isSupportedEditableWorkspacePath(selectedWorkspaceFile.path)) {
    warnings.push(
      "This path falls outside the supported editable file-type slice and may not round-trip through archive import.",
    );
  }
  if (!isGoWorkspacePath(selectedWorkspaceFile.path)) {
    warnings.push(
      "This file is not Go source. Quick run/package target controls stay disabled, and format/lint ignore it.",
    );
  }

  if (warnings.length === 0) {
    return "Editable workspace file selected.";
  }
  return warnings.join("\n");
}

export function renderWorkspaceSidebar(ctx) {
  syncWorkspaceSelect(ctx);
  renderWorkspaceTree(ctx);
}

export function renderWorkspaceChrome(ctx) {
  const dirtyState = buildDirtyWorkspaceState(ctx.editableFiles, ctx.baselineFiles);
  const selectionNote = describeWorkspaceSelectionNotice({
    selectedFile: ctx.selectedDisplayFile,
    selectedWorkspaceFile: ctx.selectedWorkspaceFile,
  });

  ctx.dirtyStatusElement.textContent = describeWorkspaceDirtyState(dirtyState);
  ctx.selectionNoteElement.textContent = selectionNote;
  renderWorkspaceSidebar({
    dirtyState,
    disableSelection: ctx.disableSelection,
    displayFiles: ctx.displayFiles,
    editablePathSet: new Set(ctx.editableFiles.map((file) => file.path)),
    entryPath: ctx.entryPath,
    onSelectPath: ctx.onSelectPath,
    packageTargetPath: ctx.packageTargetPath,
    selectElement: ctx.selectElement,
    selectedFile: ctx.selectedDisplayFile,
    selectedPath: ctx.selectedPath,
    treeElement: ctx.treeElement,
  });
  syncRenamePathInput({
    activeElement: ctx.activeElement,
    lastSuggestedRenamePath: ctx.lastSuggestedRenamePath,
    renamePathInput: ctx.renamePathInput,
    selectedWorkspaceFile: ctx.selectedWorkspaceFile,
  });
}

export function syncRenamePathInput(ctx) {
  const selectedPath = ctx.selectedWorkspaceFile?.path ?? "";
  if (!selectedPath) {
    if (ctx.renamePathInput !== ctx.activeElement) {
      ctx.renamePathInput.value = "";
    }
    ctx.lastSuggestedRenamePath.value = "";
    return;
  }

  const shouldReplace =
    ctx.renamePathInput !== ctx.activeElement
      || !ctx.renamePathInput.value
      || normalizeWorkspacePath(ctx.renamePathInput.value) === ctx.lastSuggestedRenamePath.value;
  if (shouldReplace) {
    ctx.renamePathInput.value = selectedPath;
  }
  ctx.lastSuggestedRenamePath.value = selectedPath;
}

function syncWorkspaceSelect(ctx) {
  const previousValue = ctx.selectElement.value;
  ctx.selectElement.replaceChildren();
  for (const file of ctx.displayFiles) {
    const option = document.createElement("option");
    option.value = file.path;
    option.textContent = file.path;
    if (file.path === previousValue || file.path === ctx.selectedPath) {
      option.selected = true;
    }
    ctx.selectElement.append(option);
  }
  if (ctx.selectedFile) {
    ctx.selectElement.value = ctx.selectedFile.path;
  }
}

function renderWorkspaceTree(ctx) {
  const root = buildWorkspaceTree(ctx.displayFiles);
  ctx.treeElement.replaceChildren();

  const content = document.createElement("div");
  content.className = "workspace-tree-root";

  appendTreeChildren(content, root, ctx);
  if (!content.childNodes.length) {
    const empty = document.createElement("p");
    empty.className = "note";
    empty.textContent = "Workspace is empty.";
    content.append(empty);
  }

  ctx.treeElement.append(content);
}

function buildWorkspaceTree(files) {
  const root = {
    folders: new Map(),
    files: [],
  };

  for (const file of files.slice().sort((left, right) => left.path.localeCompare(right.path))) {
    const segments = file.path.split("/");
    let node = root;
    for (const segment of segments.slice(0, -1)) {
      if (!node.folders.has(segment)) {
        node.folders.set(segment, {
          folders: new Map(),
          files: [],
          path: node.path ? `${node.path}/${segment}` : segment,
        });
      }
      node = node.folders.get(segment);
    }
    node.files.push({
      name: segments[segments.length - 1] ?? file.path,
      path: file.path,
    });
  }

  return root;
}

function appendTreeChildren(parent, node, ctx) {
  const folderNames = Array.from(node.folders.keys()).sort((left, right) => left.localeCompare(right));
  for (const folderName of folderNames) {
    const childNode = node.folders.get(folderName);
    const details = document.createElement("details");
    details.className = "workspace-folder";
    details.open = folderContainsSelection(childNode, ctx.selectedPath);

    const summary = document.createElement("summary");
    summary.textContent = folderName;
    details.append(summary);

    const children = document.createElement("div");
    children.className = "workspace-folder-children";
    appendTreeChildren(children, childNode, ctx);
    details.append(children);
    parent.append(details);
  }

  for (const file of node.files) {
    parent.append(createWorkspaceFileButton(file, ctx));
  }
}

function createWorkspaceFileButton(file, ctx) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = file.path === ctx.selectedPath
    ? "workspace-file-button workspace-file-button-selected"
    : "workspace-file-button secondary";
  button.dataset.path = file.path;
  button.disabled = ctx.disableSelection;
  button.addEventListener("click", () => {
    ctx.onSelectPath(file.path);
  });

  const label = document.createElement("span");
  label.className = "workspace-file-label";
  label.textContent = file.name;
  button.append(label);

  if (ctx.dirtyState.dirtyPaths.has(file.path)) {
    button.append(createBadge("dirty"));
  }
  if (file.path === ctx.entryPath) {
    button.append(createBadge("entry"));
  }
  if (file.path === ctx.packageTargetPath) {
    button.append(createBadge("pkg"));
  }
  if (!ctx.editablePathSet.has(file.path)) {
    button.append(createBadge("projected"));
  }

  return button;
}

function createBadge(text) {
  const badge = document.createElement("span");
  badge.className = "workspace-badge";
  badge.textContent = text;
  return badge;
}

function folderContainsSelection(node, selectedPath) {
  if (!selectedPath) {
    return true;
  }
  if (node.files.some((file) => file.path === selectedPath)) {
    return true;
  }
  return Array.from(node.folders.values()).some((child) => folderContainsSelection(child, selectedPath));
}
