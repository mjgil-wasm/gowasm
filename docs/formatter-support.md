# Formatter Support

Task `078` freezes the current formatter surface.

## Ownership

- `crates/host-types`
  - owns the `format` request and `format_result` response schema
- `crates/engine`
  - owns conservative Go formatting for parseable workspace `.go` files
- `web/`
  - owns the browser worker and shell paths that submit format requests and
    render rewritten files plus diagnostics

## Formatter Mode

- the formatter is intentionally conservative, not `gofmt`-compatible
- it only rewrites parseable editable workspace `.go` files
- non-Go files are returned unchanged
- invalid Go files keep their original contents and surface ordinary tooling
  diagnostics instead

## Supported Guarantees

- normalize `CRLF` / `CR` line endings to `LF`
- ensure formatted Go files end with a trailing newline
- trim trailing spaces and tabs on ordinary code lines
- reindent delimiter-scoped blocks with tabs
- reindent multiline import blocks
- preserve supported generic declarations and instantiations while reindenting
  surrounding delimiter-scoped blocks
- reindent other multiline delimiter-scoped constructs such as composite
  literals and multiline calls
- preserve multiline raw-string bodies exactly once a raw string is open
- keep supported line comments from distorting delimiter-scoped indentation
- preserve non-Go workspace files untouched

## Explicit Limits

- the formatter does not claim full `gofmt` parity
- it does not sort or regroup imports
- it does not rewrite broader spacing/style choices outside the current
  delimiter-scoped indentation and trailing-whitespace normalization slice
- raw-string contents remain byte-preserving instead of being normalized
- block comments remain outside the frozen surface until the parser accepts
  them through the same formatting gate
- broader comment alignment and whole-file layout rewriting outside the current
  conservative rules remain out of scope
