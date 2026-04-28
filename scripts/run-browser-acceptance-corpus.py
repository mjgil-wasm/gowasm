#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path


REQUIRED_TAGS = {
    "worker",
    "shell",
    "workspace_fs",
    "fetch",
    "timers",
    "context_cancellation",
    "module_cache",
    "snapshots",
    "project_import",
    "worker_recovery",
    "dom",
    "worker_protocol",
    "output",
    "diagnostics",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked browser acceptance corpus over the real worker and "
            "browser-shell test pages."
        )
    )
    parser.add_argument(
        "--index",
        default="testdata/browser-acceptance/index.json",
        help="Path to the browser acceptance corpus index.json.",
    )
    parser.add_argument(
        "--case",
        dest="case_ids",
        action="append",
        default=[],
        help="Only run the named case id. May be provided more than once.",
    )
    parser.add_argument(
        "--tag",
        dest="tags",
        action="append",
        default=[],
        help="Only run cases that include this tag. May be provided more than once.",
    )
    parser.add_argument(
        "--list-cases",
        action="store_true",
        help="Print the available case ids and exit.",
    )
    parser.add_argument(
        "--list-tags",
        action="store_true",
        help="Print the covered tag set and exit.",
    )
    parser.add_argument(
        "--show-tags",
        action="store_true",
        help="Include case tags in `--list-cases` output.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the runner commands without executing them.",
    )
    parser.add_argument(
        "--browser-binary",
        help="Forward an explicit browser binary to browser_page_runner.py.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def main() -> int:
    args = parse_args()
    index_path = Path(args.index).resolve()
    payload = load_index(index_path)
    validate_required_tag_coverage(payload["cases"])
    cases = select_cases(payload["cases"], args.case_ids, args.tags)

    if args.list_cases:
        for case in cases:
            render_case_listing(case, args.show_tags)
        return 0

    if args.list_tags:
        for tag in sorted({tag for case in payload["cases"] for tag in case["tags"]}):
            print(tag)
        return 0

    page_runner = repo_root() / "scripts" / "browser_page_runner.py"
    passes = 0
    for case in cases:
        argv = build_case_command(page_runner, case, args.browser_binary)
        if args.dry_run:
            print(shell_render(argv))
            passes += 1
            continue

        print(f'==> {case["id"]}: {case["name"]}')
        completed = subprocess.run(argv, cwd=repo_root(), check=False)
        if completed.returncode != 0:
            print(
                f'browser acceptance case `{case["id"]}` failed with exit code {completed.returncode}',
                file=sys.stderr,
            )
            return completed.returncode or 1
        passes += 1

    print(f"browser acceptance corpus: {passes} case(s) passed")
    return 0


def load_index(index_path: Path) -> dict:
    payload = json.loads(index_path.read_text())
    if payload.get("schema_version") != 1:
        raise SystemExit(
            f"unexpected browser acceptance schema_version {payload.get('schema_version')!r}"
        )
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise SystemExit("browser acceptance corpus should contain at least one case")
    for case in cases:
        validate_case_shape(case)
    return payload


def validate_case_shape(case: object) -> None:
    if not isinstance(case, dict):
        raise SystemExit("browser acceptance cases should be objects")
    case_id = require_non_empty_string(case.get("id"), "browser acceptance case needs `id`")
    require_non_empty_string(case.get("name"), f"browser acceptance case `{case_id}` needs `name`")
    require_non_empty_string(case.get("page"), f"browser acceptance case `{case_id}` needs `page`")
    require_non_empty_string(
        case.get("element_id"),
        f"browser acceptance case `{case_id}` needs `element_id`",
    )

    page_query = case.get("page_query")
    if page_query is not None and not isinstance(page_query, str):
        raise SystemExit(
            f"browser acceptance case `{case_id}` needs string `page_query` when provided"
        )

    for field_name in ["expect_substrings", "reject_substrings", "tags"]:
        field_value = case.get(field_name)
        if (
            not isinstance(field_value, list)
            or not field_value
            or not all(isinstance(item, str) and item for item in field_value)
        ):
            raise SystemExit(
                f"browser acceptance case `{case_id}` needs a non-empty string list `{field_name}`"
            )

    timeout_seconds = case.get("timeout_seconds")
    if not isinstance(timeout_seconds, int) or timeout_seconds <= 0:
        raise SystemExit(
            f"browser acceptance case `{case_id}` needs positive integer `timeout_seconds`"
        )


def require_non_empty_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise SystemExit(message)
    return value


def validate_required_tag_coverage(cases: list[dict]) -> None:
    seen = {tag for case in cases for tag in case["tags"]}
    missing = sorted(REQUIRED_TAGS - seen)
    if missing:
        raise SystemExit(
            "browser acceptance corpus is missing required coverage tag(s): "
            + ", ".join(missing)
        )


def select_cases(cases: list[dict], requested_ids: list[str], requested_tags: list[str]) -> list[dict]:
    selected = cases
    if requested_ids:
        requested_id_set = set(requested_ids)
        selected = [case for case in selected if case["id"] in requested_id_set]
        missing = sorted(requested_id_set - {case["id"] for case in selected})
        if missing:
            raise SystemExit(f"unknown browser acceptance case ids: {', '.join(missing)}")

    if requested_tags:
        requested_tag_set = set(requested_tags)
        unknown_tags = sorted(requested_tag_set - {tag for case in cases for tag in case["tags"]})
        if unknown_tags:
            raise SystemExit(f"unknown browser acceptance tags: {', '.join(unknown_tags)}")
        selected = [
            case for case in selected if requested_tag_set.issubset(set(case["tags"]))
        ]

    if not selected:
        raise SystemExit("browser acceptance selection should include at least one case")
    return selected


def render_case_listing(case: dict, show_tags: bool) -> None:
    if not show_tags:
        print(f'{case["id"]}: {case["name"]}')
        return
    print(f'{case["id"]}: {case["name"]} [{", ".join(case["tags"])}]')


def build_case_command(page_runner: Path, case: dict, browser_binary: str | None) -> list[str]:
    argv = [
        "python3",
        str(page_runner),
        "--page",
        case["page"],
        "--element-id",
        case["element_id"],
        "--timeout-seconds",
        str(case["timeout_seconds"]),
    ]
    page_query = case.get("page_query")
    if page_query:
        argv.extend(["--page-query", page_query])
    if browser_binary:
        argv.extend(["--browser-binary", browser_binary])
    for substring in case["expect_substrings"]:
        argv.extend(["--expect-substring", substring])
    for substring in case["reject_substrings"]:
        argv.extend(["--reject-substring", substring])
    return argv


def shell_render(argv: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in argv)


if __name__ == "__main__":
    raise SystemExit(main())
