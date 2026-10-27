#!/usr/bin/env bash
set -euo pipefail
REPO="rkriad585/tasklynx"
VERSION="${TASKLYNX_VERSION:-latest}"
BASE="https://github.com/${REPO}/releases"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"; ARCH="$(uname -m)"
case "$ARCH" in x86_64) ARCH=amd64;; aarch64|arm64) ARCH=arm64;; esac
case "$OS" in linux) OS=linux;; darwin) OS=darwin;; *) echo "Unsupported OS: $OS"; exit 1;; esac
if [ "$VERSION" = latest ]; then VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -1); fi
URL="${BASE}/download/${VERSION}/tasklynx-${OS}-${ARCH}"
TMP=$(mktemp); trap 'rm -f "$TMP"' EXIT
curl -fL "$URL" -o "$TMP"; chmod +x "$TMP"; sudo install -m 0755 "$TMP" /usr/local/bin/tasklynx
printf 'Installed tasklynx %s\n' "$VERSION"
