#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

output_path=""
report_output_path=""
baseline_metrics_path=""

usage() {
  cat <<'EOF'
usage: scripts/capture-browser-performance.sh --output path/to/browser-metrics.env [--report-output path/to/report.json] [--baseline-metrics path/to/previous-metrics.env]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      output_path="$2"
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

if [[ -z "$output_path" ]]; then
  echo "missing required --output path" >&2
  usage >&2
  exit 2
fi

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/measure-worker.html \
  --element-id metrics \
  --expect-substring "metric_version=3" \
  --reject-substring "error=" \
  --output "$output_path" \
  --timeout-seconds 180

if [[ -n "$report_output_path" ]]; then
  report_args=(
    python3 "$repo_root/scripts/generate-web-performance-report.py"
    --budget "testdata/web-performance-budget.env"
    --metrics "$output_path"
    --output "$report_output_path"
  )
  if [[ -n "$baseline_metrics_path" ]]; then
    report_args+=(--baseline-metrics "$baseline_metrics_path")
  fi
  "${report_args[@]}"
fi
