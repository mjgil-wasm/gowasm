# Lint Support

Task `079` freezes the current lint surface.

## Ownership

- `crates/host-types`
  - owns the `lint` request and `lint_result` response schema through the
    shared `Diagnostic` payload
- `crates/engine`
  - owns the current parser-gated lint rules, rule suppression handling, and
    source-positioned warning generation
- `web/`
  - owns the browser worker and shell paths that submit lint requests and
    render returned diagnostics

## Current Contract

- lint only analyzes editable workspace `.go` files
- non-Go files are ignored
- files that do not parse cleanly return the existing parse diagnostic through
  the normal tooling `Diagnostic` contract instead of partial lint results
- parseable files return warning-level tooling diagnostics for the currently
  frozen rule catalog below
- lint warnings now carry `file_path` plus `position { line, column }` when the
  current parser span data can identify the source location

## Rule Catalog

- `format-drift`
  - warns when a parseable `.go` file does not match the current conservative
    formatter output
  - the warning position points at the first differing source location
- `duplicate-import`
  - warns when the same import path appears more than once in a file
  - the warning position points at the later duplicate import entry
- `unused-import`
  - warns when a non-aliased import path is never referenced through its
    inferred package selector in the currently supported subset
  - the warning position points at the unused import path

## Suppressions

- the current suppression surface is intentionally narrow and file-scoped
- add a line comment anywhere in the file with:
  - `//gowasm:ignore format-drift`
  - `//gowasm:ignore duplicate-import`
  - `//gowasm:ignore unused-import`
- repeated directives are allowed
- comma-separated or whitespace-separated rule names on the same directive line
  are accepted
- only the exact frozen rule IDs above are recognized; unknown names are
  ignored

## Explicit Limits

- the linter does not claim broad Go static-analysis parity
- there is no alias-import, dot-import, or blank-import lint support because
  those import forms are outside the current parser surface
- suppressions are file-scoped only; there is no next-line or block-scoped
  suppression syntax
- no second diagnostics transport exists for linting; it stays on the shared
  `Diagnostic` payload used elsewhere in the worker and browser flow
