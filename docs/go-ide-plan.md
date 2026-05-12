# Go IDE HTML Page — Implementation Plan

## Goal

Build a fully browser-based single-page IDE for editing and running Go code, adopting the dark, minimal, panel-based aesthetic of the `zigwasm` reference (`/home/m/git/zigwasm/web/index.html`). The IDE runs entirely in the browser with no backend server: files are managed via the File System Access API (or an in-memory virtual file system fallback), and Go code is compiled to WASM and executed directly in the browser using the project's Go/WASM toolchain.

## Reference Style (zigwasm)

- **Palette**: deep navy background (`#1a1a2e`), panel backgrounds (`#16162a`, `#0f0f1e`), text (`#e0e0e0`), accent blue (`#0077aa`), success (`#66ff99`), error (`#ff6666`).
- **Typography**: `ui-monospace`, `Cascadia Code`, `Fira Code`, monospace; small uppercase panel headers with wide letter-spacing.
- **Layout**: CSS Grid / Flexbox, thin `1px` borders (`#333`, `#2a2a3e`), no heavy chrome.
- **Controls**: flat buttons, primary blue vs. secondary gray, disabled states, status text aligned right in toolbars.
- **Feel**: compiler-demo minimalism — every pixel serves editing, building, or running.

---

## Sequential Thinking — Implementation Order

### Phase 1: Static Shell (HTML/CSS/JS scaffold)

1. **Asset directory**: create `web/ide/`.
2. **index.html**: single-page shell with the zigwasm color palette and panel grid.
   - Header bar: title + subtitle ("Go IDE — edit, build, and run Go in the browser").
   - Three-column layout on desktop, stacked on mobile:
     - **Left panel** (20 %): file explorer tree.
     - **Center panel** (50 %): tabbed editor area.
     - **Right panel** (30 %): bottom split into **Build Output** and **Interactive Terminal**.
   - Footer toolbar: Run / Build / Test / Format buttons + status indicator.
3. **styles.css**: all custom CSS, no external framework, matching zigwasm exactly.
4. **main.js**: module skeleton, DOM refs, helper utilities (`setStatus`, `log`, `clearOutput`, `createTreeNode`).

> **Deliverable**: opening `index.html` in a browser renders a coherent dark IDE shell with empty panels and working layout collapse on narrow widths.

### Phase 2: In-Browser File System & Workspace

5. **File System Access API integration**
   - On first load, prompt the user to select a local directory via `showDirectoryPicker()`.
   - Maintain a virtual directory handle tree in memory; persist the last-used handle in IndexedDB so reopening the IDE restores the workspace (if permission grants remain valid).
   - Fallback for unsupported browsers: an in-memory virtual file system (JS `Map` of paths to file contents) with import/export as a ZIP.
6. **File operations (pure JS)**
   - `listDir(handle)` — iterate `FileSystemDirectoryHandle` entries, build JSON tree.
   - `readFile(handle, path)` — get `File` via `getFileHandle()`, read as text.
   - `writeFile(handle, path, text)` — get writable, truncate, write.
   - `deleteEntry(handle, path)` — `removeEntry()` on parent directory.
   - `goModInit(handle, moduleName)` — create `go.mod` with `module <name>\ngo 1.22`.
7. **Security / sandboxing**
   - All file access is gated by the browser's native permission prompts; the IDE cannot escape the chosen directory.
   - WASM execution runs in a same-origin `Web Worker` or `iframe` with a `Content-Security-Policy` to isolate user code.
   - Stdout/stderr from the compiled Go WASM module is captured via a hooked `console.log` / `fs.writeSync` and capped to 10 MB to prevent tab memory exhaustion.

> **Deliverable**: user selects a folder, the IDE reads the directory tree, and file CRUD works without any network round-trips.

### Phase 3: File Explorer (Left Panel)

7. **Tree rendering**
   - Fetch root on load; lazy-load children on expand.
   - Icons: folder / file glyphs (simple Unicode or inline SVG).
   - Context menu (right-click): New File, New Folder, Delete, Rename.
