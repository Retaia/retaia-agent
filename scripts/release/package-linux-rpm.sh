#!/usr/bin/env bash
set -euo pipefail

TAG="${1:?tag required (ex: v1.0.0)}"
VERSION="${TAG#v}"
ARCH_RAW="$(echo "${RUNNER_ARCH:-x64}" | tr '[:upper:]' '[:lower:]')"

case "${ARCH_RAW}" in
  x64) RPM_ARCH="x86_64" ;;
  arm64) RPM_ARCH="aarch64" ;;
  *) RPM_ARCH="${ARCH_RAW}" ;;
esac

PKG_NAME="retaia-agent"
OUT_DIR="release-assets"
TOPDIR="${PWD}/${OUT_DIR}/rpmbuild"
ICON_SRC="assets/icon/retaia-logo-512.png"

rm -rf "${TOPDIR}"
mkdir -p \
  "${TOPDIR}/BUILD" \
  "${TOPDIR}/BUILDROOT" \
  "${TOPDIR}/RPMS" \
  "${TOPDIR}/SOURCES" \
  "${TOPDIR}/SPECS" \
  "${TOPDIR}/SRPMS"

cat > "${TOPDIR}/SPECS/${PKG_NAME}.spec" <<SPEC
Name:           ${PKG_NAME}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Retaia Agent runtime, CLI and desktop shell
License:        Proprietary
BuildArch:      ${RPM_ARCH}

%description
Retaia Agent runtime, CLI and desktop shell.

%install
mkdir -p %{buildroot}/usr/local/bin
install -m 0755 target/release/agentctl %{buildroot}/usr/local/bin/agentctl
install -m 0755 target/release/agent-runtime %{buildroot}/usr/local/bin/agent-runtime
install -m 0755 target/release/agent-desktop-shell %{buildroot}/usr/local/bin/agent-desktop-shell

mkdir -p %{buildroot}/usr/share/icons/hicolor/512x512/apps
install -m 0644 ${ICON_SRC} %{buildroot}/usr/share/icons/hicolor/512x512/apps/retaia-agent.png

mkdir -p %{buildroot}/usr/share/applications
cat > %{buildroot}/usr/share/applications/retaia-agent.desktop <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Retaia Agent
Comment=Retaia Agent Control Center
Exec=/usr/local/bin/agent-desktop-shell
Icon=retaia-agent
Terminal=false
Categories=Utility;
DESKTOP

%files
/usr/local/bin/agentctl
/usr/local/bin/agent-runtime
/usr/local/bin/agent-desktop-shell
/usr/share/icons/hicolor/512x512/apps/retaia-agent.png
/usr/share/applications/retaia-agent.desktop
SPEC

rpmbuild \
  --define "_topdir ${TOPDIR}" \
  --define "_build_id_links none" \
  -bb "${TOPDIR}/SPECS/${PKG_NAME}.spec"

RPM_PATH="${TOPDIR}/RPMS/${RPM_ARCH}/${PKG_NAME}-${VERSION}-1.${RPM_ARCH}.rpm"
TARGET_PATH="${OUT_DIR}/${PKG_NAME}-${TAG}-linux-${RPM_ARCH}.rpm"
cp "${RPM_PATH}" "${TARGET_PATH}"

echo "Built: ${TARGET_PATH}"
