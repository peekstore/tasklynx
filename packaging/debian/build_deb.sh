#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.0.1-beta-1}"
ARCH="${2:-amd64}"
BINARY="${3:-target/release/tasklynx}"
OUTPUT_DIR="${4:-.}"

CLEAN_VERSION="${VERSION#v}"
PKG_DIR="/tmp/tasklynx-deb-${ARCH}"
rm -rf "$PKG_DIR"

mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/doc/tasklynx"

cp "$BINARY" "${PKG_DIR}/usr/bin/tasklynx"
chmod 755 "${PKG_DIR}/usr/bin/tasklynx"
ln -sf tasklynx "${PKG_DIR}/usr/bin/tx"

cat <<EOF > "${PKG_DIR}/DEBIAN/control"
Package: tasklynx
Version: ${CLEAN_VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: RK Riad Khan <rkriad585@gmail.com>
Description: Developer-first local task manager with Git-style workflows
 Tasklynx is a local-first task manager with Git-style branch workflows,
 terminal-native Ratatui dashboard, and embedded web interface.
EOF

OUTPUT_DEB="${OUTPUT_DIR}/tasklynx_${CLEAN_VERSION}_${ARCH}.deb"

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUTPUT_DEB"
else
    # Fallback to standard ar + tar for non-Debian environments
    BUILD_STAGE=$(mktemp -d)
    echo "2.0" > "${BUILD_STAGE}/debian-binary"
    tar --owner=0 --group=0 -czf "${BUILD_STAGE}/control.tar.gz" -C "${PKG_DIR}/DEBIAN" .
    tar --owner=0 --group=0 -czf "${BUILD_STAGE}/data.tar.gz" -C "${PKG_DIR}" --exclude="./DEBIAN" usr
    ar rcs "$OUTPUT_DEB" "${BUILD_STAGE}/debian-binary" "${BUILD_STAGE}/control.tar.gz" "${BUILD_STAGE}/data.tar.gz"
    rm -rf "$BUILD_STAGE"
fi

echo "Built ${OUTPUT_DEB}"
rm -rf "$PKG_DIR"
