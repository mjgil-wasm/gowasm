# Snippet Test Support

Task `077` freezes the current snippet-test runner surface.

## Ownership

- `crates/host-types`
  - owns the `test_snippet` request and `test_result` response schema
- `crates/engine`
  - owns snippet normalization, wrapped-source diagnostic remapping, and
    structured snippet-test result details
- `web/`
  - owns the browser worker and shell paths that submit snippet-test requests,
    render `test_result`, and display snippet cancellation state

## Supported Slice

- full Go entry files continue to run as-is through `test_snippet`
- package-less body snippets are normalized into a generated `package main`
  wrapper when the entry file does not start with a package clause
- leading snippet `import` declarations are lifted ahead of the generated
  `main` wrapper so small tutorial/playground-style snippets can run without a
  hand-written file prologue
- stdout is preserved exactly through `test_result.stdout`
- compile diagnostics and runtime stack/source locations from wrapped snippets
  are remapped back to the original entry path and original snippet line
  numbers
- instruction-budget failures use the same runtime-budget diagnostic category
  and structured `test_result.details` shape as other test failures
- snippet tests share the existing worker capability, pause/resume, and cancel
  path with normal `run` requests, and the browser shell now surfaces snippet
  cancellation with snippet-specific status/output text
- structured result details through:
  - `subject_path`
  - `planned_tests`
  - `completed_tests`
  - `active_test`

## Explicit Limits

- the wrapper path is intentionally narrow: it supports optional leading import
  declarations plus executable main-body statements, not arbitrary package-less
  top-level Go file structure
- broader playground-style rewriting, automatic helper extraction, and richer
  multi-file snippet synthesis remain out of scope in the current slice
