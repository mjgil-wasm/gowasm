#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from release_artifact_metadata import DEFAULT_ARTIFACT_PATH, build_metadata


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate reproducible metadata for the checked browser Wasm artifact."
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Optional repository root override.",
    )
    parser.add_argument(
        "--artifact",
        default=str(DEFAULT_ARTIFACT_PATH),
        help="Optional artifact path override relative to the chosen root.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    artifact = (root / args.artifact).resolve()
    json.dump(build_metadata(root, artifact), sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
