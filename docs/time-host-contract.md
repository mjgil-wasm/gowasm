# Time Host Contract

Task `045` freezes the host-facing timer and clock contract that the VM,
engine, and browser worker must preserve.

## Covered Wait Sources

- `time.Sleep`
- one-shot `time.After`
- one-shot `time.NewTimer`
- `(*time.Timer).Stop`
- `(*time.Timer).Reset`
- `context.WithDeadline` and `context.WithTimeout` waits

`time.Ticker` is not part of the current claimed surface.

## Host Capability Contract

- The VM asks the host for `clock_now` when a runtime path needs an absolute
  timestamp, such as `time.Now`, `context.WithTimeout`, and immediate timer
  delivery.
- The VM asks the host for `sleep` only when no runnable goroutine remains and
  at least one sleep, one-shot timer, or context deadline is still pending.
- The engine forwards the shortest outstanding VM timer wait as the host sleep
  request, rounded up to whole milliseconds for the browser-side capability
  protocol.
- Resuming a `sleep` capability advances sleeping goroutines, one-shot timer
  channels, and context deadlines through the same ordered timer queue.
- `time.After` and `time.NewTimer(...).C` deliver `time.Time` values stamped
  from the host-provided resume time, not from a synthetic scheduler counter.

## Timer Semantics

- `(*time.Timer).Stop` cancels only the currently active one-shot schedule.
- `(*time.Timer).Reset` returns whether the timer was still active and drains
  any stale unread buffered timer value before arming the next schedule.
- Repeated stop/reset cycles must leave only the final active timer schedule
  live.
- Cancelling a paused run is terminal; later host wakeups must not revive the
  run or surface deadlocks from stale timer state.

## Test Anchors

- VM scheduler ordering, cancellation, and mixed timer-source wakeups are pinned
  in [`crates/vm/src/tests_scheduler.rs`](../crates/vm/src/tests_scheduler.rs).
- Compiler-level source coverage for `time` and `context` waits lives in
  [`crates/compiler/src/tests_stdlib_time.rs`](../crates/compiler/src/tests_stdlib_time.rs)
  and
  [`crates/compiler/src/tests_stdlib_context.rs`](../crates/compiler/src/tests_stdlib_context.rs).
- Engine/browser-host pause/resume coverage lives in
  [`crates/engine/src/tests.rs`](../crates/engine/src/tests.rs) and
  [`crates/engine/src/tests_time_host_contract.rs`](../crates/engine/src/tests_time_host_contract.rs).
