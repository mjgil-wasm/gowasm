# Diagnostics Schema

This document freezes the shared diagnostics payload carried across the
compiler, VM, engine, Wasm worker protocol, browser worker, and browser shell.

## Ownership

- `crates/host-types`: owns the protocol-stable `Diagnostic`,
  `RuntimeDiagnostic`, `Position`, `SourceSpan`, and `SourceExcerpt` schema.
- `crates/compiler`: owns compile-time error contexts, spans, and source file
  selection for compile diagnostics.
- `crates/vm`: owns runtime categories, root-cause text, stack frames,
  instruction/source metadata, and runtime source locations.
- `crates/engine`: assembles worker-facing diagnostics, adds source excerpts
  and suggested actions where the current subset can do so, and normalizes
  wrapped-snippet diagnostics back to the original editable source.
- `web/`: renders the stable schema without string-parsing stack traces or
  excerpts back out of the human-readable message, including the browser-shell
  structured issue-summary panel ahead of the unchanged raw output block.

## Top-Level `Diagnostic`

The shared JSON payload is `gowasm_host_types::Diagnostic`:

- `message`: normalized human-readable diagnostic text.
- `severity`: `error`, `warning`, or `info`.
- `category`: shared `ErrorCategory` value.
- `file_path`: optional user-facing source path.
- `position`: optional 1-based `{ line, column }` primary location.
- `source_span`: optional 1-based `{ start, end }` span in user-facing source.
- `source_excerpt`: optional single-line excerpt with
  `highlight_start_column` and `highlight_end_column`.
- `suggested_action`: optional short next-step hint when the current subset has
  a stable action to recommend.
- `runtime`: optional structured runtime payload for VM-backed failures.

The schema omits absent optional fields on serialize and still accepts older
null-bearing payloads on deserialize.

## `RuntimeDiagnostic`

Runtime-backed failures add `Diagnostic.runtime`:

- `root_message`: root-cause text for the failing runtime condition.
- `category`: shared `ErrorCategory` for the runtime root cause.
- `stack_trace`: ordered leaf-first frames.

Each `RuntimeStackFrame` carries:

- `function`: function name.
- `instruction_index`: VM instruction index in that function.
- `source_span`: optional byte-offset source span.
- `source_location`: optional resolved `{ path, line, column, end_line,
  end_column }`.

The top-level `Diagnostic` may duplicate the first frame's file/position/span
and excerpt so browser tooling can render the primary location without
re-parsing the stack.

## Current Population Rules

- compile diagnostics now populate `category`, `severity`, `file_path`,
  `position`, `source_span`, `source_excerpt`, and the current compile-fix
  suggested action.
- lint warnings now populate `file_path`, `position`, `source_span`,
  `source_excerpt`, and rule-specific suggested actions.
- formatter/test-runner/snippet-runner parse/setup diagnostics now reuse the
  shared schema and attach suggested actions where the boundary is stable.
- runtime panics, traps, and budget/deadlock failures now populate
  `category`, `severity`, `file_path`, `position`, `source_span`,
  `source_excerpt` when the top frame resolves back to source, plus structured
  `runtime.stack_trace`. Budget/deadlock failures also carry the current
  stable suggested actions.
- wrapped snippet compile/runtime diagnostics remap top-level location fields
  and runtime stack locations back to the original snippet lines before the
  payload leaves the engine.

## Checked Evidence

- `crates/host-types/src/tests_core_protocol.rs`
- `crates/engine/src/tests_compile_fail_diagnostics_golden.rs`
- `crates/engine/src/tests_diagnostics_golden.rs`
- `crates/engine/src/tests_diagnostics.rs`
- `web/test-browser-shell-output.js`
- `web/test-worker.js`
- `web/test-browser-shell-error-ui.js`
