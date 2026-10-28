#!/bin/bash
# Tasklynx Installer for Linux/macOS/Android (Termux)
set -e

PROJECT_NAME="tasklynx"
VERSION_URL="https://raw.githubusercontent.com/peekstore/${PROJECT_NAME}/main/.version"
RELEASE_URL="https://github.com/peekstore/${PROJECT_NAME}/releases/download"

VERSION=$(curl -sL "$VERSION_URL" | tr -d '[:space:]' || echo "")
[ -z "$VERSION" ] && VERSION="v0.0.1-beta-1"
[[ "$VERSION" != v* ]] && VERSION="v${VERSION}"

echo ">>> Installing ${PROJECT_NAME} ${VERSION}..."
echo ""

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

if [ -n "${TERMUX_VERSION:-}" ] || [ -d "/data/data/com.termux" ]; then
    OS="android"
fi

case "$OS" in
    linux|android) DOWNLOAD_NAME="${PROJECT_NAME}-linux-${ARCH}" ;;
    darwin) DOWNLOAD_NAME="${PROJECT_NAME}-darwin-${ARCH}" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

DOWNLOAD_URL="${RELEASE_URL}/${VERSION}/${DOWNLOAD_NAME}"

if [ -n "$XDG_CONFIG_HOME" ]; then
    BIN_DIR="$XDG_CONFIG_HOME/neostore/${PROJECT_NAME}/bin"
else
    BIN_DIR="$HOME/.config/neostore/${PROJECT_NAME}/bin"
fi
mkdir -p "$BIN_DIR"

echo "Downloading ${DOWNLOAD_URL}"
curl -sL -o "${BIN_DIR}/${PROJECT_NAME}" "$DOWNLOAD_URL" || {
    echo "Error: Failed to download ${PROJECT_NAME}"
    exit 1
}
chmod +x "${BIN_DIR}/${PROJECT_NAME}"
[ -f "${BIN_DIR}/${PROJECT_NAME}" ] || { echo "Error: Download failed"; exit 1; }
ln -sf "${BIN_DIR}/${PROJECT_NAME}" "${BIN_DIR}/tx" 2>/dev/null

SIZE=$(du -k "${BIN_DIR}/${PROJECT_NAME}" | cut -f1)
echo "OK   Installed to ${BIN_DIR}/${PROJECT_NAME} (${SIZE} KB)"

SHELL_RC=""
case "$SHELL" in
    */bash) SHELL_RC="$HOME/.bashrc" ;;
    */zsh)  SHELL_RC="$HOME/.zshrc" ;;
    */fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
    *)      SHELL_RC="$HOME/.profile" ;;
esac

if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    echo "Adding to PATH..."
    if [ "$SHELL" = "*/fish" ] || [ "$(basename "$SHELL")" = "fish" ]; then
        echo "set -gx PATH $BIN_DIR \$PATH" >> "$SHELL_RC"
    else
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$SHELL_RC"
    fi
    echo "PATH updated. Run 'source $SHELL_RC' or restart your terminal."
else
    echo "Already in PATH"
fi

echo ""
echo "OK   ${PROJECT_NAME} ${VERSION} installed successfully!"
echo "Run '${PROJECT_NAME} --help' or 'tx --help' to get started"
