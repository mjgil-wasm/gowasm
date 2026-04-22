#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from release_artifact_metadata import (
    CHECKED_METADATA_PATH,
    DEFAULT_ARTIFACT_PATH,
    build_metadata,
    read_json,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Compare the current browser Wasm artifact metadata against the checked "
            "reproducibility snapshot."
        )
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
    parser.add_argument(
        "--checked-metadata",
        default=str(CHECKED_METADATA_PATH),
        help="Optional checked metadata JSON override relative to the chosen root.",
    )
    parser.add_argument(
        "--print-metadata",
        action="store_true",
        help="Print the current metadata and exit.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    artifact = (root / args.artifact).resolve()
    checked_metadata_path = (root / args.checked_metadata).resolve()

    generated = build_metadata(root, artifact)
    if args.print_metadata:
        json.dump(generated, sys.stdout, indent=2, sort_keys=True)
        sys.stdout.write("\n")
        return 0

    checked = read_json(checked_metadata_path)
    if generated != checked:
        print(
            "release artifact metadata is stale; rebuild the artifact with "
            "`scripts/build-web.sh` and refresh the checked snapshot with "
            "`python3 scripts/generate-release-artifact-metadata.py > "
            "docs/generated/release-artifact-metadata.json`",
            file=sys.stderr,
        )
        print(
            json.dumps({"expected": generated, "checked": checked}, indent=2, sort_keys=True),
            file=sys.stderr,
        )
        return 1

    print("release artifact reproducibility metadata passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
