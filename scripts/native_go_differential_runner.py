#!/usr/bin/env python3
from __future__ import annotations

import argparse
import difflib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a checked-in native-Go differential corpus and compare the "
            "current native Go stdout/stderr/exit code against the stored "
            "expected values."
        )
    )
    parser.add_argument(
        "--index",
        required=True,
        help="Path to the corpus index.json file.",
    )
    parser.add_argument(
        "--case",
        dest="case_ids",
        action="append",
        default=[],
        help="Only run the named corpus case id. May be provided more than once.",
    )
    parser.add_argument(
        "--go",
        default="go",
        help="Go binary to execute. Defaults to `go`.",
    )
    parser.add_argument(
        "--keep-temp",
        action="store_true",
        help="Keep successful temp workspaces instead of cleaning them up.",
    )
    parser.add_argument(
        "--list-cases",
        action="store_true",
        help="Print the available corpus case ids and exit.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    index_path = Path(args.index).resolve()
    corpus_root = index_path.parent
    index = load_index(index_path)

    if args.list_cases:
        for case in index["cases"]:
            print(f'{case["id"]}: {case["name"]}')
        return 0

    cases = select_cases(index["cases"], args.case_ids)
    failures: list[str] = []
    passes = 0

    for case in cases:
        error = run_case(corpus_root, case, args.go, args.keep_temp)
        if error is None:
            passes += 1
            print(f'PASS {case["id"]}: {case["name"]}')
            continue
        failures.append(error)

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print(f"native-go differential corpus: {passes} case(s) passed")
    return 0


def load_index(index_path: Path) -> dict:
    index = json.loads(index_path.read_text())
    if index.get("schema_version") != 1:
        raise SystemExit(
            f"unexpected differential corpus schema_version {index.get('schema_version')!r}"
        )
    if not index.get("cases"):
        raise SystemExit("differential corpus should contain at least one case")
    return index


def select_cases(cases: list[dict], requested_ids: list[str]) -> list[dict]:
    if not requested_ids:
        return cases

    requested = set(requested_ids)
    selected = [case for case in cases if case["id"] in requested]
    missing = sorted(requested - {case["id"] for case in selected})
    if missing:
        raise SystemExit(f"unknown differential corpus case ids: {', '.join(missing)}")
    return selected


def run_case(
    corpus_root: Path, case: dict, go_binary: str, keep_temp: bool
) -> str | None:
    temp_root = Path(tempfile.mkdtemp(prefix=f'native-go-diff-{case["id"]}-'))
    workspace_root = temp_root / "workspace"
    try:
        materialize_workspace(corpus_root, workspace_root, case)
        completed = subprocess.run(
            [go_binary, "run", "."],
            cwd=workspace_root,
            capture_output=True,
            text=True,
            check=False,
        )

        expected_exit_code = int(case.get("expected_native_go_exit_code", 0))
        expected_stdout = case.get("expected_native_go_stdout", "")
        expected_stderr = case.get("expected_native_go_stderr", "")

        if completed.returncode != expected_exit_code:
            return (
                f'FAIL {case["id"]}: native Go exit code diverged\n'
                f"  workspace: {workspace_root}\n"
                f"  expected: {expected_exit_code}\n"
                f"  actual: {completed.returncode}\n"
                f"  stderr:\n{indent_text(completed.stderr.rstrip())}"
            )
        if completed.stdout != expected_stdout:
            return render_diff_failure(
                case["id"],
                workspace_root,
                "expected_native_go_stdout",
                "native_go_stdout",
                expected_stdout,
                completed.stdout,
            )
        if completed.stderr != expected_stderr:
            return render_diff_failure(
                case["id"],
                workspace_root,
                "expected_native_go_stderr",
                "native_go_stderr",
                expected_stderr,
                completed.stderr,
            )

        if keep_temp:
            print(f"  kept temp workspace: {workspace_root}")
            return None

        shutil.rmtree(temp_root)
        return None
    except FileNotFoundError as exc:
        return (
            f'FAIL {case["id"]}: could not execute {go_binary!r}\n'
            f"  missing executable: {exc.filename}"
        )
    except Exception as exc:  # pragma: no cover - defensive script path
        return f'FAIL {case["id"]}: {exc}\n  workspace: {workspace_root}'


def materialize_workspace(corpus_root: Path, workspace_root: Path, case: dict) -> None:
    workspace_root.mkdir(parents=True, exist_ok=True)
    source_root = corpus_root / case["id"] / "workspace"
    for relative_path in case["workspace_files"]:
        source_path = source_root / relative_path
        destination_path = workspace_root / relative_path
        destination_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_path, destination_path)

    entry_path = workspace_root / case["entry_path"]
    if not entry_path.exists():
        raise RuntimeError(f"entry_path `{case['entry_path']}` should exist in the workspace")

    go_mod_path = workspace_root / "go.mod"
    if not go_mod_path.exists():
        go_mod_path.write_text("module semanticdifferential\n\ngo 1.21\n")


def render_diff_failure(
    case_id: str,
    workspace_root: Path,
    expected_name: str,
    actual_name: str,
    expected: str,
    actual: str,
) -> str:
    diff = "\n".join(
        difflib.unified_diff(
            expected.splitlines(),
            actual.splitlines(),
            fromfile=expected_name,
            tofile=actual_name,
            lineterm="",
        )
    )
    return (
        f"FAIL {case_id}: native Go output diverged\n"
        f"  workspace: {workspace_root}\n"
        f"  diff:\n{indent_text(diff)}"
    )


def indent_text(text: str) -> str:
    if not text:
        return "    <empty>"
    return "\n".join(f"    {line}" for line in text.splitlines())


if __name__ == "__main__":
    raise SystemExit(main())
