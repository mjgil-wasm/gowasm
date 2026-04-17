# Ten Out Of Ten Acceptance

This document is the canonical acceptance rubric for the broader `gowasm`
design target.

## How To Read This Spec

Use these labels consistently when discussing maturity:

- `supported now`: backed by checked tests on the current supported path
- `implemented but not yet supported`: present in code, but not promised until
  the same change updates the support docs and evidence
- `deferred`: intended for the broader design target, but not yet implemented
- `excluded`: out of scope unless product direction changes

A feature only counts toward a "10/10" claim when it satisfies the gates below
and the checked evidence is current.

## Architecture And Host Boundary

Acceptance gate:

- the Rust/Wasm engine remains responsible for parsing, compilation, runtime
  semantics, diagnostics, formatting, and test execution
- the browser or host boundary remains responsible for real I/O, timers,
  storage, networking, module fetching, and UI
- unsupported host capabilities cannot bypass the explicit capability bridge
- crate boundaries stay explicit across lexer, parser, compiler, VM, engine,
  Wasm ABI, and host protocol layers

Evidence required:

- engine, worker, and browser tests proving capability-gated behavior
- architecture docs that describe the boundary without widening
  parked-state support claims

## Test Discipline And Evidence

Acceptance gate:

- every broadened behavior lands with direct unit or integration coverage
- every widened supported slice updates support docs and checked evidence in
  the same change
- release-gate orchestration covers compiler, VM, engine, worker, browser,
  parity, docs, and performance checks
- failures remain reproducible through checked scripts or fixtures

Evidence required:

- checked gate scripts for support-surface workflow, differential replay, docs
  consistency, browser performance, and browser-facing harnesses
- maintained golden fixtures or corpora for compiler/runtime/worker contracts

## Runtime Semantics

Acceptance gate:

- the modeled Go subset behaves consistently across compiler checks, VM
  execution, diagnostics, and host-wait resume paths
- runtime invariants for values, frames, pointers, closures, maps, slices,
  channels, interfaces, and errors are documented and tested
- panic, recover, budget exhaustion, and traced runtime failures preserve
  machine-readable diagnostics plus user-facing stack information

Evidence required:

- direct compiler and VM regression tests for the supported runtime model
- golden diagnostics for compile failures, runtime faults, and panic stacks

## Go Parity And Differential Evidence

Acceptance gate:

- representative language and stdlib slices are compared against native Go
  wherever parity is claimed
- differential corpora record both supported parity and intentional rejection
  behavior
- unsupported or deferred features fail clearly instead of looking silently
  accepted

Evidence required:

- checked semantic, stdlib, JSON, reflect/fmt, and browser-facing parity
  corpora
- documented allowed deviations when the browser-first product boundary
  intentionally differs from native Go

## Browser Product Readiness

Acceptance gate:

- the real browser shell and worker path support the documented workflow for
  edit, run, snippet test, package test, format, lint, module loading,
  cancellation, and recovery
- browser-only capabilities remain behind explicit consent and validation
  boundaries
- performance and storage budgets have checked thresholds and replayable
  measurement steps

Evidence required:

- checked browser worker and browser shell harnesses
- checked performance measurements and budget verification scripts

## Maintainability And Change Discipline

Acceptance gate:

- maintainers can add features, stdlib entries, worker messages, and browser
  surface without re-deriving the repo workflow
- implementation inventories, support docs, and acceptance docs stay aligned
- final certification claims record exact toolchain, browser, and replay gate
  inputs

Evidence required:

- an up-to-date maintainer playbook and support docs
- checked docs gates that fail when required acceptance or support docs drift
