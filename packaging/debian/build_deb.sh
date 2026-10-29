#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.0.1-beta-2}"
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

dpkg-deb --build --root-owner-group "$PKG_DIR" "${OUTPUT_DIR}/tasklynx_${CLEAN_VERSION}_${ARCH}.deb"
echo "Built ${OUTPUT_DIR}/tasklynx_${CLEAN_VERSION}_${ARCH}.deb"
rm -rf "$PKG_DIR"
