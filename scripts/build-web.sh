#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT/web/generated"
WASM_TARGET="wasm32-unknown-unknown"
WASM_PROFILE="ship"
WASM_BUILD_TARGET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/gowasm-build-web.XXXXXX")"
WASM_PATH="$WASM_BUILD_TARGET_DIR/$WASM_TARGET/$WASM_PROFILE/gowasm_engine_wasm.wasm"
CARGO_CONFIG_ARGS=()

if [[ -n "${GOWASM_CARGO_CONFIG:-}" ]]; then
  CARGO_CONFIG_ARGS=(--config "$GOWASM_CARGO_CONFIG")
fi

mkdir -p "$TARGET_DIR"
trap 'rm -rf "$WASM_BUILD_TARGET_DIR"' EXIT

if ! rustup target list --installed | grep -Fxq "$WASM_TARGET"; then
  rustup target add "$WASM_TARGET"
fi

CARGO_TARGET_DIR="$WASM_BUILD_TARGET_DIR" cargo build \
  "${CARGO_CONFIG_ARGS[@]}" \
  --manifest-path "$ROOT/Cargo.toml" \
  --package gowasm-engine-wasm \
  --target "$WASM_TARGET" \
  --profile "$WASM_PROFILE"

cp "$WASM_PATH" "$TARGET_DIR/gowasm_engine_wasm.wasm"
echo "copied $WASM_PROFILE-profile wasm artifact to $TARGET_DIR/gowasm_engine_wasm.wasm"
