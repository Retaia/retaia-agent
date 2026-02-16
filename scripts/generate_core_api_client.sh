#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/crates/retaia-core-client"

rm -rf "$OUT_DIR"

docker run --rm \
  -u "$(id -u):$(id -g)" \
  -v "$ROOT_DIR:/local" \
  openapitools/openapi-generator-cli generate \
  -i /local/specs/api/openapi/v1.yaml \
  -g rust \
  -o /local/crates/retaia-core-client \
  --additional-properties=library=reqwest-trait,supportAsync=true,packageName=retaia_core_client,packageVersion=0.1.0

rm -rf "$OUT_DIR/target" "$OUT_DIR/.travis.yml" "$OUT_DIR/git_push.sh"

echo "Generated Rust OpenAPI client at $OUT_DIR"
