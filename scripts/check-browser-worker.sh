#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

python3 "$repo_root/scripts/run-browser-acceptance-corpus.py" \
  --index "$repo_root/testdata/browser-acceptance/index.json" \
  --tag worker \
  "$@"
