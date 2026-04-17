# Browser-Safe `os` Contract

The parked-state `os` slice is intentionally narrow and workspace-backed.

Supported helpers:
- environment helpers: `Getenv`, `Setenv`, `Unsetenv`, `Clearenv`, `LookupEnv`, `Environ`, `ExpandEnv`, `Expand`
- workspace filesystem helpers: `DirFS`, `ReadFile`, `WriteFile`, `ReadDir`, `Stat`, `Lstat`, `MkdirAll`, `RemoveAll`
- browser-safe path/process helpers: `Getwd`, `TempDir`, `UserHomeDir`, `UserCacheDir`, `UserConfigDir`, `Hostname`, `Executable`, `Getuid`, `Geteuid`, `Getgid`, `Getegid`, `Getpid`, `Getppid`, `Getpagesize`, `Getgroups`
- exported sentinels and path/process constants already registered in the `os` package

Frozen behavior:
- the current cwd is the logical workspace root `"/"`
- `ReadFile`, `WriteFile`, `ReadDir`, `Stat`, `Lstat`, `MkdirAll`, and `RemoveAll` all operate on the same mutable workspace tree used by `io/fs`
- cleaned absolute slash paths and cleaned relative paths are both accepted on that logical root
- `WriteFile` creates or truncates regular workspace files, requires the parent directory to already exist, and keeps using the current wrapped-error contract for `ErrInvalid` and `ErrNotExist`
- browser-safe sentinel values stay explicit: `Hostname()` returns `"js"`, `Executable()` returns the current js/wasm-style error, uid/gid/pid helpers return `-1`, `Getpagesize()` returns `65536`, and `Getgroups()` returns the current js/wasm-style not-implemented error

Explicit exclusions:
- raw host file/process APIs such as `os.Open`, `os.Create`, and wider host-process parity are not part of the current subset
- the runtime does not claim direct host descriptors, raw host cwd changes, or unrestricted host filesystem access

The same change that adjusts this contract must also update checked
tests and browser capability coverage when the supported surface changes
