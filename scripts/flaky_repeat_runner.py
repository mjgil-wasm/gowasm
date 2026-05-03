#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_BASE_SEED = 0x105F1A9E
DEFAULT_RUNS = 3
SEED_MULTIPLIER = 6364136223846793005
SEED_INCREMENT = 1442695040888963407
MASK_64 = (1 << 64) - 1


@dataclass
class CommandTarget:
    id: str
    name: str
    argv: list[str]
    tags: list[str]


@dataclass
class BrowserPageTarget:
    id: str
    name: str
    page: str
    page_query_template: str
    element_id: str
    expect_substrings: list[str]
    reject_substrings: list[str]
    timeout_seconds: int
    artifact_element_id: str | None
    tags: list[str]


Target = CommandTarget | BrowserPageTarget


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Repeat the checked scheduler/select/concurrency/browser suites with "
            "deterministic seeds and emit replayable failure reports."
        )
    )
    parser.add_argument(
        "--catalog",
        help="Optional flaky-repeat catalog path override.",
    )
    parser.add_argument(
        "--target",
        action="append",
        default=[],
        help="Optional target id filter. May be passed multiple times.",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=DEFAULT_RUNS,
        help="Number of deterministic seeds to run per selected target.",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_BASE_SEED,
        help="Base deterministic seed used to derive the repeat sequence.",
    )
    parser.add_argument(
        "--list-targets",
        action="store_true",
        help="Print the checked flaky-repeat target ids and exit.",
    )
    parser.add_argument(
        "--list-seeds",
        action="store_true",
        help="Print the deterministic seed sequence for the chosen runs and exit.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the commands that would run without executing them.",
    )
    parser.add_argument(
        "--report-output",
        help="Optional JSON report path override.",
    )
    parser.add_argument(
        "--replay-report",
        help="Replay the failing target/seed recorded in a previous report.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def default_catalog(root: Path) -> Path:
    return root / "testdata/flaky-repeat/index.json"


def shell_render(argv: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in argv)


def load_catalog(path: Path) -> list[Target]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("schema_version") != 1:
        raise SystemExit("flaky-repeat catalog should use schema_version 1")
    raw_targets = payload.get("targets")
    if not isinstance(raw_targets, list) or not raw_targets:
        raise SystemExit("flaky-repeat catalog should define non-empty targets")
    targets: list[Target] = []
    for raw in raw_targets:
        if not isinstance(raw, dict):
            raise SystemExit("flaky-repeat targets should be objects")
        kind = raw.get("kind")
        target_id = raw.get("id")
        name = raw.get("name")
        tags = raw.get("tags")
        if not isinstance(target_id, str) or not target_id:
            raise SystemExit("flaky-repeat targets need a non-empty id")
        if not isinstance(name, str) or not name:
            raise SystemExit(f"flaky-repeat target `{target_id}` needs a non-empty name")
        if not isinstance(tags, list) or not all(isinstance(tag, str) and tag for tag in tags):
            raise SystemExit(f"flaky-repeat target `{target_id}` needs non-empty string tags")
        if kind == "command":
            argv = raw.get("argv")
            if not isinstance(argv, list) or not all(isinstance(arg, str) and arg for arg in argv):
                raise SystemExit(f"command target `{target_id}` needs a non-empty argv list")
            targets.append(
                CommandTarget(
                    id=target_id,
                    name=name,
                    argv=argv,
                    tags=tags,
                )
            )
        elif kind == "browser_page":
            page = raw.get("page")
            page_query_template = raw.get("page_query_template")
            element_id = raw.get("element_id")
            expect_substrings = raw.get("expect_substrings")
            reject_substrings = raw.get("reject_substrings")
            timeout_seconds = raw.get("timeout_seconds")
            artifact_element_id = raw.get("artifact_element_id")
            if not isinstance(page, str) or not page:
                raise SystemExit(f"browser target `{target_id}` needs a non-empty page")
            if not isinstance(page_query_template, str) or "{seed}" not in page_query_template:
                raise SystemExit(
                    f"browser target `{target_id}` needs a page_query_template containing {{seed}}"
                )
            if not isinstance(element_id, str) or not element_id:
                raise SystemExit(f"browser target `{target_id}` needs a non-empty element_id")
            if not isinstance(expect_substrings, list) or not all(
                isinstance(item, str) and item for item in expect_substrings
            ):
                raise SystemExit(
                    f"browser target `{target_id}` needs non-empty expect_substrings"
                )
            if not isinstance(reject_substrings, list) or not all(
                isinstance(item, str) and item for item in reject_substrings
            ):
                raise SystemExit(
                    f"browser target `{target_id}` needs non-empty reject_substrings"
                )
            if not isinstance(timeout_seconds, int) or timeout_seconds <= 0:
                raise SystemExit(
                    f"browser target `{target_id}` needs a positive timeout_seconds"
                )
            if artifact_element_id is not None and not isinstance(artifact_element_id, str):
                raise SystemExit(
                    f"browser target `{target_id}` artifact_element_id must be a string when present"
                )
            targets.append(
                BrowserPageTarget(
                    id=target_id,
                    name=name,
                    page=page,
                    page_query_template=page_query_template,
                    element_id=element_id,
                    expect_substrings=expect_substrings,
                    reject_substrings=reject_substrings,
                    timeout_seconds=timeout_seconds,
                    artifact_element_id=artifact_element_id,
                    tags=tags,
                )
            )
        else:
            raise SystemExit(f"flaky-repeat target `{target_id}` has unsupported kind `{kind}`")
    return targets


def derive_seeds(base_seed: int, runs: int) -> list[int]:
    if runs <= 0:
        raise SystemExit("--runs should be positive")
    state = base_seed & MASK_64
    if state == 0:
        state = 1
    seeds: list[int] = []
    for _ in range(runs):
        state = (state * SEED_MULTIPLIER + SEED_INCREMENT) & MASK_64
        seeds.append(state)
    return seeds


def filter_targets(targets: list[Target], selected_ids: list[str]) -> list[Target]:
    if not selected_ids:
        return targets
    selected_set = set(selected_ids)
    filtered = [target for target in targets if target.id in selected_set]
    if len(filtered) != len(selected_set):
        missing = sorted(selected_set - {target.id for target in targets})
        raise SystemExit(f"unknown flaky-repeat target id(s): {', '.join(missing)}")
    return filtered


def artifact_path(root: Path, explicit: str | None) -> Path:
    if explicit:
        return Path(explicit)
    return Path(tempfile.gettempdir()) / "gowasm-flaky-repeat-report.json"


def build_browser_runner_argv(
    root: Path,
    target: BrowserPageTarget,
    seed: int,
    artifact_output: Path | None,
) -> list[str]:
    argv = [
        "python3",
        str(root / "scripts/browser_page_runner.py"),
        "--page",
        target.page,
        "--page-query",
        target.page_query_template.format(seed=seed),
        "--element-id",
        target.element_id,
        "--timeout-seconds",
        str(target.timeout_seconds),
    ]
    for expected in target.expect_substrings:
        argv.extend(["--expect-substring", expected])
    for rejected in target.reject_substrings:
        argv.extend(["--reject-substring", rejected])
    if artifact_output is not None and target.artifact_element_id:
        argv.extend(
            [
                "--artifact-element-id",
                target.artifact_element_id,
                "--artifact-output",
                str(artifact_output),
            ]
        )
    return argv


def write_report(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_target_iteration(
    root: Path,
    target: Target,
    seed: int,
    iteration: int,
    report_path: Path,
    dry_run: bool,
) -> dict[str, Any]:
    env_overrides = {
        "GOWASM_FLAKY_SEED": str(seed),
        "GOWASM_FLAKY_ITERATION": str(iteration),
        "GOWASM_FLAKY_TARGET": target.id,
    }
    browser_artifact_output = None
    if isinstance(target, CommandTarget):
        argv = target.argv
        cwd = root
    else:
        browser_artifact_output = report_path.with_suffix(
            f".{target.id}.{iteration}.artifact.json"
        )
        argv = build_browser_runner_argv(root, target, seed, browser_artifact_output)
        cwd = root

    shell_command = shell_render(argv)
    if dry_run:
        print(
            f"{target.id} seed={seed} iteration={iteration}: "
            f"{' '.join(f'{key}={value}' for key, value in env_overrides.items())} {shell_command}"
        )
        return {
            "target_id": target.id,
            "seed": seed,
            "iteration": iteration,
            "status": "dry_run",
            "argv": argv,
            "env_overrides": env_overrides,
            "shell_command": shell_command,
        }

    print(
        f"==> target={target.id} seed={seed} iteration={iteration} {shell_command}",
        flush=True,
    )
    completed = subprocess.run(
        argv,
        cwd=cwd,
        env=os.environ | env_overrides,
        check=False,
        capture_output=True,
        text=True,
    )
    return {
        "target_id": target.id,
        "seed": seed,
        "iteration": iteration,
        "status": "passed" if completed.returncode == 0 else "failed",
        "exit_code": completed.returncode,
        "argv": argv,
        "env_overrides": env_overrides,
        "shell_command": shell_command,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
        "browser_artifact_output": str(browser_artifact_output)
        if browser_artifact_output is not None
        else None,
    }


def replay_from_report(path: Path) -> int:
    payload = json.loads(path.read_text(encoding="utf-8"))
    replay = payload.get("failure")
    if not isinstance(replay, dict):
        raise SystemExit("replay report should contain a `failure` object")
    argv = replay.get("argv")
    env_overrides = replay.get("env_overrides")
    if not isinstance(argv, list) or not all(isinstance(arg, str) and arg for arg in argv):
        raise SystemExit("replay report should contain a non-empty argv list")
    if not isinstance(env_overrides, dict) or not all(
        isinstance(key, str) and isinstance(value, str) for key, value in env_overrides.items()
    ):
        raise SystemExit("replay report should contain string env_overrides")
    root = repo_root()
    print(f"==> replay: {shell_render(argv)}")
    completed = subprocess.run(
        argv,
        cwd=root,
        env=os.environ | env_overrides,
        check=False,
    )
    return completed.returncode


def main() -> int:
    args = parse_args()
    if args.replay_report:
        return replay_from_report(Path(args.replay_report))

    root = repo_root()
    catalog_path = Path(args.catalog) if args.catalog else default_catalog(root)
    targets = filter_targets(load_catalog(catalog_path), args.target)

    if args.list_targets:
        for target in targets:
            print(target.id)
        return 0

    seeds = derive_seeds(args.seed, args.runs)
    if args.list_seeds:
        for seed in seeds:
            print(seed)
        return 0

    report_output = artifact_path(root, args.report_output)
    summary: dict[str, Any] = {
        "schema_version": 1,
        "catalog": str(catalog_path),
        "base_seed": args.seed,
        "runs": args.runs,
        "selected_targets": [target.id for target in targets],
        "seeds": seeds,
        "results": [],
        "status": "running",
    }

    for target in targets:
        for iteration, seed in enumerate(seeds):
            result = run_target_iteration(
                root=root,
                target=target,
                seed=seed,
                iteration=iteration,
                report_path=report_output,
                dry_run=args.dry_run,
            )
            summary["results"].append(result)
            if result["status"] == "failed":
                summary["status"] = "failed"
                summary["failure"] = result
                summary["replay"] = {
                    "report_path": str(report_output),
                    "shell_command": f"python3 scripts/flaky_repeat_runner.py --replay-report {shlex.quote(str(report_output))}",
                }
                write_report(report_output, summary)
                print(
                    f"flaky repeat runner failed for target `{target.id}` seed {seed}; "
                    f"report written to {report_output}",
                    file=sys.stderr,
                )
                return result.get("exit_code", 1) or 1

    summary["status"] = "dry_run" if args.dry_run else "passed"
    write_report(report_output, summary)
    print(
        f"flaky repeat runner: {len(targets)} target(s) x {len(seeds)} seed(s) {summary['status']}"
    )
    print(f"report: {report_output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
