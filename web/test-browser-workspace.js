import {
  configuredModuleRootsAreFresh,
  editableWorkspaceFiles,
  isModuleCachePath,
  moduleBundlesToWorkspaceFiles,
  moduleRootsConfigKey,
} from "./browser-workspace.js";

export function testReservedModuleCacheWorkspacePaths({ assert }) {
  assert(
    isModuleCachePath("__module_cache__/example.com/remote/@v1.2.3/tool.go"),
    "browser workspace recognizes reserved module cache paths",
  );
  assert(
    !isModuleCachePath("main.go"),
    "browser workspace does not treat ordinary files as module cache paths",
  );

  const configuredModules = [
    {
      module_path: "example.com/remote",
      version: "v1.2.3",
      fetch_url: "https://example.invalid/module.json",
    },
  ];
  const configuredKey = moduleRootsConfigKey(configuredModules);
  assert(
    configuredModuleRootsAreFresh(configuredModules, configuredKey, false),
    "browser workspace treats matching module roots as fresh when no stale flag is set",
  );
  assert(
    !configuredModuleRootsAreFresh(configuredModules, configuredKey, true),
    "browser workspace does not treat stale loaded bundles as fresh even when roots match",
  );

  const editableFiles = editableWorkspaceFiles([
    { path: "main.go", contents: "package main\n" },
    {
      path: "__module_cache__/example.com/remote/@v1.2.3/tool.go",
      contents: "package tool\n",
    },
  ]);
  assert(
    editableFiles.length === 1 && editableFiles[0]?.path === "main.go",
    "browser workspace keeps module cache files out of the editable file set",
    `got: ${JSON.stringify(editableFiles)}`,
  );

  const executionFiles = moduleBundlesToWorkspaceFiles(editableFiles, [
    {
      module: {
        module_path: "example.com/remote",
        version: "v1.2.3",
      },
      files: [
        {
          path: "tool.go",
          contents: "package tool\n",
        },
      ],
    },
  ]);
  assert(
    executionFiles.some(
      (file) =>
        file.path === "__module_cache__/example.com/remote/@v1.2.3/tool.go",
    ),
    "browser workspace still projects host-owned module bundles into execution files",
    `got: ${JSON.stringify(executionFiles)}`,
  );
}
