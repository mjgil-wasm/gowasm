#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked differential/native-Go release gates that protect "
            "the parked-state support surface."
        )
    )
    parser.add_argument(
        "--commands-file",
        help=(
            "Optional JSON file used by tests to override the default suite "
            "command list."
        ),
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="Print the configured suite names and exit.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def default_suites(root: Path) -> list[dict[str, object]]:
    return [
        {
            "name": "compiler_vm_representative_parity",
            "argv": [
                "env",
                "RUSTFLAGS=-Awarnings",
                "cargo",
                "test",
                "-p",
                "gowasm-compiler",
                "parity_release_gate_cases_pass_through_direct_compiler_and_vm_execution",
                "--",
                "--nocapture",
            ],
        },
        {
            "name": "native_go_representative_parity",
            "argv": [
                "bash",
                "./scripts/run-native-go-parity-corpus.sh",
                "--case",
                "local_package_chain",
                "--case",
                "mixed_local_remote_chain",
            ],
        },
        {
            "name": "compiler_vm_imported_package_release_gate",
            "argv": [
                "env",
                "RUSTFLAGS=-Awarnings",
                "cargo",
                "test",
                "-p",
                "gowasm-compiler",
                "imported_package_release_gate_cases_pass_through_direct_compiler_and_vm_execution",
                "--",
                "--nocapture",
            ],
        },
        {
            "name": "compiler_vm_json_differential",
            "argv": [
                "env",
                "RUSTFLAGS=-Awarnings",
                "cargo",
                "test",
                "-p",
                "gowasm-compiler",
                "json_differential_corpus_matches_checked_in_native_go_outputs",
                "--",
                "--nocapture",
            ],
        },
        {
            "name": "compiler_vm_reflect_fmt_differential",
            "argv": [
                "env",
                "RUSTFLAGS=-Awarnings",
                "cargo",
                "test",
                "-p",
                "gowasm-compiler",
                "reflect_fmt_differential_corpus_matches_checked_in_native_go_outputs",
                "--",
                "--nocapture",
            ],
        },
        {
            "name": "compiler_vm_semantic_differential",
            "argv": [
                "env",
                "RUSTFLAGS=-Awarnings",
                "cargo",
                "test",
                "-p",
                "gowasm-compiler",
                "semantic_differential_corpus_matches_checked_in_native_go_outputs",
                "--",
                "--nocapture",
            ],
        },
        {
            "name": "native_go_semantic_differential",
            "argv": ["bash", "./scripts/run-native-go-semantic-corpus.sh"],
        },
        {
            "name": "native_go_stdlib_differential",
            "argv": ["bash", "./scripts/run-native-go-stdlib-corpus.sh"],
        },
    ]


def load_suites(root: Path, commands_file: str | None) -> list[dict[str, object]]:
    if commands_file is None:
        return default_suites(root)

    payload = json.loads(Path(commands_file).read_text())
    if payload.get("schema_version") != 1:
        raise SystemExit(
            "custom differential release gate commands should use schema_version 1"
        )
    suites = payload.get("suites")
    if not isinstance(suites, list) or not suites:
        raise SystemExit("custom differential release gate commands should include suites")

    validated: list[dict[str, object]] = []
    for suite in suites:
        if not isinstance(suite, dict):
            raise SystemExit("custom differential release gate suites should be objects")
        name = suite.get("name")
        argv = suite.get("argv")
        if not isinstance(name, str) or not name:
            raise SystemExit("custom differential release gate suites need a non-empty name")
        if (
            not isinstance(argv, list)
            or not argv
            or not all(isinstance(arg, str) and arg for arg in argv)
        ):
            raise SystemExit(
                f"custom differential release gate suite `{name}` needs a non-empty argv list"
            )
        validated.append({"name": name, "argv": argv})
    return validated


def shell_render(argv: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in argv)


def main() -> int:
    args = parse_args()
    root = repo_root()
    suites = load_suites(root, args.commands_file)

    if args.list:
        for suite in suites:
            print(suite["name"])
        return 0

    for suite in suites:
        name = suite["name"]
        argv = suite["argv"]
        assert isinstance(name, str)
        assert isinstance(argv, list)
        print(f"==> {name}: {shell_render(argv)}")
        completed = subprocess.run(argv, cwd=root, check=False)
        if completed.returncode != 0:
            print(
                f"differential release gate `{name}` failed with exit code {completed.returncode}",
                file=sys.stderr,
            )
            return completed.returncode or 1

    print(f"differential release gates: {len(suites)} suite(s) passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
