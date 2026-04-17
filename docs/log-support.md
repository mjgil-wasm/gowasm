# `log` Support

The parked-state `log` slice is intentionally narrow and frozen around the
package-level helpers already exercised by tutorial and playground workloads.

## Supported functions

- `log.Print`
- `log.Println`
- `log.Printf`
- `log.Fatal`
- `log.Fatalf`
- `log.SetFlags`
- `log.Flags`
- `log.SetPrefix`
- `log.Prefix`

## Behavior contract

- `Print`, `Printf`, and `Println` write one newline-terminated log entry per
  call.
- `Print` follows `fmt.Sprint` spacing rules before the trailing newline is
  added.
- `Printf` reuses the current supported `fmt` formatting slice and only adds a
  newline when the formatted message does not already end with one.
- `Fatal` and `Fatalf` emit their log entry first, then terminate the run
  through the VM `ProgramExit { code: 1 }` path.
- Prefixes are prepended to every emitted log entry through `SetPrefix` and
  observed through `Prefix`.
- Only `SetFlags(0)` is supported. Non-zero flags fail with a Go-visible panic
  instead of silently approximating unsupported timestamp or file metadata.

## Output model

Native Go writes package-level `log` output to stderr. The current parked
browser/VM model routes `log` output through the same captured stdout buffer as
`fmt`, so the checked differential corpus stores native-Go stderr as the
expected reference output for those cases.

## Explicitly out of scope

- `log.Logger`
- `log.New`
- `log.SetOutput`
- non-zero flag formatting such as timestamps, file names, or microseconds
- host-backed stderr stream separation
