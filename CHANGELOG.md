# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Compiler and VM support for `len(ch)` and `cap(ch)` on channels.
- Supported Go subset matrix and standard library coverage table to README.
- `cargo fmt --check` to CI workflow.

### Fixed
- `CONTRIBUTING.md` removed references to non-existent release gate suites.
- Regenerated missing `docs/generated/release-artifact-metadata.json`.
- Regenerated missing `docs/generated/browser-performance-metrics.env` baseline.
- `.gitignore` now excludes `__pycache__/` and `*.pyc`.

### Changed
- Integer arithmetic now wraps on overflow (two's complement), matching Go semantics.
- `time.Timer` zero value (`var t time.Timer`) is now valid; `Stop()` and `Reset()` return `false` gracefully when the channel is missing.
- `%T` verb fallback renders proper names for built-in types (`time.Time`, `sync.Mutex`, `http.Header`, etc.).

## [0.1.0] - 2025-05-10

### Added
- Initial open-source release of gowasm.
- Browser-first Go execution environment in Rust compiled to WebAssembly.
- Go 1.21 subset compiler and bytecode VM.
- Goroutines, channels, `select`, interfaces, generics, methods, closures, defer/panic/recover.
- Browser-backed adapters for `time`, `net/http`, `io/fs`, `context`, and narrow `os`.
- Multi-platform CI (Linux, macOS, Windows) with headless browser integration.
- Release gates for artifact reproducibility, differential corpora, fuzz tests, and performance budgets.
