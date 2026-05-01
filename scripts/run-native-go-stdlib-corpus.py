#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


VALID_OUTPUT_FIELDS = {"stdout", "stderr", "stdout_and_stderr"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked stdlib native-Go differential catalog, which groups "
            "the per-package-family corpora under one validated index."
        )
    )
    parser.add_argument(
        "--index",
        default="testdata/stdlib-differential/index.json",
        help="Path to the stdlib differential catalog index.json.",
    )
    parser.add_argument(
        "--package",
        dest="package_ids",
        action="append",
        default=[],
        help="Only run the named package-family id. May be provided more than once.",
    )
    parser.add_argument(
        "--list-packages",
        action="store_true",
        help="Print the available package-family ids and exit.",
    )
    parser.add_argument(
        "--show-browser-deviations",
        action="store_true",
        help="Include the allowed browser deviations in package listings.",
    )
    parser.add_argument(
        "--go",
        default="go",
        help="Go binary to execute. Defaults to `go`.",
    )
    parser.add_argument(
        "--keep-temp",
        action="store_true",
        help="Forward --keep-temp to the underlying native-Go corpus runner.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    index_path = Path(args.index).resolve()
    index = load_catalog(index_path)
    packages = select_packages(index["packages"], args.package_ids)

    if args.list_packages:
        for package in packages:
            render_package_listing(package, args.show_browser_deviations)
        return 0

    repo_root = index_path.parent.parent.parent
    runner_path = repo_root / "scripts" / "native_go_differential_runner.py"
    passes = 0

    for package in packages:
        validate_package_corpus(index_path.parent, package)
        argv = [
            "python3",
            str(runner_path),
            "--index",
            str((index_path.parent / package["corpus_index"]).resolve()),
            "--go",
            args.go,
        ]
        if args.keep_temp:
            argv.append("--keep-temp")
        print(f'==> {package["id"]}: {package["name"]}')
        completed = subprocess.run(argv, cwd=repo_root, check=False)
        if completed.returncode != 0:
            print(
                f'stdlib differential package `{package["id"]}` failed with exit code {completed.returncode}',
                file=sys.stderr,
            )
            return completed.returncode or 1
        passes += 1

    print(f"stdlib native-go differential corpus: {passes} package(s) passed")
    return 0


def load_catalog(index_path: Path) -> dict:
    payload = json.loads(index_path.read_text())
    if payload.get("schema_version") != 1:
        raise SystemExit(
            f"unexpected stdlib differential schema_version {payload.get('schema_version')!r}"
        )
    packages = payload.get("packages")
    if not isinstance(packages, list) or not packages:
        raise SystemExit("stdlib differential catalog should contain at least one package")
    for package in packages:
        validate_catalog_package_shape(package)
    return payload


def validate_catalog_package_shape(package: object) -> None:
    if not isinstance(package, dict):
        raise SystemExit("stdlib differential package entries should be objects")
    package_id = package.get("id")
    name = package.get("name")
    package_names = package.get("packages")
    corpus_index = package.get("corpus_index")
    output_field = package.get("expected_output_field")
    host_independent = package.get("host_independent")
    allowed_browser_deviations = package.get("allowed_browser_deviations")

    if not isinstance(package_id, str) or not package_id:
        raise SystemExit("stdlib differential packages need a non-empty `id`")
    if not isinstance(name, str) or not name:
        raise SystemExit(f"stdlib differential package `{package_id}` needs a non-empty `name`")
    if (
        not isinstance(package_names, list)
        or not package_names
        or not all(isinstance(item, str) and item for item in package_names)
    ):
        raise SystemExit(
            f"stdlib differential package `{package_id}` needs a non-empty string `packages` list"
        )
    if not isinstance(corpus_index, str) or not corpus_index:
        raise SystemExit(
            f"stdlib differential package `{package_id}` needs a non-empty `corpus_index`"
        )
    if output_field not in VALID_OUTPUT_FIELDS:
        raise SystemExit(
            f"stdlib differential package `{package_id}` needs `expected_output_field` "
            f"in {sorted(VALID_OUTPUT_FIELDS)}"
        )
    if not isinstance(host_independent, bool):
        raise SystemExit(
            f"stdlib differential package `{package_id}` needs boolean `host_independent`"
        )
    if (
        not isinstance(allowed_browser_deviations, list)
        or not all(isinstance(item, str) and item for item in allowed_browser_deviations)
    ):
        raise SystemExit(
            f"stdlib differential package `{package_id}` needs string `allowed_browser_deviations`"
        )


def select_packages(packages: list[dict], requested_ids: list[str]) -> list[dict]:
    if not requested_ids:
        return packages

    requested = set(requested_ids)
    selected = [package for package in packages if package["id"] in requested]
    missing = sorted(requested - {package["id"] for package in selected})
    if missing:
        raise SystemExit(
            f"unknown stdlib differential package ids: {', '.join(missing)}"
        )
    return selected


def validate_package_corpus(index_root: Path, package: dict) -> None:
    corpus_path = (index_root / package["corpus_index"]).resolve()
    if not corpus_path.exists():
        raise SystemExit(
            f'stdlib differential package `{package["id"]}` references missing corpus `{corpus_path}`'
        )
    payload = json.loads(corpus_path.read_text())
    if payload.get("schema_version") != 1:
        raise SystemExit(
            f'stdlib differential package `{package["id"]}` references unexpected corpus schema '
            f"{payload.get('schema_version')!r}"
        )
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise SystemExit(
            f'stdlib differential package `{package["id"]}` should reference at least one case'
        )

    output_field = package["expected_output_field"]
    for case in cases:
        if not isinstance(case, dict) or not isinstance(case.get("id"), str):
            raise SystemExit(
                f'stdlib differential package `{package["id"]}` contains an invalid case entry'
            )
        ensure_case_output_field(package["id"], case, output_field)


def ensure_case_output_field(package_id: str, case: dict, output_field: str) -> None:
    needs_stdout = output_field in {"stdout", "stdout_and_stderr"}
    needs_stderr = output_field in {"stderr", "stdout_and_stderr"}
    if needs_stdout and "expected_native_go_stdout" not in case:
        raise SystemExit(
            f'stdlib differential package `{package_id}` case `{case["id"]}` '
            "should define `expected_native_go_stdout`"
        )
    if needs_stderr and "expected_native_go_stderr" not in case:
        raise SystemExit(
            f'stdlib differential package `{package_id}` case `{case["id"]}` '
            "should define `expected_native_go_stderr`"
        )


def render_package_listing(package: dict, show_browser_deviations: bool) -> None:
    package_names = ", ".join(package["packages"])
    print(
        f'{package["id"]}: {package_names} '
        f'[{package["expected_output_field"]}] '
        f'host_independent={str(package["host_independent"]).lower()}'
    )
    if not show_browser_deviations:
        return
    deviations = package["allowed_browser_deviations"]
    if not deviations:
        print("  browser deviations: none")
        return
    print("  browser deviations:")
    for deviation in deviations:
        print(f"    - {deviation}")


if __name__ == "__main__":
    raise SystemExit(main())
