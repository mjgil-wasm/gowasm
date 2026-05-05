#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import tomllib
from pathlib import Path


REQUIRED_TOP_LEVEL_SNIPPETS = [
    "name: ci",
    "pull_request:",
    "push:",
    "workflow_dispatch:",
]

REQUIRED_EXECUTABLE_SCRIPTS = [
    "scripts/check-fuzz-harness.sh",
    "scripts/check-browser-worker.sh",
    "scripts/check-browser-shell.sh",
    "scripts/capture-browser-performance.sh",
]

REQUIRED_JOB_SNIPPETS = {
    "portable-rust": [
        "ubuntu-latest",
        "macos-latest",
        "windows-latest",
        "toolchain: ${{ matrix.rust }}",
        "cargo test --release --workspace --no-run",
    ],
    "linux-pinned-rust": [
        "toolchain: 1.94.1",
        "targets: wasm32-unknown-unknown",
        "cargo test --release --workspace --no-run",
        "bash ./scripts/build-web.sh",
    ],
    "native-go-differential": [
        "go-version: '1.23.3'",
        "bash ./scripts/check-differential-release-gates.sh",
    ],
    "fuzz-property": [
        "toolchain: 1.94.1",
        "bash ./scripts/check-fuzz-harness.sh",
    ],
    "browser-worker": [
        "browser-actions/setup-chrome@v1",
        "bash ./scripts/build-web.sh",
        "bash ./scripts/check-browser-worker.sh",
    ],
    "browser-shell": [
        "browser-actions/setup-chrome@v1",
        "bash ./scripts/build-web.sh",
        "bash ./scripts/check-browser-shell.sh",
    ],
    "browser-performance": [
        "browser-actions/setup-chrome@v1",
        "bash ./scripts/build-web.sh",
        "bash ./scripts/capture-browser-performance.sh --output \"$RUNNER_TEMP/browser-metrics.env\"",
        "bash ./scripts/check-web-performance.sh --metrics \"$RUNNER_TEMP/browser-metrics.env\"",
    ],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate the checked GitHub Actions CI matrix and the helper "
            "scripts it depends on."
        )
    )
    parser.add_argument(
        "--workflow",
        help="Optional workflow path override used by tests.",
    )
    parser.add_argument(
        "--list-jobs",
        action="store_true",
        help="Print the required checked job ids and exit.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def workflow_path(root: Path, explicit: str | None) -> Path:
    if explicit:
        return Path(explicit)
    return root / ".github/workflows/ci.yml"


def load_pinned_rust_version(root: Path) -> str:
    toolchain = tomllib.loads((root / "rust-toolchain.toml").read_text(encoding="utf-8"))
    version = toolchain.get("toolchain", {}).get("channel")
    if not isinstance(version, str) or not version:
        raise SystemExit("rust-toolchain.toml should define a non-empty toolchain.channel")
    return version


def extract_job_blocks(workflow_text: str) -> dict[str, str]:
    lines = workflow_text.splitlines()
    jobs_started = False
    current_job: str | None = None
    current_lines: list[str] = []
    blocks: dict[str, str] = {}

    for line in lines:
        if not jobs_started:
            if line.strip() == "jobs:":
                jobs_started = True
            continue

        if line.startswith("  ") and not line.startswith("    ") and line.rstrip().endswith(":"):
            if current_job is not None:
                blocks[current_job] = "\n".join(current_lines)
            current_job = line.strip()[:-1]
            current_lines = [line]
            continue

        if current_job is not None:
            current_lines.append(line)

    if current_job is not None:
        blocks[current_job] = "\n".join(current_lines)

    return blocks


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate_scripts(root: Path) -> None:
    for relative_path in REQUIRED_EXECUTABLE_SCRIPTS:
        path = root / relative_path
        require(path.is_file(), f"missing required CI helper script: {relative_path}")
        require(os.access(path, os.X_OK), f"CI helper script should be executable: {relative_path}")


def validate_workflow(root: Path, workflow: Path) -> None:
    require(workflow.is_file(), f"missing CI workflow: {workflow}")
    workflow_text = workflow.read_text(encoding="utf-8")

    for snippet in REQUIRED_TOP_LEVEL_SNIPPETS:
        require(snippet in workflow_text, f"workflow should contain top-level snippet: {snippet}")

    pinned_rust = load_pinned_rust_version(root)
    require(
        f"toolchain: {pinned_rust}" in workflow_text,
        f"workflow should pin Rust toolchain {pinned_rust}",
    )

    blocks = extract_job_blocks(workflow_text)
    for job_id, snippets in REQUIRED_JOB_SNIPPETS.items():
        block = blocks.get(job_id)
        require(block is not None, f"workflow should define job `{job_id}`")
        for snippet in snippets:
            require(
                snippet in block,
                f"job `{job_id}` should contain required snippet: {snippet}",
            )


def main() -> int:
    args = parse_args()
    if args.list_jobs:
        for job_id in REQUIRED_JOB_SNIPPETS:
            print(job_id)
        return 0

    root = repo_root()
    validate_scripts(root)
    validate_workflow(root, workflow_path(root, args.workflow))
    print("ci matrix validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
