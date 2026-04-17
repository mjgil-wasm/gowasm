#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
soak_artifact_path="${TMPDIR:-/tmp}/gowasm-browser-shell-soak-artifact.json"

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/test-browser-capability-security.html \
  --element-id summary \
  --expect-substring "all browser capability security tests passed" \
  --reject-substring "failure" \
  --timeout-seconds 240 \
  "$@"

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/test-browser-compatibility.html \
  --element-id summary \
  --expect-substring "all browser compatibility tests passed" \
  --reject-substring "failure" \
  --timeout-seconds 120 \
  "$@"

python3 "$repo_root/scripts/run-browser-acceptance-corpus.py" \
  --index "$repo_root/testdata/browser-acceptance/index.json" \
  --tag shell \
  "$@"

if ! python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/test-browser-shell-soak.html \
  --element-id summary \
  --artifact-element-id artifact \
  --artifact-output "$soak_artifact_path" \
  --expect-substring "all browser shell soak tests passed" \
  --reject-substring "FAIL" \
  --timeout-seconds 360 \
  "$@"; then
  if [[ -f "$soak_artifact_path" ]]; then
    echo "browser shell soak replay artifact: $soak_artifact_path" >&2
    cat "$soak_artifact_path" >&2
  fi
  exit 1
fi

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/ci-browser-shell-smoke.html \
  --element-id summary \
  --expect-substring "all browser shell ci smoke tests passed" \
  --reject-substring "FAIL" \
  --timeout-seconds 180 \
  "$@"

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/test-browser-shell-examples.html \
  --element-id summary \
  --expect-substring "all packaged browser example tests passed" \
  --reject-substring "failure" \
  --timeout-seconds 240 \
  "$@"
