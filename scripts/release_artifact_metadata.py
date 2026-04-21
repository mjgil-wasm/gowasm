#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

import tomllib


DEFAULT_ARTIFACT_PATH = Path("web/generated/gowasm_engine_wasm.wasm")
CHECKED_METADATA_PATH = Path("docs/generated/release-artifact-metadata.json")
RUST_TOOLCHAIN_PATH = Path("rust-toolchain.toml")
WORKSPACE_MANIFEST_PATH = Path("Cargo.toml")
ENGINE_WASM_MANIFEST_PATH = Path("crates/engine-wasm/Cargo.toml")
CARGO_LOCK_PATH = Path("Cargo.lock")
BUILD_SCRIPT_PATH = Path("scripts/build-web.sh")
WASM_TARGET = "wasm32-unknown-unknown"
WASM_PROFILE = "ship"


def sha256_hex_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def sha256_hex_file(path: Path) -> str:
    return sha256_hex_bytes(path.read_bytes())


def read_toolchain_channel(path: Path) -> str:
    payload = tomllib.loads(path.read_text(encoding="utf-8"))
    return payload["toolchain"]["channel"]


def relative_path(root: Path, path: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def build_metadata(root: Path, artifact_path: Path) -> dict[str, Any]:
    workspace_manifest = root / WORKSPACE_MANIFEST_PATH
    engine_wasm_manifest = root / ENGINE_WASM_MANIFEST_PATH
    cargo_lock = root / CARGO_LOCK_PATH
    build_script = root / BUILD_SCRIPT_PATH
    rust_toolchain = root / RUST_TOOLCHAIN_PATH

    if not artifact_path.is_file():
        raise SystemExit(
            f"release artifact not found: {artifact_path}\n"
            "build it first with scripts/build-web.sh"
        )

    artifact_bytes = artifact_path.read_bytes()
    return {
        "schema_version": 1,
        "artifact": {
            "path": relative_path(root, artifact_path),
            "bytes": len(artifact_bytes),
            "sha256": sha256_hex_bytes(artifact_bytes),
        },
        "build": {
            "target": WASM_TARGET,
            "profile": WASM_PROFILE,
            "toolchain_channel": read_toolchain_channel(rust_toolchain),
            "build_script_path": BUILD_SCRIPT_PATH.as_posix(),
            "build_script_sha256": sha256_hex_file(build_script),
        },
        "inputs": {
            "workspace_manifest_path": WORKSPACE_MANIFEST_PATH.as_posix(),
            "workspace_manifest_sha256": sha256_hex_file(workspace_manifest),
            "engine_wasm_manifest_path": ENGINE_WASM_MANIFEST_PATH.as_posix(),
            "engine_wasm_manifest_sha256": sha256_hex_file(engine_wasm_manifest),
            "cargo_lock_path": CARGO_LOCK_PATH.as_posix(),
            "cargo_lock_sha256": sha256_hex_file(cargo_lock),
            "rust_toolchain_path": RUST_TOOLCHAIN_PATH.as_posix(),
            "rust_toolchain_sha256": sha256_hex_file(rust_toolchain),
        },
    }


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))
