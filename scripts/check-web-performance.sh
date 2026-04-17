#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_PATH="$ROOT/web/generated/gowasm_engine_wasm.wasm"
DEFAULT_BUDGET_PATH="$ROOT/testdata/web-performance-budget.env"

budget_path="$DEFAULT_BUDGET_PATH"
metrics_path=""
report_output_path=""
baseline_metrics_path=""

usage() {
  cat <<'EOF'
usage: scripts/check-web-performance.sh --metrics path/to/browser-metrics.env [--budget path/to/budget.env] [--report-output path/to/report.json] [--baseline-metrics path/to/previous-metrics.env]

Expected metrics file format:
  metric_version=3
  wasm_bytes=123456
  worker_boot_ms=123
  compile_diagnostics_ms=120
  hello_run_ms=45
  worker_boot_and_run_ms=168
  gc_churn_run_ms=220
  module_load_ms=95
  hello_memory_bytes=12345678
  gc_churn_memory_bytes=12349999
  module_load_memory_bytes=23456789
  module_cache_storage_delta_bytes=12345
  shell_ready_ms=210
  shell_run_ms=80

The Wasm artifact size is always read from web/generated/gowasm_engine_wasm.wasm.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --budget)
      budget_path="$2"
      shift 2
      ;;
    --metrics)
      metrics_path="$2"
      shift 2
      ;;
    --report-output)
      report_output_path="$2"
      shift 2
      ;;
    --baseline-metrics)
      baseline_metrics_path="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$metrics_path" ]]; then
  echo "missing required --metrics file" >&2
  usage >&2
  exit 2
fi

if [[ ! -f "$budget_path" ]]; then
  echo "budget file not found: $budget_path" >&2
  exit 1
fi

if [[ ! -f "$metrics_path" ]]; then
  echo "metrics file not found: $metrics_path" >&2
  exit 1
fi

if [[ ! -f "$WASM_PATH" ]]; then
  echo "wasm artifact not found: $WASM_PATH" >&2
  echo "build it first via scripts/build-web.sh" >&2
  exit 1
fi

declare -A BUDGET
declare -A METRICS

parse_key_value_file() {
  local path="$1"
  local -n target="$2"
  local line key value

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [[ -z "$line" || "${line:0:1}" == "#" ]] && continue
    if [[ "$line" != *=* ]]; then
      echo "invalid line in $path: $line" >&2
      exit 1
    fi
    key="${line%%=*}"
    value="${line#*=}"
    target["$key"]="$value"
  done < "$path"
}

require_key() {
  local -n target="$1"
  local key="$2"
  local value="${target[$key]-}"
  if [[ -z "${value:-}" ]]; then
    echo "missing required key '$key'" >&2
    exit 1
  fi
}

require_non_negative_integer() {
  local label="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "$label must be a non-negative integer, got '$value'" >&2
    exit 1
  fi
}

check_metric() {
  local metric_name="$1"
  local actual="$2"
  local max_allowed="$3"
  local -n failure_list_ref="$4"

  printf '  %-24s actual=%-8s max=%s\n' "$metric_name" "$actual" "$max_allowed"
  if (( actual > max_allowed )); then
    failure_list_ref+=$'\n'"- ${metric_name} exceeded budget (${actual} > ${max_allowed})"
  fi
}

parse_key_value_file "$budget_path" BUDGET
parse_key_value_file "$metrics_path" METRICS

for key in \
  metric_version \
  wasm_bytes_max \
  worker_boot_ms_max \
  compile_diagnostics_ms_max \
  hello_run_ms_max \
  worker_boot_and_run_ms_max \
  gc_churn_run_ms_max \
  module_load_ms_max \
  hello_memory_bytes_max \
  gc_churn_memory_bytes_max \
  module_load_memory_bytes_max \
  module_cache_storage_delta_bytes_max \
  shell_ready_ms_max \
  shell_run_ms_max; do
  require_key BUDGET "$key"
done

for key in \
  metric_version \
  worker_boot_ms \
  compile_diagnostics_ms \
  hello_run_ms \
  worker_boot_and_run_ms \
  gc_churn_run_ms \
  module_load_ms \
  hello_memory_bytes \
  gc_churn_memory_bytes \
  module_load_memory_bytes \
  module_cache_storage_delta_bytes \
  shell_ready_ms \
  shell_run_ms; do
  require_key METRICS "$key"
done

if [[ "${BUDGET[metric_version]}" != "3" ]]; then
  echo "unsupported budget metric_version: ${BUDGET[metric_version]}" >&2
  exit 1
fi

if [[ "${METRICS[metric_version]}" != "${BUDGET[metric_version]}" ]]; then
  echo "metric_version mismatch: budget=${BUDGET[metric_version]} metrics=${METRICS[metric_version]}" >&2
  exit 1
fi

