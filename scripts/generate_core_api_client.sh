#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/crates/retaia-core-client"
DOCS_OUT_DIR="$ROOT_DIR/docs/api"
SPEC_PATH="specs/api/openapi/v1.yaml"
OUT_DIR_REL="crates/retaia-core-client"
DERIVED_API_PATH="$OUT_DIR/src/apis/derived_api.rs"

apply_known_generated_client_fixes() {
  if [[ ! -f "$DERIVED_API_PATH" ]]; then
    echo "Expected generated file missing: $DERIVED_API_PATH" >&2
    exit 1
  fi

  python3 - <<'PY' "$DERIVED_API_PATH"
from pathlib import Path
import sys

path = Path(sys.argv[1])
old = """        if let Some(ref path) = chunk {\n            local_var_form = local_var_form.file(\"chunk\", path.as_os_str()).await?;\n        }\n"""
new = """        local_var_form = local_var_form.file(\"chunk\", &chunk).await?;\n"""
text = path.read_text()

if old in text:
    path.write_text(text.replace(old, new))
    print(f"Applied post-generation fix in {path}")
else:
    print(f"No post-generation fix needed in {path}")
PY
}

rm -rf "$OUT_DIR"
rm -rf "$DOCS_OUT_DIR"/*.md

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  docker run --rm \
    -u "$(id -u):$(id -g)" \
    -v "$ROOT_DIR:/local" \
    openapitools/openapi-generator-cli validate \
    -i /local/specs/api/openapi/v1.yaml

  docker run --rm \
    -u "$(id -u):$(id -g)" \
    -v "$ROOT_DIR:/local" \
    openapitools/openapi-generator-cli generate \
    -i /local/specs/api/openapi/v1.yaml \
    -g rust \
    -o /local/crates/retaia-core-client \
    --additional-properties=library=reqwest-trait,supportAsync=true,packageName=retaia_core_client,packageVersion=0.1.0
else
  (
    cd "$ROOT_DIR"
    npx -y @openapitools/openapi-generator-cli validate \
      -i "$SPEC_PATH"

    npx -y @openapitools/openapi-generator-cli generate \
      -i "$SPEC_PATH" \
      -g rust \
      -o "$OUT_DIR_REL" \
      --additional-properties=library=reqwest-trait,supportAsync=true,packageName=retaia_core_client,packageVersion=0.1.0
  )
fi

rm -rf "$OUT_DIR/target" "$OUT_DIR/.travis.yml" "$OUT_DIR/git_push.sh"
mkdir -p "$DOCS_OUT_DIR"
mv "$OUT_DIR/docs" "$DOCS_OUT_DIR"
mv "$DOCS_OUT_DIR/docs"/*.md "$DOCS_OUT_DIR/"
rmdir "$DOCS_OUT_DIR/docs"

apply_known_generated_client_fixes

(
  cd "$ROOT_DIR"
  cargo test --features core-api-client --tests --no-run
)

echo "Generated Rust OpenAPI client at $OUT_DIR"
echo "Generated API docs at $DOCS_OUT_DIR"