8. **Interactions**
   - Click file → open tab in center editor.
   - Click folder → toggle expand/collapse.
   - Drag-and-drop reordering (optional future; skip for MVP if complex).

> **Deliverable**: user can navigate the project tree and open files into the editor.

### Phase 4: Tabbed Editor (Center Panel)

9. **Tab bar**
   - One tab per open file; close button on each tab.
   - Unsaved-change indicator (dot or italic title).
   - Keyboard shortcut: `Ctrl/Cmd+W` closes tab; `Ctrl/Cmd+S` saves.
10. **Editor surface**
    - Start with a `<textarea>` (same as zigwasm) for simplicity.
    - Layer **Go syntax highlighting** via a lightweight tokenizer (e.g., custom JS regex highlighter or Prism.js single-file build) over a `contenteditable` or `<pre>` overlay, or adopt CodeMirror 6 in single-file mode if bundle size is acceptable.
    - Line numbers gutter (absolute-positioned `<div>` left of textarea).
    - Tab-key inserts two spaces (Go convention).
    - Auto-bracket pairing for `()`, `[]`, `{}`.
11. **Autosave & dirty tracking**
    - Debounced write to the File System Access API (or in-memory virtual FS) 500 ms after keystroke.
    - Explicit Save button and `Ctrl/Cmd+S` fallback.

> **Deliverable**: user can edit multiple files with syntax highlighting, line numbers, and persistent saves.

### Phase 5: Build / Run / Test Toolbar

12. **Button wiring**
    - **Run** (`go run .` or `go run <main_file>`) — streams to Terminal panel.
    - **Build** (`go build .`) — streams to Build Output panel; on success shows binary name.
    - **Test** (`go test ./... -v`) — streams to Build Output panel; parses `PASS`/`FAIL` for summary badge.
    - **Format** (`gofmt -w` or `go fmt ./...`) — runs in place, then refreshes editor content.
    - **Vet** (`go vet ./...`) — streams warnings to Build Output.
13. **Process lifecycle**
    - Abort button (terminates the Web Worker / WASM runtime iframe).
    - Spinner/disabled state while process is alive.
    - Exit-code coloring (green 0, red non-0).

> **Deliverable**: all five primary Go actions execute and stream output correctly.

### Phase 6: Output Panels (Right Side / Bottom)

14. **Build Output panel**
    - Read-only, `white-space: pre-wrap`, scroll-to-bottom on new lines.
    - Error linker: regex-parse `file.go:12:34:` patterns into clickable links that jump the editor to line 12, column 34.
    - Clear button.
15. **Interactive Terminal panel**
    - Same styling as Build Output but for running program stdin/stdout.
    - If the running Go program expects input, provide an input line at the bottom of the terminal (echo to stdout pane, send to process stdin).

> **Deliverable**: compiler errors are navigable; interactive programs accept stdin.

### Phase 7: Go-Specific IDE Conveniences

16. **go.mod awareness**
    - Parse `go.mod` on load; display module name in header subtitle.
    - If `go.mod` is missing, show a "Initialize Module" button that creates `go.mod` via the File System Access API.
17. **Basic IntelliSense / Snippets**
    - Hard-coded snippets: `func main()`, `func TestX(t *testing.T)`, `package main`, `fmt.Println()`.
    - Triggered by `Ctrl+Space` or typing prefix.
18. **goimports on save** (optional)
    - Run `goimports -w` after `go fmt` if `goimports` is available in `$PATH`.
19. **Test runner tree** (optional)
    - Parse `go test -json` output to show a tree of passed/failed tests per package.

> **Deliverable**: module initialization, snippets, and optional import formatting.

### Phase 8: Go-to-WASM Compilation & Runtime