for key in \
  wasm_bytes_max \
  worker_boot_ms_max \
  compile_diagnostics_ms_max \
  hello_run_ms_max \
  worker_boot_and_run_ms_max \
  gc_churn_run_ms_max \
  module_load_ms_max \
  hello_memory_bytes_max \
  gc_churn_memory_bytes_max \
  module_load_memory_bytes_max \
  module_cache_storage_delta_bytes_max \
  shell_ready_ms_max \
  shell_run_ms_max; do
  require_non_negative_integer "$key" "${BUDGET[$key]}"
done

for key in \
  worker_boot_ms \
  compile_diagnostics_ms \
  hello_run_ms \
  worker_boot_and_run_ms \
  gc_churn_run_ms \
  module_load_ms \
  hello_memory_bytes \
  gc_churn_memory_bytes \
  module_load_memory_bytes \
  module_cache_storage_delta_bytes \
  shell_ready_ms \
  shell_run_ms; do
  require_non_negative_integer "$key" "${METRICS[$key]}"
done

actual_wasm_bytes="$(wc -c < "$WASM_PATH" | tr -d '[:space:]')"
require_non_negative_integer "wasm_bytes" "$actual_wasm_bytes"

if [[ -n "${METRICS[wasm_bytes]-}" ]]; then
  require_non_negative_integer "metrics wasm_bytes" "${METRICS[wasm_bytes]}"
  if [[ "${METRICS[wasm_bytes]}" != "$actual_wasm_bytes" ]]; then
    echo "metrics wasm_bytes (${METRICS[wasm_bytes]}) did not match artifact size ($actual_wasm_bytes)" >&2
    exit 1
  fi
fi

echo "checking browser performance budgets"
echo "  budget file:  $budget_path"
echo "  metrics file: $metrics_path"
echo "  wasm path:    $WASM_PATH"

failures=""
check_metric "wasm_bytes" "$actual_wasm_bytes" "${BUDGET[wasm_bytes_max]}" failures
check_metric "worker_boot_ms" "${METRICS[worker_boot_ms]}" "${BUDGET[worker_boot_ms_max]}" failures
check_metric \
  "compile_diagnostics_ms" \
  "${METRICS[compile_diagnostics_ms]}" \
  "${BUDGET[compile_diagnostics_ms_max]}" \
  failures
check_metric "hello_run_ms" "${METRICS[hello_run_ms]}" "${BUDGET[hello_run_ms_max]}" failures
check_metric \
  "worker_boot_and_run_ms" \
  "${METRICS[worker_boot_and_run_ms]}" \
  "${BUDGET[worker_boot_and_run_ms_max]}" \
  failures
check_metric \
  "gc_churn_run_ms" \
  "${METRICS[gc_churn_run_ms]}" \
  "${BUDGET[gc_churn_run_ms_max]}" \
  failures
check_metric \
  "module_load_ms" \
  "${METRICS[module_load_ms]}" \
  "${BUDGET[module_load_ms_max]}" \
  failures
check_metric \
  "hello_memory_bytes" \
  "${METRICS[hello_memory_bytes]}" \
  "${BUDGET[hello_memory_bytes_max]}" \
  failures
check_metric \
  "gc_churn_memory_bytes" \
  "${METRICS[gc_churn_memory_bytes]}" \
  "${BUDGET[gc_churn_memory_bytes_max]}" \
  failures
check_metric \
  "module_load_memory_bytes" \
  "${METRICS[module_load_memory_bytes]}" \
  "${BUDGET[module_load_memory_bytes_max]}" \
  failures
check_metric \
  "module_cache_storage_delta_bytes" \
  "${METRICS[module_cache_storage_delta_bytes]}" \
  "${BUDGET[module_cache_storage_delta_bytes_max]}" \
  failures
check_metric \
  "shell_ready_ms" \
  "${METRICS[shell_ready_ms]}" \
  "${BUDGET[shell_ready_ms_max]}" \
  failures
check_metric \
  "shell_run_ms" \
  "${METRICS[shell_run_ms]}" \
  "${BUDGET[shell_run_ms_max]}" \
  failures

if [[ -n "$failures" ]]; then
  echo
  echo "browser performance regression gate failed:$failures" >&2
  exit 1
fi

echo
echo "browser performance regression gate passed"

if [[ -n "$report_output_path" ]]; then
  report_budget_path="$budget_path"
  if [[ "$budget_path" == "$DEFAULT_BUDGET_PATH" ]]; then
    report_budget_path="testdata/web-performance-budget.env"
  fi
  report_args=(
    python3 "$ROOT/scripts/generate-web-performance-report.py"
    --budget "$report_budget_path"
    --metrics "$metrics_path"
    --output "$report_output_path"
  )
  if [[ -n "$baseline_metrics_path" ]]; then
    report_args+=(--baseline-metrics "$baseline_metrics_path")
  fi
  "${report_args[@]}"
fi
