#!/usr/bin/env python3

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = ROOT / "scripts" / "duplicate_deps_allowlist.txt"


def cargo_metadata() -> dict:
    output = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1"],
        cwd=ROOT,
        text=True,
    )
    return json.loads(output)


def current_duplicates() -> list[str]:
    metadata = cargo_metadata()
    versions_by_name: dict[str, set[str]] = {}
    for package in metadata["packages"]:
        versions_by_name.setdefault(package["name"], set()).add(package["version"])

    duplicates = []
    for name, versions in sorted(versions_by_name.items()):
        if len(versions) > 1:
            duplicates.append(f"{name}: {', '.join(sorted(versions))}")
    return duplicates


def load_allowlist() -> list[str]:
    lines = []
    for raw_line in ALLOWLIST.read_text().splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        lines.append(line)
    return lines


def main() -> int:
    actual = current_duplicates()
    expected = load_allowlist()

    actual_set = set(actual)
    expected_set = set(expected)

    missing = sorted(expected_set - actual_set)
    unexpected = sorted(actual_set - expected_set)

    if not missing and not unexpected:
        print("Duplicate dependency set matches allowlist.")
        return 0

    if unexpected:
        print("New duplicate dependencies detected:")
        for item in unexpected:
            print(f"  + {item}")

    if missing:
        print("Allowlist contains duplicates no longer present:")
        for item in missing:
            print(f"  - {item}")

    print()
    print("Update scripts/duplicate_deps_allowlist.txt intentionally if this change is accepted.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
