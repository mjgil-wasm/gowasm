# Browser-Backed `net/http` Contract

The parked-state `net/http` slice is intentionally client-only and browser-backed.

Supported helpers:
- package helpers: `CanonicalHeaderKey`, `StatusText`, `ParseHTTPVersion`,
  `DetectContentType`, `ParseTime`, `NewRequest`, `NewRequestWithContext`,
  `Get`, `Post`, `PostForm`, and `Head`
- client methods: `(*http.Client).Do`, `Get`, `Head`, `Post`, and `PostForm`
- request helpers: `(*http.Request).Context`, `WithContext`, and `Clone`
- response helpers: `(*http.Response).Location`
- header helpers: `http.Header.Clone`, `Get`, `Values`, `Set`, `Add`, and `Del`
- response body helpers exposed through the visible `io.ReadCloser` field:
  `Read` and `Close`
- type-only `import "io"` use for `io.Reader`, `io.Closer`, and
  `io.ReadCloser`

Frozen behavior:
- requests map onto the browser fetch boundary through typed capability payloads
  shared by the VM, engine, and worker
- nil, empty, and already-buffered request bodies stay on the one-shot fetch
  path, while live `io.Reader` request bodies stream through the staged
  `fetch_start` / `fetch_body_chunk` / `fetch_body_complete` turns
- streamed response bodies keep shared progress and close state across copied
  `http.Response` values, with later `Read` calls using
  `fetch_response_chunk` and unread tails closing through `fetch_response_close`
- request-context cancellation and deadlines stop transport locally before host
  dispatch when already expired, and stop in-flight browser fetch work through
  the worker `AbortController` bridge once dispatched
- response status text normalizes back through Go's status table when the
  browser leaves `statusText` empty
- `HEAD` plus HTTP no-body statuses (`1xx`, `204`, `205`, and `304`) are
  treated as bodyless even when a browser-specific `Response.body` object
  exists
- the final browser response URL is preserved and used by
  `(*http.Response).Location()` for redirect-aware relative resolution
- the worker preserves `Set-Cookie` lists when the browser exposes
  `Headers.getSetCookie()`, while ordinary duplicate response names stay on the
  Fetch-combined header path

Explicit exclusions:
- server-side `net/http` APIs are not part of the parked-state slice
- custom transports, cookie jars, redirect policy callbacks, proxy config, and
  other wider `http.Client` knobs remain out of scope
- host sockets, listeners, and unrestricted raw transport access are not
  claimed

The current boundary is pinned by:
- compiler/VM `net/http` source tests for request building, header mutation,
  request/response reuse, redirects, failure text, and context behavior
- engine fetch tests for buffered fetch, streamed upload/download, response
  close semantics, redirect-aware `Location`, and cancellation/deadline paths
- browser worker harness coverage for the supported `net/http` capability path
  plus worker-level fetch failure handling

The same change that adjusts this contract must also update checked
tests and browser capability coverage when the supported surface changes
