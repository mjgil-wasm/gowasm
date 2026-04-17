#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shlex
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_SUITE_PATH = "testdata/benchmark-suite/index.json"
DEFAULT_BUDGET_PATH = "testdata/benchmark-budget.env"
DEFAULT_BROWSER_METRICS_PATH = "docs/generated/browser-performance-metrics.env"


@dataclass
class CommandBenchmark:
    id: str
    name: str
    metric_key: str
    budget_key: str
    argv: list[str]
    runs: int
    tags: list[str]


@dataclass
class BrowserMetricBenchmark:
    id: str
    name: str
    metric_key: str
    budget_key: str
    source_metric: str
    tags: list[str]


Benchmark = CommandBenchmark | BrowserMetricBenchmark


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked benchmark smoke suite and compare the results "
            "against the pinned benchmark budget file."
        )
    )
    parser.add_argument(
        "--suite",
        default=DEFAULT_SUITE_PATH,
        help="Benchmark suite JSON file.",
    )
    parser.add_argument(
        "--budget",
        default=DEFAULT_BUDGET_PATH,
        help="Benchmark budget env file.",
    )
    parser.add_argument(
        "--metrics",
        default=DEFAULT_BROWSER_METRICS_PATH,
        help="Browser metrics env file used by browser_metric benchmarks.",
    )
    parser.add_argument(
        "--benchmark",
        action="append",
        default=[],
        help="Optional benchmark id filter. May be passed multiple times.",
    )
    parser.add_argument(
        "--tag",
        action="append",
        default=[],
        help="Optional benchmark tag filter. May be passed multiple times.",
    )
    parser.add_argument(
        "--list-benchmarks",
        action="store_true",
        help="Print the checked benchmark ids and exit.",
    )
    parser.add_argument(
        "--list-tags",
        action="store_true",
        help="Print the checked benchmark tags and exit.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the benchmark commands and budgets without executing them.",
    )
    parser.add_argument(
        "--report-output",
        help="Optional JSON report path.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def shell_render(argv: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in argv)


def parse_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if not separator:
            raise SystemExit(f"invalid env line in {path}: {raw_line}")
        values[key.strip()] = value.strip()
    return values


def parse_float_env(path: Path) -> dict[str, float]:
    parsed: dict[str, float] = {}
    for key, value in parse_env_file(path).items():
        try:
            parsed[key] = float(value)
        except ValueError as exc:
            raise SystemExit(f"invalid numeric benchmark budget `{key}={value}`") from exc
    return parsed


def parse_numeric_metrics_env(path: Path) -> dict[str, float]:
    parsed: dict[str, float] = {}
    for key, value in parse_env_file(path).items():
        try:
            parsed[key] = float(value)
        except ValueError:
            continue
    return parsed


def load_suite(path: Path) -> list[Benchmark]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("schema_version") != 1:
        raise SystemExit("benchmark suite should use schema_version 1")
    raw_benchmarks = payload.get("benchmarks")
    if not isinstance(raw_benchmarks, list) or not raw_benchmarks:
        raise SystemExit("benchmark suite should define non-empty benchmarks")

    benchmarks: list[Benchmark] = []
    for raw in raw_benchmarks:
        if not isinstance(raw, dict):
            raise SystemExit("benchmark suite entries should be objects")
        benchmark_id = raw.get("id")
        name = raw.get("name")
        metric_key = raw.get("metric_key")
        budget_key = raw.get("budget_key")
        tags = raw.get("tags")
        if not isinstance(benchmark_id, str) or not benchmark_id:
            raise SystemExit("benchmark entries need a non-empty id")
        if not isinstance(name, str) or not name:
            raise SystemExit(f"benchmark `{benchmark_id}` needs a non-empty name")
        if not isinstance(metric_key, str) or not metric_key:
            raise SystemExit(f"benchmark `{benchmark_id}` needs a non-empty metric_key")
        if not isinstance(budget_key, str) or not budget_key:
            raise SystemExit(f"benchmark `{benchmark_id}` needs a non-empty budget_key")
        if not isinstance(tags, list) or not all(isinstance(tag, str) and tag for tag in tags):
            raise SystemExit(f"benchmark `{benchmark_id}` needs non-empty tags")

        kind = raw.get("kind")
        if kind == "command":
            argv = raw.get("argv")
            runs = raw.get("runs")
            if not isinstance(argv, list) or not all(
                isinstance(arg, str) and arg for arg in argv
            ):
                raise SystemExit(f"command benchmark `{benchmark_id}` needs a non-empty argv")
            if not isinstance(runs, int) or runs <= 0:
                raise SystemExit(f"command benchmark `{benchmark_id}` needs positive runs")
            benchmarks.append(
                CommandBenchmark(
                    id=benchmark_id,
                    name=name,
                    metric_key=metric_key,
                    budget_key=budget_key,
                    argv=argv,
                    runs=runs,
                    tags=tags,
                )
            )
        elif kind == "browser_metric":
            source_metric = raw.get("source_metric")
            if not isinstance(source_metric, str) or not source_metric:
                raise SystemExit(
                    f"browser_metric benchmark `{benchmark_id}` needs a source_metric"
                )
            benchmarks.append(
                BrowserMetricBenchmark(
                    id=benchmark_id,
                    name=name,
                    metric_key=metric_key,
                    budget_key=budget_key,
                    source_metric=source_metric,
                    tags=tags,
                )
            )
        else:
            raise SystemExit(f"benchmark `{benchmark_id}` has unsupported kind `{kind}`")
    return benchmarks


def filter_benchmarks(
    benchmarks: list[Benchmark], selected_ids: list[str], selected_tags: list[str]
) -> list[Benchmark]:
    filtered = benchmarks
    if selected_ids:
        selected_id_set = set(selected_ids)
        filtered = [benchmark for benchmark in filtered if benchmark.id in selected_id_set]
        found_ids = {benchmark.id for benchmark in filtered}
        missing_ids = sorted(selected_id_set - found_ids)
        if missing_ids:
            raise SystemExit(f"unknown benchmark id(s): {', '.join(missing_ids)}")
    if selected_tags:
        selected_tag_set = set(selected_tags)
        filtered = [
            benchmark
            for benchmark in filtered
            if selected_tag_set.intersection(benchmark.tags)
        ]
        if not filtered:
            raise SystemExit(
                f"no benchmarks matched tag filter(s): {', '.join(sorted(selected_tag_set))}"
            )
    return filtered


def report_path(root: Path, explicit: str | None) -> Path:
    if explicit:
        return Path(explicit)
    return Path(tempfile.gettempdir()) / "gowasm-benchmark-suite-report.json"


def run_command_benchmark(benchmark: CommandBenchmark, dry_run: bool) -> dict[str, Any]:
    if dry_run:
        return {
            "id": benchmark.id,
            "status": "dry_run",
            "kind": "command",
            "argv": benchmark.argv,
            "runs": benchmark.runs,
        }

    durations_ms: list[float] = []
    for _ in range(benchmark.runs):
        started = time.perf_counter()
        completed = subprocess.run(
            benchmark.argv,
            cwd=repo_root(),
            check=False,
            capture_output=True,
            text=True,
        )
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        durations_ms.append(elapsed_ms)
        if completed.returncode != 0:
            return {
                "id": benchmark.id,
                "status": "failed",
                "kind": "command",
                "argv": benchmark.argv,
                "runs": benchmark.runs,
                "exit_code": completed.returncode,
                "stdout": completed.stdout,
                "stderr": completed.stderr,
                "durations_ms": durations_ms,
            }
    return {
        "id": benchmark.id,
        "status": "measured",
        "kind": "command",
        "argv": benchmark.argv,
        "runs": benchmark.runs,
        "durations_ms": durations_ms,
        "min_ms": min(durations_ms),
        "median_ms": statistics.median(durations_ms),
        "max_ms": max(durations_ms),
    }


def run_browser_metric_benchmark(
    benchmark: BrowserMetricBenchmark, metrics: dict[str, float], dry_run: bool
) -> dict[str, Any]:
    if dry_run:
        return {
            "id": benchmark.id,
            "status": "dry_run",
            "kind": "browser_metric",
            "source_metric": benchmark.source_metric,
        }
    if benchmark.source_metric not in metrics:
        raise SystemExit(
            f"browser metrics file is missing `{benchmark.source_metric}` for "
            f"benchmark `{benchmark.id}`"
        )
    value = metrics[benchmark.source_metric]
    return {
        "id": benchmark.id,
        "status": "measured",
        "kind": "browser_metric",
        "source_metric": benchmark.source_metric,
        "value": value,
    }


def compare_against_budget(
    benchmark: Benchmark, result: dict[str, Any], budgets: dict[str, float]
) -> dict[str, Any]:
    budget = budgets.get(benchmark.budget_key)
    if budget is None:
        raise SystemExit(
            f"benchmark budget file is missing `{benchmark.budget_key}` for `{benchmark.id}`"
        )
    if result["status"] == "dry_run":
        result["budget"] = budget
        return result
    if result["status"] == "failed":
        result["budget"] = budget
        return result
    measured = (
        result["median_ms"]
        if isinstance(benchmark, CommandBenchmark)
        else result["value"]
    )
    result["budget"] = budget
    result["measured"] = measured
    result["within_budget"] = measured <= budget
    result["status"] = "passed" if measured <= budget else "budget_failed"
    return result


def write_report(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    root = repo_root()
    suite_path = (root / args.suite).resolve()
    budget_path = (root / args.budget).resolve()
    metrics_path = (root / args.metrics).resolve()

    benchmarks = filter_benchmarks(load_suite(suite_path), args.benchmark, args.tag)
    if args.list_benchmarks:
        for benchmark in benchmarks:
            print(benchmark.id)
        return 0
    if args.list_tags:
        for tag in sorted({tag for benchmark in benchmarks for tag in benchmark.tags}):
            print(tag)
        return 0

    budgets = parse_float_env(budget_path)
    browser_metrics = parse_numeric_metrics_env(metrics_path) if any(
        isinstance(benchmark, BrowserMetricBenchmark) for benchmark in benchmarks
    ) else {}

    results: list[dict[str, Any]] = []
    for benchmark in benchmarks:
        if isinstance(benchmark, CommandBenchmark):
            result = run_command_benchmark(benchmark, args.dry_run)
        else:
            result = run_browser_metric_benchmark(benchmark, browser_metrics, args.dry_run)
        result = compare_against_budget(benchmark, result, budgets)
        results.append(result)

        if args.dry_run:
            if isinstance(benchmark, CommandBenchmark):
                print(
                    f"{benchmark.id}: budget<={result['budget']}ms runs={benchmark.runs} "
                    f"{shell_render(benchmark.argv)}"
                )
            else:
                print(
                    f"{benchmark.id}: budget<={result['budget']} from "
                    f"metrics[{benchmark.source_metric}]"
                )
            continue

        if result["status"] == "failed":
            report = {
                "schema_version": 1,
                "suite": str(suite_path),
                "budget": str(budget_path),
                "metrics": str(metrics_path),
                "status": "failed",
                "results": results,
            }
            output_path = report_path(root, args.report_output)
            write_report(output_path, report)
            print(
                f"benchmark suite command failed for `{benchmark.id}`; report: {output_path}",
                file=sys.stderr,
            )
            return result.get("exit_code", 1) or 1

        if result["status"] == "budget_failed":
            report = {
                "schema_version": 1,
                "suite": str(suite_path),
                "budget": str(budget_path),
                "metrics": str(metrics_path),
                "status": "budget_failed",
                "results": results,
            }
            output_path = report_path(root, args.report_output)
            write_report(output_path, report)
            print(
                f"benchmark `{benchmark.id}` exceeded budget: "
                f"measured={result['measured']} budget={result['budget']}",
                file=sys.stderr,
            )
            print(f"report: {output_path}", file=sys.stderr)
            return 1

    summary = {
        "schema_version": 1,
        "suite": str(suite_path),
        "budget": str(budget_path),
        "metrics": str(metrics_path),
        "status": "dry_run" if args.dry_run else "passed",
        "results": results,
    }
    output_path = report_path(root, args.report_output)
    write_report(output_path, summary)
    print(
        f"benchmark suite: {len(benchmarks)} benchmark(s) "
        f"{'dry-run complete' if args.dry_run else 'passed'}"
    )
    print(f"report: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
