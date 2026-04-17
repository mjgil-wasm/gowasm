#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

if [[ "${1:-}" == "--list-targets" ]]; then
  python3 - "$repo_root/testdata/fuzz-harness/index.json" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

for target in payload["targets"]:
    print(target["name"])
PY
  exit 0
fi

cd "$repo_root"
cargo test --release -p gowasm-engine tests_fuzz_harness -- --nocapture
