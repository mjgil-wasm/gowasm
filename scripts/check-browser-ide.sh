#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

python3 "$repo_root/scripts/browser_page_runner.py" \
  --page web/ide/index.html \
  --page-query "ci=1&ci_compile_smoke=1" \
  --element-id ci-summary \
  --expect-substring "all IDE compile tests passed" \
  --reject-substring "failed" \
  --timeout-seconds 180 \
  "$@"
