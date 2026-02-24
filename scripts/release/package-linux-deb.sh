#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?tag required (ex: v1.0.0)}"
VERSION="${TAG#v}"
ARCH_RAW="$(echo "${RUNNER_ARCH:-x64}" | tr '[:upper:]' '[:lower:]')"

case "${ARCH_RAW}" in
  x64) DEB_ARCH="amd64" ;;
  arm64) DEB_ARCH="arm64" ;;
  *) DEB_ARCH="${ARCH_RAW}" ;;
esac

OUT_DIR="release-assets"
PKG_NAME="retaia-agent"
ROOT="${OUT_DIR}/${PKG_NAME}_${VERSION}_${DEB_ARCH}"
BIN_DIR="target/release"
ICON_SRC="assets/icon/retaia-logo-512.png"

rm -rf "${ROOT}"
mkdir -p \
  "${ROOT}/DEBIAN" \
  "${ROOT}/usr/local/bin" \
  "${ROOT}/usr/share/icons/hicolor/512x512/apps" \
  "${ROOT}/usr/share/applications"

cat > "${ROOT}/DEBIAN/control" <<CONTROL
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: Retaia <support@retaia.com>
Description: Retaia Agent runtime, CLI and desktop shell
CONTROL

install -m 0755 "${BIN_DIR}/agentctl" "${ROOT}/usr/local/bin/agentctl"
install -m 0755 "${BIN_DIR}/agent-runtime" "${ROOT}/usr/local/bin/agent-runtime"
install -m 0755 "${BIN_DIR}/agent-desktop-shell" "${ROOT}/usr/local/bin/agent-desktop-shell"
install -m 0644 "${ICON_SRC}" "${ROOT}/usr/share/icons/hicolor/512x512/apps/retaia-agent.png"

cat > "${ROOT}/usr/share/applications/retaia-agent.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Retaia Agent
Comment=Retaia Agent Control Center
Exec=/usr/local/bin/agent-desktop-shell
Icon=retaia-agent
Terminal=false
Categories=Utility;
DESKTOP

dpkg-deb --build --root-owner-group "${ROOT}" "${OUT_DIR}/${PKG_NAME}-${TAG}-linux-${DEB_ARCH}.deb"

echo "Built: ${OUT_DIR}/${PKG_NAME}-${TAG}-linux-${DEB_ARCH}.deb"
