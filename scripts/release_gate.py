#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path


DEFAULT_PERFORMANCE_BUDGET = "testdata/web-performance-budget.env"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked release-gate suites that protect the parked-state "
            "browser, docs, parity, and performance contract."
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
        "--browser-worker-command",
        help=(
            "Shell command that runs the checked browser worker harness. "
            "Defaults to $GOWASM_BROWSER_WORKER_COMMAND."
        ),
    )
    parser.add_argument(
        "--browser-shell-command",
        help=(
            "Shell command that runs the checked browser shell harness. "
            "Defaults to $GOWASM_BROWSER_SHELL_COMMAND."
        ),
    )
    parser.add_argument(
        "--performance-metrics",
        help=(
            "Optional output path for the browser metrics env file captured "
            "by scripts/capture-browser-performance.sh and consumed by "
            "scripts/check-web-performance.sh. Defaults to "
            "$GOWASM_WEB_PERFORMANCE_METRICS or a temp-file path."
        ),
    )
    parser.add_argument(
        "--performance-budget",
        default=DEFAULT_PERFORMANCE_BUDGET,
        help=(
            "Optional performance budget env file passed to "
            "scripts/check-web-performance.sh."
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


def suite_names() -> list[str]:
    return [
        "compile_checks",
        "release_artifact_reproducibility",
        "unit_tests",
        "differential_corpora",
        "fuzz_property_tests",
        "browser_worker_tests",
        "browser_shell_tests",
        "capture_performance_metrics",
        "performance_budgets",
    ]


def browser_command(
    explicit_value: str | None, env_var: str, help_label: str
) -> str:
    value = explicit_value or os.environ.get(env_var)
    if value:
        return value
    raise SystemExit(
        f"missing required {help_label}; pass the flag explicitly or set {env_var}"
    )


def resolved_path(root: Path, value: str) -> str:
    path = Path(value)
    if not path.is_absolute():
        path = root / path
    return str(path.resolve())


def performance_metrics_path(root: Path, args: argparse.Namespace) -> str:
    value = args.performance_metrics or os.environ.get("GOWASM_WEB_PERFORMANCE_METRICS")
    if value:
        return resolved_path(root, value)
    return str(
        Path(tempfile.gettempdir()).joinpath(
            "gowasm-release-gate-browser-metrics.env"
        )
    )


def default_suites(root: Path, args: argparse.Namespace) -> list[dict[str, object]]:
    browser_worker_command = browser_command(
        args.browser_worker_command,
        "GOWASM_BROWSER_WORKER_COMMAND",
        "--browser-worker-command",
    )
    browser_shell_command = browser_command(
        args.browser_shell_command,
        "GOWASM_BROWSER_SHELL_COMMAND",
        "--browser-shell-command",
    )
    performance_metrics = performance_metrics_path(root, args)

    performance_budget = str((root / args.performance_budget).resolve())
    baseline_metrics = str(
        (root / "docs/generated/browser-performance-metrics.env").resolve()
    )
    return [
        {
            "name": "compile_checks",
            "argv": [
                "bash",
                "-lc",
                "cargo test --release --workspace --no-run && bash ./scripts/build-web.sh",
            ],
        },
        {
            "name": "release_artifact_reproducibility",
            "argv": ["bash", "./scripts/check-release-artifact-reproducibility.sh"],
        },
        {
            "name": "unit_tests",
            "argv": ["bash", "-lc", "cargo test --release --workspace"],
        },
        {
            "name": "differential_corpora",
            "argv": ["bash", "./scripts/check-differential-release-gates.sh"],
        },
        {
            "name": "fuzz_property_tests",
            "argv": ["bash", "./scripts/check-fuzz-harness.sh"],
        },
        {
            "name": "browser_worker_tests",
            "argv": ["bash", "-lc", browser_worker_command],
        },
        {
            "name": "browser_shell_tests",
            "argv": ["bash", "-lc", browser_shell_command],
        },
        {
            "name": "capture_performance_metrics",
            "argv": [
                "bash",
                "./scripts/capture-browser-performance.sh",
                "--output",
                performance_metrics,
                "--baseline-metrics",
                baseline_metrics,
            ],
        },
        {
            "name": "performance_budgets",
            "argv": [
                "bash",
                "./scripts/check-web-performance.sh",
                "--metrics",
                performance_metrics,
                "--budget",
                performance_budget,
            ],
        },
    ]


def validate_suites(suites: object, help_label: str) -> list[dict[str, object]]:
    if not isinstance(suites, list) or not suites:
        raise SystemExit(f"{help_label} should include suites")

    validated: list[dict[str, object]] = []
    for suite in suites:
        if not isinstance(suite, dict):
            raise SystemExit(f"{help_label} suites should be objects")
        name = suite.get("name")
        argv = suite.get("argv")
        if not isinstance(name, str) or not name:
            raise SystemExit(f"{help_label} suites need a non-empty name")
        if (
            not isinstance(argv, list)
            or not argv
            or not all(isinstance(arg, str) and arg for arg in argv)
        ):
            raise SystemExit(f"{help_label} suite `{name}` needs a non-empty argv list")
        validated.append({"name": name, "argv": argv})
    return validated


def load_suites(root: Path, args: argparse.Namespace) -> list[dict[str, object]]:
    if args.commands_file is None:
        if args.list:
            return [{"name": name, "argv": ["true"]} for name in suite_names()]
        return default_suites(root, args)

    payload = json.loads(Path(args.commands_file).read_text())
    if payload.get("schema_version") != 1:
        raise SystemExit("custom release gate commands should use schema_version 1")
    return validate_suites(
        payload.get("suites"), "custom release gate commands"
    )


def shell_render(argv: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in argv)


def main() -> int:
    args = parse_args()
    root = repo_root()
    suites = load_suites(root, args)

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
                f"release gate suite `{name}` failed with exit code {completed.returncode}",
                file=sys.stderr,
            )
            return completed.returncode or 1

    print(f"release gate: {len(suites)} suite(s) passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
