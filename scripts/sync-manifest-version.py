#!/usr/bin/env python3
"""Synchronize template manifest versions with the release tag."""

import argparse
import json
import re
import sys
from pathlib import Path


SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")


def load_manifest(path):
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def write_manifest(path, manifest):
    original = path.read_text(encoding="utf-8")
    updated, count = re.subn(
        r'("version"\s*:\s*")[^"]*(")',
        rf"\g<1>{manifest['version']}\2",
        original,
        count=1,
    )
    if count != 1:
        raise ValueError(f"missing version field in {path}")
    json.loads(updated)
    path.write_text(updated, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Sync manifest.json version fields")
    parser.add_argument("version", help="Release version, with or without leading v")
    parser.add_argument("manifests", nargs="+", help="manifest.json files to update")
    parser.add_argument("--check", action="store_true", help="Only validate; do not write files")
    args = parser.parse_args()

    version = args.version.removeprefix("v")
    if not SEMVER_RE.match(version):
        print(f"invalid release version: {args.version}", file=sys.stderr)
        return 2

    failures = []
    for raw_path in args.manifests:
        path = Path(raw_path)
        manifest = load_manifest(path)
        current = manifest.get("version", "")
        name = manifest.get("name", path.parent.name)

        if current == version:
            print(f"{name}: manifest version already {version}")
            continue

        if args.check:
            failures.append(f"{name}: manifest version {current or '<empty>'} != {version}")
            continue

        manifest["version"] = version
        write_manifest(path, manifest)
        print(f"{name}: manifest version {current or '<empty>'} -> {version}")

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
