import { exportZip, importZip } from "./zip.js";

export async function runIdeZipTests({ assert, log }) {
  log("\n--- IDE zip.js tests ---");

  const files = [
    { path: "go.mod", contents: "module example.com/app\n\ngo 1.22\n" },
    { path: "main.go", contents: 'package main\n\nimport "fmt"\n\nfunc main() {\n\tfmt.Println("hello")\n}\n' },
    { path: "pkg/util.go", contents: "package pkg\n\nfunc Helper() int {\n\treturn 42\n}\n" },
  ];

  const blob = await exportZip(files);
  assert(blob instanceof Blob, "exportZip returns a Blob");
  assert(blob.size > 0, "ZIP blob has content");

  const imported = await importZip(blob);
  assert(imported.length === 3, "importZip returns 3 files");

  const byPath = new Map(imported.map((f) => [f.path, f.contents]));
  assert(byPath.get("go.mod") === files[0].contents, "go.mod contents match");
  assert(byPath.get("main.go") === files[1].contents, "main.go contents match");
  assert(byPath.get("pkg/util.go") === files[2].contents, "pkg/util.go contents match");
}
