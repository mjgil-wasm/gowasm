# Browser Capability Security Model

The parked-state browser surface is intentionally narrow. This document freezes
the reviewed host-boundary rules for fetch, storage, archive import, module
cache, worker messages, Wasm bridge buffers, and browser-side diagnostic
rendering.

## Security Boundary

- The Rust engine never touches browser-owned network, storage, or DOM APIs
  directly. Those operations stay behind the worker/browser capability bridge.
- The browser shell treats every fetched archive, boot manifest, module bundle,
  cache record, and worker reply as untrusted input until the relevant
  path/shape checks pass.
- Browser-shell output and source-link rendering stay plain-text and DOM-node
  based. No browser-facing diagnostic path uses `innerHTML` for worker or
  compiler content.

## Reviewed Surfaces

### Archive Import

- Uploaded and fetched project archives stay limited to slash-normalized UTF-8
  text files.
- Absolute paths, traversal entries, empty normalized paths, and reserved
  `__module_cache__/...` paths are rejected before workspace replacement.
- Unsupported file types and binary-looking text are rejected instead of being
  partially unpacked.

### Boot Manifest And Browser Fetch

- Boot-manifest fetch remains browser-owned and is never initiated by the
  engine.
- Remote boot-manifest autoload still requires explicit `boot_consent=1`.
- Ordinary archive and boot fetches use the browser `fetch` boundary and
  surface ordinary shell diagnostics on failure instead of mutating the
  workspace partially.

### Module Fetch And Module Cache

- Fetched module bundles now normalize every module-relative file path and
  reject absolute, traversal, empty, and reserved projected paths before the
  bundle is accepted.
- Persisted module-cache records are shape-validated before reuse; poisoned
  records are treated as cache misses rather than replayed into
  `__module_cache__/...`.
- The browser cache panel uses the same safe-file checks when reporting module
  cache validity, so poisoned persisted records show up as stale instead of
  valid.

### Worker Messages

- The worker now rejects non-object request payloads and unsupported request
  kinds before loading the engine or touching the Wasm bridge.
- Execution requests validate their top-level required fields, and
  `load_module_graph` validates every requested module root shape before the
  worker continues.
- Unsupported or malformed worker requests return ordinary `fatal` protocol
  responses instead of throwing through the worker boundary.

### Wasm Bridge Buffers

- The worker now bounds-checks the engine response pointer/length pair against
  the current `WebAssembly.Memory` buffer before constructing a view.
- Out-of-range response windows fail with a deterministic bridge error instead
  of relying on an unchecked typed-array construction.

### Diagnostic And Source-Link Rendering

- Diagnostic text is rendered through `textContent`.
- Source-link headings and buttons are created as DOM nodes with `textContent`
  labels rather than injected HTML.
- Hostile-looking diagnostic payloads therefore stay inert text in both the
  output pane and the source-link panel.

## Checked Regressions

The browser security contract is pinned by:

- `web/test-browser-capability-security.html`
- `scripts/check-browser-shell.sh`

That harness currently covers:

- archive traversal rejection
- malformed worker message rejection
- module-cache poisoning rejection
- Wasm response-buffer bounds checks
- XSS-like diagnostic/source-link rendering staying plain text

Local reproduction:

```bash
bash scripts/check-browser-shell.sh
```
