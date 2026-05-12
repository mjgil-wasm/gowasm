import { createMemoryFS } from "./fs.js";

export async function runIdeFsTests({ assert, log }) {
  log("\n--- IDE fs.js tests ---");

  const fs = createMemoryFS();

  assert(fs.files.get("go.mod") === "module example.com/app\n\ngo 1.22\n", "initial go.mod");
  assert(fs.files.has("main.go"), "initial main.go exists");

  const root = fs.listDir("");
  assert(root.length === 2, "root has 2 entries");
  assert(root[0].name === "go.mod", "root[0] is go.mod");
  assert(root[0].kind === "file", "go.mod is file");
  assert(root[1].name === "main.go", "root[1] is main.go");

  const mainContent = fs.readFile("main.go");
  assert(mainContent.includes("package main"), "main.go contains package main");

  fs.writeFile("foo.go", "package main\n");
  assert(fs.readFile("foo.go") === "package main\n", "writeFile/readFile roundtrip");

  fs.createDirectory("pkg");
  assert(fs.listDir("pkg").length === 0, "empty new directory");

  fs.writeFile("pkg/bar.go", "package pkg\n");
  assert(fs.readFile("pkg/bar.go") === "package pkg\n", "nested file");

  const pkgEntries = fs.listDir("pkg");
  assert(pkgEntries.length === 1, "pkg has 1 entry");
  assert(pkgEntries[0].name === "bar.go", "pkg entry is bar.go");

  fs.deleteEntry("foo.go");
  assert(!fs.files.has("foo.go"), "foo.go deleted");

  fs.deleteEntry("pkg");
  assert(!fs.files.has("pkg/bar.go"), "pkg/bar.go deleted recursively");

  fs.renameEntry("main.go", "cmd/main.go");
  assert(!fs.files.has("main.go"), "old path gone after rename");
  assert(fs.files.has("cmd/main.go"), "new path exists after rename");

  const exported = fs.exportFiles();
  assert(exported.some((f) => f.path === "go.mod"), "export includes go.mod");
  assert(exported.some((f) => f.path === "cmd/main.go"), "export includes cmd/main.go");

  const fs2 = createMemoryFS();
  fs2.importFiles([{ path: "a.go", contents: "// a" }]);
  assert(fs2.readFile("a.go") === "// a", "importFiles works");
}
