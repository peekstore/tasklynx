#!/usr/bin/env bash
# installer.sh - Installer and Uninstaller for tasklynx
set -e

PROJECT_NAME="tasklynx"
GITHUB_REPO="peekstore/tasklynx"
BIN_DIR="${HOME}/.local/bin"
BINARY_PATH="${BIN_DIR}/${PROJECT_NAME}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; CYAN='\033[0;36m'; NC='\033[0m'

if [ "${1:-}" = "--selfuninstall" ] || [ "${1:-}" = "-u" ] || [ "${1:-}" = "--uninstall" ]; then
    echo -e "\n${YELLOW}tasklynx Uninstaller${NC}\n"
    rm -f "$BINARY_PATH" "${BIN_DIR}/tx" 2>/dev/null && echo -e "  ${GREEN}Removed${NC}"
    if command -v nix-env >/dev/null 2>&1 && nix-env -q 2>/dev/null | grep -q tasklynx; then
        nix-env -e tasklynx 2>/dev/null || true
    fi
    echo -e "\n${GREEN}tasklynx uninstalled.${NC}\n"
    exit 0
fi

echo -e "\n${CYAN}tasklynx Installer${NC}\n"

OS_NAME=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS_NAME" in
    linux*)
        if [ -n "${TERMUX_VERSION:-}" ] || [ -d "/data/data/com.termux" ]; then OS="android"
        elif [ -f /etc/NIXOS ]; then OS="linux"
        else OS="linux"; fi
        ;;
    darwin*) OS="darwin" ;;
    *) echo -e "${RED}Unsupported OS${NC}"; exit 1 ;;
esac

ARCH_NAME=$(uname -m)
case "$ARCH_NAME" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo -e "${RED}Unsupported arch${NC}"; exit 1 ;;
esac

printf "  Platform: %s-%s\n" "$OS" "$ARCH"

VERSION="v0.0.1-beta-1"
if command -v curl >/dev/null 2>&1; then
    V=$(curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | cut -d'"' -f4 | head -1 || true)
    [ -n "$V" ] && VERSION="$V"
fi

URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/tasklynx-${OS}-${ARCH}"
echo -e "  Downloading from: ${URL}"
mkdir -p "$BIN_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -fL -o "$BINARY_PATH" "$URL" || { echo -e "${RED}Failed${NC}"; exit 1; }
else
    wget -O "$BINARY_PATH" "$URL" || { echo -e "${RED}Failed${NC}"; exit 1; }
fi

chmod +x "$BINARY_PATH"
ln -sf "$BINARY_PATH" "${BIN_DIR}/tx" 2>/dev/null || cp "$BINARY_PATH" "${BIN_DIR}/tx" && chmod +x "${BIN_DIR}/tx"

export PATH="$HOME/.local/bin:$PATH"
grep -q '.local/bin' "$HOME/.bashrc" 2>/dev/null || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
grep -q '.local/bin' "$HOME/.zshrc" 2>/dev/null || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"

echo -e "\n${GREEN}✓ Installed to $BINARY_PATH${NC}"
echo -e "${GREEN}✓ Created tx alias${NC}\n"
