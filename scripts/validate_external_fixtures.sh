#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
MANIFEST_PATH="${1:-$ROOT_DIR/fixtures/external/manifest.tsv}"

if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "manifest not found: $MANIFEST_PATH" >&2
  exit 1
fi

sha256_file() {
  local file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
    return
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
    return
  fi
  echo "no sha256 tool available (need shasum or sha256sum)" >&2
  exit 2
}

is_valid_kind() {
  case "$1" in
    raw_photo|preview_video|preview_audio) return 0 ;;
    *) return 1 ;;
  esac
}

is_valid_expected() {
  case "$1" in
    supported|unsupported|negative) return 0 ;;
    *) return 1 ;;
  esac
}

entries=0
errors=0

while IFS=$'\t' read -r rel sha kind expected notes || [[ -n "${rel:-}" ]]; do
  [[ -z "${rel:-}" ]] && continue
  [[ "${rel:0:1}" == "#" ]] && continue
  [[ "$rel" == "relative_path" ]] && continue

  entries=$((entries + 1))
  file_path="$ROOT_DIR/fixtures/external/$rel"

  if [[ ! -f "$file_path" ]]; then
    echo "[ERROR] missing file: fixtures/external/$rel"
    errors=$((errors + 1))
    continue
  fi

  if [[ -z "${sha:-}" || "$sha" == "<sha256>" ]]; then
    echo "[ERROR] sha256 missing placeholder unresolved: fixtures/external/$rel"
    errors=$((errors + 1))
    continue
  fi

  if ! is_valid_kind "${kind:-}"; then
    echo "[ERROR] invalid kind '${kind:-}' for fixtures/external/$rel"
    errors=$((errors + 1))
    continue
  fi

  if ! is_valid_expected "${expected:-}"; then
    echo "[ERROR] invalid expected '${expected:-}' for fixtures/external/$rel"
    errors=$((errors + 1))
    continue
  fi

  actual_sha="$(sha256_file "$file_path")"
  if [[ "$actual_sha" != "$sha" ]]; then
    echo "[ERROR] sha256 mismatch for fixtures/external/$rel"
    echo "        expected: $sha"
    echo "        actual:   $actual_sha"
    errors=$((errors + 1))
    continue
  fi

  echo "[OK] fixtures/external/$rel ($kind, $expected)"
done < "$MANIFEST_PATH"

if [[ "$entries" -eq 0 ]]; then
  echo "[WARN] manifest has no active entries: $MANIFEST_PATH"
fi

if [[ "$errors" -ne 0 ]]; then
  echo "validation failed: $errors error(s)"
  exit 1
fi

echo "validation passed: $entries entrie(s)"
