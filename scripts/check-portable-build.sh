#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

env \
  -u GOWASM_CARGO_CONFIG \
  -u RUSTC_WRAPPER \
  -u CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER \
  -u RUSTFLAGS \
  cargo check --package gowasm-vm --tests

env \
  -u GOWASM_CARGO_CONFIG \
  -u RUSTC_WRAPPER \
  -u CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER \
  -u RUSTFLAGS \
  cargo check --package gowasm-engine-wasm --tests
