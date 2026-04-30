use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn generator_script() -> PathBuf {
    repo_root().join("scripts/generate-release-artifact-metadata.py")
}

fn checker_script() -> PathBuf {
    repo_root().join("scripts/check-release-artifact-reproducibility.py")
}

fn temp_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("gowasm-{name}-{}-{unique}", process::id()))
}

fn write_file(root: &Path, relative_path: &str, contents: &[u8]) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, contents).expect("fixture file should be written");
}

fn run_generator(root: &Path, args: &[&str]) -> Output {
    Command::new("python3")
        .arg(generator_script())
        .arg("--root")
        .arg(root)
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("artifact metadata generator should run")
}

fn run_checker(root: &Path, args: &[&str]) -> Output {
    Command::new("python3")
        .arg(checker_script())
        .arg("--root")
        .arg(root)
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("artifact reproducibility checker should run")
}

fn write_fixture_root(root: &Path) {
    write_file(
        root,
        "rust-toolchain.toml",
        br#"[toolchain]
channel = "1.94.1"
components = ["rustfmt"]
profile = "minimal"
"#,
    );
    write_file(
        root,
        "Cargo.toml",
        br#"[workspace]
members = ["crates/engine-wasm"]

[profile.ship]
inherits = "release"
opt-level = 3
lto = true
codegen-units = 1
incremental = false
"#,
    );
    write_file(
        root,
        "Cargo.lock",
        b"# fake lockfile for reproducibility tests\n",
    );
    write_file(
        root,
        "crates/engine-wasm/Cargo.toml",
        br#"[package]
name = "gowasm-engine-wasm"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(
        root,
        "scripts/build-web.sh",
        br#"#!/usr/bin/env bash
set -euo pipefail
echo fake build
"#,
    );
    write_file(
        root,
        "web/generated/gowasm_engine_wasm.wasm",
        b"\0asmfake-wasm-artifact-for-repro-tests",
    );
}

#[test]
fn release_artifact_metadata_generator_emits_stable_shape() {
    let root = temp_root("release-artifact-generator-shape");
    write_fixture_root(&root);

    let output = run_generator(&root, &[]);
    let _ = fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "generator should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("generated metadata should be valid json");
    assert_eq!(payload["schema_version"], 1);
    assert_eq!(
        payload["artifact"]["path"],
        "web/generated/gowasm_engine_wasm.wasm"
    );
    assert_eq!(payload["build"]["target"], "wasm32-unknown-unknown");
    assert_eq!(payload["build"]["profile"], "ship");
    assert_eq!(payload["build"]["toolchain_channel"], "1.94.1");
}

#[test]
fn release_artifact_reproducibility_checker_accepts_matching_checked_metadata() {
    let root = temp_root("release-artifact-checker-pass");
    write_fixture_root(&root);

    let generated = run_generator(&root, &[]);
    assert!(
        generated.status.success(),
        "generator should succeed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    write_file(
        &root,
        "docs/generated/release-artifact-metadata.json",
        &generated.stdout,
    );

    let output = run_checker(&root, &[]);
    let _ = fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "checker should accept matching metadata: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("release artifact reproducibility metadata passed"),
        "checker should report success, got:\n{stdout}"
    );
}

#[test]
fn release_artifact_reproducibility_checker_rejects_stale_checked_metadata() {
    let root = temp_root("release-artifact-checker-stale");
    write_fixture_root(&root);
    write_file(
        &root,
        "docs/generated/release-artifact-metadata.json",
        br#"{
  "schema_version": 1,
  "artifact": {
    "path": "web/generated/gowasm_engine_wasm.wasm",
    "bytes": 1,
    "sha256": "stale"
  },
  "build": {
    "target": "wasm32-unknown-unknown",
    "profile": "ship",
    "toolchain_channel": "1.94.1",
    "build_script_path": "scripts/build-web.sh",
    "build_script_sha256": "stale"
  },
  "inputs": {
    "workspace_manifest_path": "Cargo.toml",
    "workspace_manifest_sha256": "stale",
    "engine_wasm_manifest_path": "crates/engine-wasm/Cargo.toml",
    "engine_wasm_manifest_sha256": "stale",
    "cargo_lock_path": "Cargo.lock",
    "cargo_lock_sha256": "stale",
    "rust_toolchain_path": "rust-toolchain.toml",
    "rust_toolchain_sha256": "stale"
  }
}"#,
    );

    let output = run_checker(&root, &[]);
    let _ = fs::remove_dir_all(&root);

    assert!(
        !output.status.success(),
        "checker should reject stale checked metadata"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("release artifact metadata is stale"),
        "failure output should mention stale metadata, got:\n{stderr}"
    );
}

#[test]
fn release_artifact_reproducibility_checker_prints_current_metadata() {
    let root = temp_root("release-artifact-checker-print");
    write_fixture_root(&root);

    let output = run_checker(&root, &["--print-metadata"]);
    let _ = fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "checker should print current metadata: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("printed metadata should be valid json");
    assert_eq!(payload["build"]["toolchain_channel"], "1.94.1");
    assert_eq!(
        payload["artifact"]["path"],
        "web/generated/gowasm_engine_wasm.wasm"
    );
}
