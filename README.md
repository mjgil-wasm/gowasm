# gowasm

**gowasm** is a browser-first Go execution environment implemented in Rust and
compiled to WebAssembly. It runs Go code directly in the browser via a dedicated
WebAssembly engine with a browser-managed worker.

## Status

`gowasm` implements a substantial Go language/runtime slice targeted at tutorial,
playground, and small app workloads that fit a static browser deployment.
It is not claiming full Go parity. For the broader design-target acceptance
rubric, see [`docs/ten-out-of-ten-acceptance.md`](./docs/ten-out-of-ten-acceptance.md).

## Highlights

- Browser worker plus WebAssembly engine path
- Multi-file workspace execution with reachable imports
- Representative goroutine, channel, and `select` workloads
- Browser-backed `time`, `net/http`, `io/fs`, `context`, and narrow `os`
  adapters
- Snippet tests, package tests, formatting, and linting
- Checked-in browser performance/resource gate

## Quick Start

### Build the Browser Artifact

```bash
./scripts/build-web.sh
```

### Run Tests

```bash
cargo test --workspace
```

### Run the Browser Shell

```bash
./run.sh
```

Or serve manually:

```bash
cd web
python3 -m http.server 8000
```

Then open `http://127.0.0.1:8000/`.

## Project Structure

```
gowasm/
├── Cargo.toml           # Rust workspace definition
├── crates/
│   ├── lexer/           # Go lexical analysis
│   ├── parser/          # Go parsing
│   ├── compiler/        # Go to bytecode compilation
│   ├── vm/              # Bytecode virtual machine
│   ├── engine/          # Engine host bridge
│   ├── engine-wasm/     # WebAssembly export layer
│   └── host-types/      # Host type definitions
├── web/                 # Browser shell UI and worker
│   ├── index.html       # Main browser shell
│   └── generated/       # Built WebAssembly artifact
├── testdata/            # Test fixtures and corpora
├── scripts/             # Build and test scripts
└── docs/                # Architecture and design docs
```

## Scope

The project is browser-first and intentionally narrower than "all Go
everywhere." Areas like `cgo`, `plugin`, raw process control, arbitrary host
filesystem access, and full native-host parity are outside the current
supported target.

## Supported Go Subset

| Language Feature | Status | Notes |
|------------------|--------|-------|
| Goroutines | ✅ Supported | Cooperative scheduling (no pre-emption) |
| Channels | ✅ Supported | Buffered and unbuffered, `len`/`cap`/`close` |
| `select` | ✅ Supported | Including nil channel cases |
| Interfaces | ✅ Supported | Type assertions and type switches |
| Generics | ✅ Supported | Type parameters and constraints |
| Methods / Embedding | ✅ Supported | Method sets and struct embedding |
| Closures | ✅ Supported | Capturing variables by reference |
| `defer` / `panic` / `recover` | ✅ Supported | Full stack unwinding with deferred calls |
| `init` functions | ✅ Supported | Package initialization order |
| Maps | ✅ Supported | Insertion order is deterministic (not randomized) |
| Slices / Arrays | ✅ Supported | Including `append`, `copy`, `make` |
| `range` | ✅ Supported | Over slices, arrays, maps, strings, channels |
| `cgo` | ❌ Excluded | No foreign function interface |
| `unsafe` | ❌ Excluded | No unsafe pointer operations |
| Reflection | ⚠️ Read-only | `reflect.Value` inspection works; mutable operations excluded |
| Build tags | ❌ Excluded | Not supported in the current subset |
| `nil == nil` | ❌ Rejected | Explicitly disallowed (deviation from Go semantics) |

### Standard Library Coverage

| Package | Status |
|---------|--------|
| `bytes`, `strings`, `unicode/utf8` | ✅ Broad coverage |
| `fmt` | ✅ Supported (`Println`, `Printf`, `Sprintf`, etc.) |
| `time` | ✅ Supported (timers, formatting, parsing; no `Ticker`) |
| `context` | ✅ Supported |
| `sync` | ✅ Supported (`Mutex`, `RWMutex`, `WaitGroup`, `Once`, `Map`) |
| `net/http` | ✅ Narrow support (GET/POST/HEAD, headers, basic client) |
| `net/url` | ✅ Supported (parsing, query escaping, userinfo) |
| `io/fs` | ✅ Supported (virtual filesystem via browser) |
| `os` | ✅ Narrow support (env, cwd, hostname, file reads) |
| `json` | ✅ Supported (`Marshal`, `Unmarshal`) |
| `regexp` | ✅ Supported (`MatchString`, `FindAllString`, `ReplaceAllString`) |
| `strconv` | ✅ Supported |
| `math`, `math/bits` | ✅ Supported |
| `crypto/*` (md5, sha1, sha256, sha512) | ✅ Supported |
| `encoding/base64`, `encoding/hex` | ✅ Supported |
| `sort`, `slices`, `maps`, `cmp` | ✅ Supported |
| `reflect` | ⚠️ Read-only |
| `filepath`, `path` | ✅ Supported |
| `log` | ✅ Supported |
| `rand` | ✅ Supported |

## Documentation

- [`docs/ten-out-of-ten-acceptance.md`](./docs/ten-out-of-ten-acceptance.md):
  Broader design-target acceptance rubric
- [`docs/wasm-worker-protocol.md`](./docs/wasm-worker-protocol.md): Worker JSON
  and WebAssembly ABI contract
- [`docs/browser-capability-security.md`](./docs/browser-capability-security.md):
  Browser surface security model

## License

Licensed under the ISC License. See [LICENSE](./LICENSE) for details.
