# Workspace FS Contract

The parked-state filesystem contract is the workspace-backed `io/fs` and narrow
`os` slice used by browser runs. It is not a promise of arbitrary host
filesystem access or broad standalone `io` package parity.

## Backing Model

- Engine `Run` requests seed the VM from the incoming workspace file list.
- The logical workspace root is `"/"`.
- Paths use the current slash-only browser-safe model.
- File-backed paths create implicit parent directories.
- `os.MkdirAll` can add explicit empty directories beside those implicit ones.

## Claimed `io` Surface

- Minimal imported interface shapes used by the supported adapters are in
  scope: `io.Reader`, `io.Closer`, and `io.ReadCloser`.
- The standalone `io` package is still outside the parked-state support claim.

## Claimed `io/fs` Surface

- Helper functions: `ValidPath`, `ReadFile`, `Stat`, `Sub`, `Glob`, `ReadDir`,
  `WalkDir`, `FileInfoToDirEntry`, `FormatDirEntry`, and `FormatFileInfo`.
- Interface dispatch is part of the contract for `fs.ReadFileFS`, `fs.StatFS`,
  `fs.ReadDirFS`, `fs.GlobFS`, and `fs.SubFS`.
- Direct handle methods are part of the contract for `fs.FS.Open`,
  `fs.File.Read`, `fs.File.Stat`, `fs.File.Close`, and
  `fs.ReadDirFile.ReadDir`.

## Metadata And Traversal

- Regular file sizes report the current byte length.
- Directory sizes report `0`.
- Workspace-backed metadata uses deterministic zero `ModTime`.
- Workspace-backed metadata returns `nil` from `Sys()`.
- Directory entries and walks use lexical order.
- `WalkDir` follows the current Go-style pre-order traversal plus `SkipDir` /
  `SkipAll` control flow over the logical workspace tree.

## Handle Behavior

- `Open` returns per-open handles for regular files and directories.
- File reads advance a per-open offset.
- `Close` mutates that hidden handle state.
- Duplicate close attempts fail instead of silently succeeding.
- Closed-handle operations stay on stable path-shaped errors:
  - `read <path>: file already closed`
  - `stat <path>: file already closed`
  - `readdir <path>: file already closed`

## Scope Boundary

- The contract is pinned by VM, engine, and browser-worker tests for read/stat,
  directory enumeration, subtree rebasing, lexical walking, metadata, and
  closed-handle errors.
- Arbitrary host filesystems, richer host metadata, and a broader standalone
  `io` package surface remain outside the parked-state claim.
