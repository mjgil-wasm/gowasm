# Wasm Worker Protocol

This document is the formal spec for the `gowasm-engine-wasm` request/response
boundary and the browser worker contract layered on top of it.

## Scope

The protocol has two layers:

- the shared JSON protocol types in `gowasm-host-types`
- the raw Wasm ABI exported by `gowasm-engine-wasm`

The current checked shared protocol version is `12`, carried in the
`Ready.info.protocol_version` boot response.

## Top-Level JSON Requests

<!-- engine-request-kinds:start -->
- `Boot`
- `LoadModuleGraph`
- `Compile`
- `Format`
- `Lint`
- `TestPackage`
- `TestSnippet`
- `Run`
- `Resume`
- `ResumeModule`
- `Cancel`
<!-- engine-request-kinds:end -->

Encoding rules:

- top-level requests use `#[serde(tag = "kind", rename_all = "snake_case")]`
- every request body is valid UTF-8 JSON
- file payloads are carried as `WorkspaceFile { path, contents }`
- `Run` provides entry-path and optional host clock fields
- `TestPackage` provides a target path and an optional exact-name `filter`
  field for one supported top-level `Test*` function
- `Resume` carries a `CapabilityResult`
- `ResumeModule` carries a `ModuleResult`

## Top-Level JSON Responses

<!-- engine-response-kinds:start -->
- `Ready`
- `ModuleGraphResult`
- `Diagnostics`
- `FormatResult`
- `LintResult`
- `TestResult`
- `RunResult`
- `CapabilityRequest`
- `ModuleRequest`
- `Cancelled`
- `Fatal`
<!-- engine-response-kinds:end -->

Encoding rules:

- top-level responses also use `#[serde(tag = "kind", rename_all = "snake_case")]`
- `Ready` carries `EngineInfo { protocol_version, engine_name }`
- `CapabilityRequest` and `ModuleRequest` pause execution and hand control back
  to the browser worker
- `Diagnostics`, `RunResult`, and `TestResult` diagnostics now carry
  `Diagnostic.category`, and runtime-backed diagnostics also carry
  `RuntimeDiagnostic.category`
- those same diagnostics now also freeze optional `source_span`,
  `source_excerpt`, and `suggested_action` fields on the shared `Diagnostic`
  payload, while runtime-backed failures keep structured `stack_trace` frames
  under `Diagnostic.runtime`
- `Cancelled` now carries `category = "runtime_cancellation"`
- `Fatal` is the terminal low-level protocol failure envelope for ABI or bridge
  failures and now carries an explicit category field

## Error Categories

The shared protocol category field uses the `gowasm-host-types::ErrorCategory`
enum serialized as `snake_case`.

The currently emitted response categories are:

- `compile_error` for compiler/type-check diagnostics
- `tooling` for formatting, linting, and test-runner orchestration diagnostics
- `protocol_error` for malformed request payloads, stale resume/module replies,
  and Wasm bridge-level protocol faults
- `host_error` for browser-owned module/cache/fetch failures after a valid
  protocol exchange
- `runtime_panic`, `runtime_trap`, `runtime_budget_exhaustion`, and
  `runtime_deadlock` for VM-backed failures
- `runtime_cancellation` for `Cancelled`

## Capability Subprotocol

Capability request kinds:

<!-- capability-request-kinds:start -->
- `ClockNow`
- `Sleep`
- `Fetch`
- `FetchStart`
- `FetchBodyChunk`
- `FetchBodyComplete`
- `FetchBodyAbort`
- `FetchResponseChunk`
- `FetchResponseClose`
- `Yield`
<!-- capability-request-kinds:end -->

Capability result kinds:

<!-- capability-result-kinds:start -->
- `ClockNow`
- `Sleep`
- `Fetch`
- `FetchStart`
- `FetchBodyChunk`
- `FetchBodyComplete`
- `FetchBodyAbort`
- `FetchResponseChunk`
- `FetchResponseClose`
- `Yield`
<!-- capability-result-kinds:end -->

`Fetch*` flows support both one-shot and streamed request/response bodies. The
worker runtime owns browser `fetch()` integration and converts that host state
into these protocol messages.

## Module Subprotocol

Module request kinds:

<!-- module-request-kinds:start -->
- `CacheLookup`
- `Fetch`
- `CacheFill`
<!-- module-request-kinds:end -->

Module result kinds:

<!-- module-result-kinds:start -->
- `CacheLookup`
- `Fetch`
- `CacheFill`
<!-- module-result-kinds:end -->

The worker owns module-cache lookup, remote bundle fetch, and cache fill
operations. The engine only requests those actions through this typed subflow.

## Wasm ABI Exports

The Wasm bridge exports:

- `alloc_request_buffer(len) -> *mut u8`
- `free_request_buffer(ptr, len)`
- `handle_request(ptr, len) -> u32`
- `response_ptr() -> *const u8`
- `response_len() -> usize`
- `free_response_buffer(ptr, len)`

Request/response ownership rules:

1. The caller allocates a request buffer with `alloc_request_buffer`.
2. The caller writes UTF-8 JSON request bytes into that buffer.
3. The caller invokes `handle_request(ptr, len)`.
4. The caller frees the request buffer with `free_request_buffer(ptr, len)`.
5. The caller reads the owned response bytes from `response_ptr()` and
   `response_len()`.
6. The caller copies those response bytes out.
7. The caller releases that owned response allocation with
   `free_response_buffer(ptr, len)`.

The bridge keeps at most one owned response buffer alive at a time; a newer
request replaces and frees the previous response allocation.

## ABI Status Codes

| Code | Name | Meaning |
| --- | --- | --- |
<!-- wasm-abi-status-codes:start -->
| `0` | `Ok` | Request decoded, handled, and serialized normally. |
| `1` | `InvalidRequestBuffer` | The incoming raw pointer or length was invalid for reading request bytes. |
| `2` | `InvalidUtf8` | Request bytes were readable but not valid UTF-8. |
| `3` | `InvalidProtocol` | UTF-8 decoded but did not parse as `EngineRequest` JSON. |
| `4` | `Panic` | The engine panicked before producing a normal response. |
<!-- wasm-abi-status-codes:end -->

## Panic And Error Serialization

- `InvalidRequestBuffer`, `InvalidUtf8`, `InvalidProtocol`, and `Panic` still
  return a serialized `EngineResponse::Fatal` body
- malformed-protocol and bridge-level failures therefore preserve both a raw
  numeric ABI status and a JSON fatal envelope with
  `category = "protocol_error"`
- runtime or compile failures that occur after a valid request decode stay on
  the normal `EngineResponse` path rather than changing ABI status codes

## Browser Worker Replay Surface

The real browser worker harness lives in `web/test-worker.html` and
`web/test-worker.js`. That checked worker path exercises boot, run, cancel,
tooling, module, and capability-backed flows against the Wasm bridge and worker
runtime instead of relying only on crate-local unit tests.

## Checked Evidence

- `crates/engine-wasm/src/lib.rs` contains ABI round-trip, malformed UTF-8,
  malformed protocol, and panic-serialization tests
- `crates/host-types/src/tests_tooling_protocol.rs` and
  `crates/host-types/src/tests_module_protocol.rs` pin the JSON request/response
  shapes
- `web/test-worker.html` replays the browser worker path over the checked
  protocol kinds and capability flows