20. **WASM build pipeline (browser-side)**
    - Integrate the project's Go/WASM compiler (already compiling Go → WASM) as a Web Worker.
    - The worker accepts a message `{ action: "build", files: { "main.go": "...", "go.mod": "..." } }`.
    - The compiler runs entirely in the worker; on success it returns the `.wasm` bytes; on failure it returns the compiler error output.
21. **WASM execution environment**
    - Load the resulting `.wasm` in a sandboxed `<iframe>` or second Web Worker with a custom `wasm_exec.js` bridge.
    - Override `fs` and `syscall/js` imports so that `fmt.Println` writes to the Terminal panel, not the browser console.
    - Provide a mock `net/http` stub that returns a meaningful error ("network unavailable in browser IDE") instead of hanging.
22. **Run lifecycle**
    - **Run** button compiles then instantiates the WASM module; stdout/stderr stream to the Terminal panel.
    - **Abort** button terminates the worker / iframe.
    - **Build Only** mode validates compilation without execution, showing compiler diagnostics in the Build Output panel.

> **Deliverable**: pressing Run compiles and executes Go code entirely in the browser with no server, native binary, or backend process.

---

## File Layout

```
web/ide/
  index.html          — single-page shell
  style.css           — dark panel layout (zigwasm palette)
  main.js             — app shell, DOM refs, panel layout
  fs.js               — File System Access API wrapper + in-memory fallback
  editor.js           — tabbed editor, syntax highlighter, snippets
  compiler-worker.js  — Go-to-WASM compiler running in a Web Worker
  runtime-iframe.html — sandboxed host for compiled WASM modules
  go-mode.js          — lightweight Go syntax highlighter (or CodeMirror 6 single-file)
assets/
  (existing static assets remain unchanged)
```

The IDE is served as static files from `web/ide/` (e.g., via `python -m http.server` or any static host). No backend router or server-side code is required.

---

## UI Component Map (zigwasm style)

| Component | zigwasm Equivalent | Color Tokens |
|-----------|-------------------|--------------|
| App background | `body` | `#1a1a2e` |
| Panel header | `.panel-header` | `#16162a`, text `#888` |
| Editor textarea | `textarea` | `#0f0f1e`, text `#d0d0e0` |
| Output pane | `.output` | `#0f0f1e`, text `#e0e0e0` |
| Primary button | `button` | `#0077aa` → `#0099cc` hover |
| Secondary button | `button.secondary` | `#333` → `#444` hover |
| Status OK | `#status.ok` | `#66ff99` |
| Status Error | `#status.error` | `#ff6666` |
| Borders | `.panel`, `.toolbar` | `#333`, `#2a2a3e` |

---

## Implementation Status

All core phases have been implemented in `web/ide/`:

| Phase | Status | Files |
|-------|--------|-------|
| Phase 1 | ✅ Complete | `index.html`, `style.css` |
| Phase 2 | ✅ Complete | `fs.js` (File System Access API + MemoryFS fallback) |
| Phase 3 | ✅ Complete | File tree rendering in `main.js` |
| Phase 4 | ✅ Complete | `editor.js` (CodeMirror 6, tabs, snippets, line numbers) |
| Phase 5 | ✅ Complete | Toolbar wiring in `main.js` (Run, Build, Test, Format, Vet) |
| Phase 6 | ✅ Complete | Output panels with error link parsing in `main.js` |
| Phase 7 | ✅ Complete | `go.mod` awareness, Initialize Module button, snippets |
| Phase 8 | ✅ Complete | `compiler-worker.js`, `runtime-iframe.html`, engine integration |

### Tests

- `web/ide/test-ide-fs.html` / `test-ide-fs.js` / `test-ide-fs-runner.js` — browser test harness for fs.js
- `web/ide/test-ide-node.mjs` — Node.js/jsdom structural tests (49 assertions, all passing)

### Optional future enhancements

- **goimports on save**: Requires `goimports` binary or a WASM port; deferred.
- **Test runner tree**: Requires parsing `go test -json` output; deferred.
- **Drag-and-drop file reordering**: Optional UX improvement; deferred.
