#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

exec python3 \
  "$repo_root/scripts/run-native-go-stdlib-corpus.py" \
  --index "$repo_root/testdata/stdlib-differential/index.json" \
  "$@"
