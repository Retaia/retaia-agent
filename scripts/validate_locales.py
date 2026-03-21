#!/usr/bin/env python3

import json
import sys
from pathlib import Path


def load_locale(path: Path) -> dict[str, str]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SystemExit(f"{path}: invalid json: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit(f"{path}: root must be a JSON object")
    for key, value in payload.items():
        if not isinstance(key, str):
            raise SystemExit(f"{path}: locale keys must be strings")
        if not isinstance(value, str):
            raise SystemExit(f"{path}: locale value for '{key}' must be a string")
        if not value.strip():
            raise SystemExit(f"{path}: locale value for '{key}' must be non-empty")
    return payload


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    en_path = root / "locales" / "en.json"
    fr_path = root / "locales" / "fr.json"

    en = load_locale(en_path)
    fr = load_locale(fr_path)

    en_keys = set(en.keys())
    fr_keys = set(fr.keys())
    if en_keys != fr_keys:
        missing_in_fr = sorted(en_keys - fr_keys)
        missing_in_en = sorted(fr_keys - en_keys)
        if missing_in_fr:
            print(f"missing in fr.json: {', '.join(missing_in_fr)}", file=sys.stderr)
        if missing_in_en:
            print(f"missing in en.json: {', '.join(missing_in_en)}", file=sys.stderr)
        return 1

    print(f"locale validation ok: {len(en_keys)} keys in en/fr")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
