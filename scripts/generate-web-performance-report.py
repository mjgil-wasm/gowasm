#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Generate a JSON report from a browser performance metrics env file, "
            "the checked budget env file, and an optional previous metrics env file."
        )
    )
    parser.add_argument("--budget", required=True, help="Path to the budget env file.")
    parser.add_argument("--metrics", required=True, help="Path to the current metrics env file.")
    parser.add_argument(
        "--baseline-metrics",
        help="Optional previous metrics env file used to emit delta/trend fields.",
    )
    parser.add_argument("--output", required=True, help="Path to the JSON report to write.")
    return parser.parse_args()


def parse_env_file(path: Path) -> dict[str, str]:
    if not path.is_file():
        raise SystemExit(f"env file not found: {path}")

    result: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise SystemExit(f"invalid env line in {path}: {raw_line!r}")
        key, value = line.split("=", 1)
        result[key] = value
    return result


def require_int(mapping: dict[str, str], key: str, label: str) -> int:
    value = mapping.get(key)
    if value is None:
        raise SystemExit(f"missing required key {key!r} in {label}")
    try:
        numeric = int(value)
    except ValueError as error:
        raise SystemExit(f"expected integer for {key!r} in {label}, got {value!r}") from error
    if numeric < 0:
        raise SystemExit(f"expected non-negative integer for {key!r} in {label}, got {numeric}")
    return numeric


def percent(numerator: int, denominator: int) -> float:
    if denominator <= 0:
        return 0.0
    return round((numerator / denominator) * 100, 2)


def metric_entry(
    name: str,
    actual: int,
    budget_max: int,
    baseline_actual: int | None,
) -> dict[str, object]:
    entry: dict[str, object] = {
        "name": name,
        "actual": actual,
        "budget_max": budget_max,
        "headroom": budget_max - actual,
        "utilization_percent": percent(actual, budget_max),
        "status": "pass" if actual <= budget_max else "fail",
    }
    if baseline_actual is not None:
        delta = actual - baseline_actual
        entry["baseline_actual"] = baseline_actual
        entry["delta_from_baseline"] = delta
        entry["delta_direction"] = "flat" if delta == 0 else ("up" if delta > 0 else "down")
    return entry


def main() -> int:
    args = parse_args()
    budget_label = args.budget
    metrics_label = args.metrics
    baseline_label = args.baseline_metrics
    budget_path = Path(args.budget).resolve()
    metrics_path = Path(args.metrics).resolve()
    output_path = Path(args.output).resolve()
    baseline_path = Path(args.baseline_metrics).resolve() if args.baseline_metrics else None

    budget = parse_env_file(budget_path)
    metrics = parse_env_file(metrics_path)
    baseline = parse_env_file(baseline_path) if baseline_path else None

    if budget.get("metric_version") != "3":
        raise SystemExit(f"unsupported budget metric_version: {budget.get('metric_version')!r}")
    if metrics.get("metric_version") != budget["metric_version"]:
        raise SystemExit(
            "metric_version mismatch: "
            f"budget={budget['metric_version']} metrics={metrics.get('metric_version')!r}"
        )
    if baseline and baseline.get("metric_version") != budget["metric_version"]:
        raise SystemExit(
            "metric_version mismatch: "
            f"budget={budget['metric_version']} baseline={baseline.get('metric_version')!r}"
        )

    metric_names = [
        "wasm_bytes",
        "worker_boot_ms",
        "compile_diagnostics_ms",
        "hello_run_ms",
        "worker_boot_and_run_ms",
        "gc_churn_run_ms",
        "module_load_ms",
        "hello_memory_bytes",
        "gc_churn_memory_bytes",
        "module_load_memory_bytes",
        "module_cache_storage_delta_bytes",
        "shell_ready_ms",
        "shell_run_ms",
    ]

    entries: list[dict[str, object]] = []
    failures: list[str] = []
    for metric_name in metric_names:
        actual = require_int(metrics, metric_name, str(metrics_path))
        budget_max = require_int(budget, f"{metric_name}_max", str(budget_path))
        baseline_actual = (
            require_int(baseline, metric_name, str(baseline_path)) if baseline and baseline_path else None
        )
        entry = metric_entry(metric_name, actual, budget_max, baseline_actual)
        entries.append(entry)
        if entry["status"] != "pass":
            failures.append(metric_name)

    report = {
        "metric_version": budget["metric_version"],
        "budget_path": budget_label,
        "metrics_path": metrics_label,
        "baseline_metrics_path": baseline_label,
        "memory_metric_source": metrics.get("memory_metric_source"),
        "summary": {
            "total_metrics": len(entries),
            "failed_metrics": failures,
            "status": "pass" if not failures else "fail",
        },
        "metrics": entries,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
