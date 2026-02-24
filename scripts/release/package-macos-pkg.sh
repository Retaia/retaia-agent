#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?tag required (ex: v1.0.0)}"
VERSION="${TAG#v}"
ARCH="$(echo "${RUNNER_ARCH:-x64}" | tr '[:upper:]' '[:lower:]')"
OUT_DIR="release-assets"
PKG_ROOT="${OUT_DIR}/pkgroot"
BIN_DIR="target/release"

rm -rf "${PKG_ROOT}"
mkdir -p "${PKG_ROOT}/usr/local/bin"

install -m 0755 "${BIN_DIR}/agentctl" "${PKG_ROOT}/usr/local/bin/agentctl"
install -m 0755 "${BIN_DIR}/agent-runtime" "${PKG_ROOT}/usr/local/bin/agent-runtime"
install -m 0755 "${BIN_DIR}/agent-desktop-shell" "${PKG_ROOT}/usr/local/bin/agent-desktop-shell"

pkgbuild \
  --root "${PKG_ROOT}" \
  --identifier "com.retaia.agent" \
  --version "${VERSION}" \
  --install-location "/" \
  "${OUT_DIR}/retaia-agent-${TAG}-macos-${ARCH}.pkg"

echo "Built: ${OUT_DIR}/retaia-agent-${TAG}-macos-${ARCH}.pkg"
