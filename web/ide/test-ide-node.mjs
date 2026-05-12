/**
 * Node.js-based IDE tests using jsdom for DOM simulation.
 * Run with: node test-ide-node.mjs
 */
import { JSDOM } from "/tmp/jsdom-test/node_modules/jsdom/lib/api.js";
import { readFileSync } from "fs";
import { createMemoryFS } from "./fs.js";

let passed = 0;
let failed = 0;

function assert(cond, msg) {
  if (cond) {
    passed++;
    console.log("PASS", msg);
  } else {
    failed++;
    console.log("FAIL", msg);
  }
}

function assertEq(a, b, msg) {
  assert(a === b, `${msg} (expected ${JSON.stringify(b)}, got ${JSON.stringify(a)})`);
}

// 1. HTML structure test
const html = readFileSync("./index.html", "utf8");
const dom = new JSDOM(html, { url: "http://localhost/ide", runScripts: "dangerously" });
const doc = dom.window.document;

assert(doc.querySelector("header h1").textContent === "gowasm", "header title");
assert(doc.getElementById("ide-layout"), "ide-layout exists");
assert(doc.getElementById("file-tree"), "file-tree exists");
assert(doc.getElementById("editor-tabs"), "editor-tabs exists");
assert(!doc.getElementById("editor"), "old textarea removed (CodeMirror only)");
assert(!doc.getElementById("build-output"), "build-output pane removed (terminal only)");
assert(doc.getElementById("terminal-output"), "terminal-output exists");
assert(doc.getElementById("run-btn"), "run button exists");
assert(doc.getElementById("build-btn"), "build button exists");
assert(doc.getElementById("test-btn"), "test button exists");
assert(doc.getElementById("format-btn"), "format button exists");
assert(doc.getElementById("vet-btn"), "vet button exists");
assert(doc.getElementById("abort-btn"), "abort button exists");
assert(doc.getElementById("context-menu"), "context-menu exists");
assert(doc.getElementById("modal"), "modal exists");
assert(doc.getElementById("snippets"), "snippets exists");
assert(doc.getElementById("import-zip-btn"), "import zip button exists");
assert(doc.getElementById("export-zip-btn"), "export zip button exists");
assert(doc.getElementById("zip-import-input"), "zip import input exists");

// 2. CSS structure test
const css = readFileSync("./style.css", "utf8");
assert(css.includes("--bg: #1a1a2e"), "CSS has bg color");
assert(css.includes("--accent: #0077aa"), "CSS has accent color");
assert(css.includes(".tree-node"), "CSS has tree-node");
assert(css.includes(".editor-tab"), "CSS has editor-tab");
assert(css.includes(".output-pane"), "CSS has output-pane");
assert(css.includes("@media (max-width: 900px)"), "CSS has responsive breakpoint");

// 3. fs.js MemoryFS tests
const fs = createMemoryFS();
assertEq(fs.files.get("go.mod"), "module example.com/app\n\ngo 1.22\n", "initial go.mod");
assert(fs.files.has("main.go"), "initial main.go exists");

const root = fs.listDir("");
assertEq(root.length, 2, "root has 2 entries");
assertEq(root[0].name, "go.mod", "root[0] is go.mod");
assertEq(root[0].kind, "file", "go.mod is file");
assertEq(root[1].name, "main.go", "root[1] is main.go");

fs.writeFile("foo.go", "package main\n");
assertEq(fs.readFile("foo.go"), "package main\n", "writeFile/readFile");

fs.createDirectory("pkg");
fs.writeFile("pkg/bar.go", "package pkg\n");
assertEq(fs.readFile("pkg/bar.go"), "package pkg\n", "nested file");

const pkgEntries = fs.listDir("pkg");
assertEq(pkgEntries.length, 1, "pkg has 1 entry");
assertEq(pkgEntries[0].name, "bar.go", "pkg entry is bar.go");

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
assertEq(fs2.readFile("a.go"), "// a", "importFiles works");

// 4. Test that fs.js exports the right functions
import * as fsModule from "./fs.js";
assert(typeof fsModule.pickDirectory === "function", "pickDirectory exported");
assert(typeof fsModule.restoreDirectory === "function", "restoreDirectory exported");
assert(typeof fsModule.listDir === "function", "listDir exported");
assert(typeof fsModule.readFile === "function", "readFile exported");
assert(typeof fsModule.writeFile === "function", "writeFile exported");
assert(typeof fsModule.deleteEntry === "function", "deleteEntry exported");
assert(typeof fsModule.createDirectory === "function", "createDirectory exported");
assert(typeof fsModule.renameEntry === "function", "renameEntry exported");
assert(typeof fsModule.createMemoryFS === "function", "createMemoryFS exported");
assert(typeof fsModule.isFileSystemAccessSupported === "function", "isFileSystemAccessSupported exported");

// 5. Verify engine worker security layer accepts "compile"
const securitySrc = readFileSync("../browser-capability-security.js", "utf8");
assert(securitySrc.includes('"compile"'), "browser-capability-security.js accepts compile requests");
assert(securitySrc.includes("case \"compile\":") && securitySrc.includes("validateRequiredString(request.entry_path"), "compile validates entry_path");

// 6. Verify editor.js has dirty/saved indicator logic
const editorSrc = readFileSync("./editor.js", "utf8");
assert(editorSrc.includes("this.saved = new Set()"), "editor tracks saved state");
assert(editorSrc.includes("tab-indicator dirty"), "editor renders dirty indicator");
assert(editorSrc.includes("tab-indicator saved"), "editor renders saved indicator");
assert(editorSrc.includes("setTimeout") && editorSrc.includes("this.saved.delete"), "editor clears saved indicator after timeout");

// 7. Verify CSS has indicator and flex-wrap styles
assert(css.includes("flex-wrap: wrap"), "panel-toolbar allows wrapping");
assert(css.includes(".editor-tab .tab-indicator"), "CSS styles tab indicators");
assert(css.includes(".tab-indicator.dirty"), "CSS styles dirty indicator");
assert(css.includes(".tab-indicator.saved"), "CSS styles saved indicator");

// 8. Verify Run and Build both output to the terminal panel
const mainSrc = readFileSync("./main.js", "utf8");
assert(mainSrc.includes('case "run_result":'), "main.js handles run_result");
assert(mainSrc.includes('case "diagnostics":'), "main.js handles diagnostics (compile)");
assert(!mainSrc.includes("logTerminal("), "no stale logTerminal calls remain");
assert(mainSrc.includes('kind: "compile"'), "Build sends compile request");
assert(mainSrc.includes('kind: "run"'), "Run sends run request");
assert(mainSrc.includes("logOutput(stdout)"), "Run stdout goes through logOutput");
assert(mainSrc.includes("$ go build"), "Build command logged to terminal");
assert(mainSrc.includes("$ go run"), "Run command logged to terminal");

// 9. Verify engine worker has compile in allowed kinds with cache bust
const workerSrc = readFileSync("../engine-worker.js", "utf8");
assert(securitySrc.includes('"compile"'), "engine worker security allows compile");
assert(workerSrc.includes("import(`./browser-capability-security.js${v}`)"), "engine worker uses dynamic import with cache bust");

// 10. Verify engine-worker.js scope fix: handleWorkerMessage is declared at top level
assert(workerSrc.includes("let handleWorkerMessage = null"), "handleWorkerMessage declared at top level");
assert(workerSrc.includes("void handleWorkerMessage(data)"), "message listener calls handleWorkerMessage via top-level closure");
assert(workerSrc.includes("handleWorkerMessage = async function"), "handleWorkerMessage assigned inside IIFE");

console.log(`\nDone — ${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
