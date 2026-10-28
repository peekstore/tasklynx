#!/usr/bin/env bash
# installer.sh - Installer and Uninstaller for tasklynx
# Supports Linux, macOS, NixOS, and Android Termux

set -euo pipefail

PROJECT_NAME="tasklynx"
GITHUB_REPO="peekstore/tasklynx"

BIN_DIR="${HOME}/.local/bin"
BINARY_PATH="${BIN_DIR}/${PROJECT_NAME}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

IS_UNINSTALL=false
for arg in "$@"; do
    if [[ "$arg" == "--selfuninstall" || "$arg" == "-u" || "$arg" == "--uninstall" ]]; then
        IS_UNINSTALL=true
    fi
done

if [ "$IS_UNINSTALL" = true ]; then
    echo -e "\n${YELLOW}tasklynx Uninstaller${NC}\n"
    if [[ -f "$BINARY_PATH" ]]; then
        rm -f "$BINARY_PATH"
        echo -e "  ${GREEN}Removed${NC} $BINARY_PATH"
    fi
    rm -f "${BIN_DIR}/tx" 2>/dev/null || true
    if command -v nix-env >/dev/null 2>&1 && nix-env -q | grep -q tasklynx 2>/dev/null; then
        nix-env -e tasklynx 2>/dev/null || true
        echo -e "  ${GREEN}Removed${NC} from nix profile"
    fi
    echo -e "\n${GREEN}tasklynx uninstalled.${NC}\n"
    exit 0
fi

echo -e "\n${CYAN}tasklynx Installer${NC}\n"

# Check if on NixOS/Nix
if command -v nix >/dev/null 2>&1; then
    echo -e "  ${CYAN}Nix detected${NC}"
    if [ -f /etc/NIXOS ]; then
        echo "  For NixOS, add to your configuration.nix or home-manager:"
        echo "    environment.systemPackages = [ pkgs.tasklynx ];"
        echo "  Or use: nix profile install github:peekstore/tasklynx"
        echo "  Or run directly: nix run github:peekstore/tasklynx"
    else
        echo "  Try: nix profile install github:peekstore/tasklynx"
        echo "  Or run: nix run github:peekstore/tasklynx -- --help"
    fi
    echo ""
fi

# Detect platform
OS_NAME=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS_NAME" in
    linux*)
        if [ -n "${TERMUX_VERSION:-}" ] || [ -d "/data/data/com.termux" ]; then
            OS="android"
        elif [ -f /etc/NIXOS ]; then
            OS="linux"
        else
            OS="linux"
        fi
        ;;
    darwin*) OS="darwin" ;;
    *) echo -e "${RED}Unsupported OS${NC}"; exit 1 ;;
esac

ARCH_NAME=$(uname -m)
case "$ARCH_NAME" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7l) ARCH="arm64" ;;
    *) echo -e "${RED}Unsupported arch${NC}"; exit 1 ;;
esac

printf "  Platform: %s-%s\n" "$OS" "$ARCH"

# Get version (don't read from stdin when piped)
VERSION="latest"
if [ -t 0 ] && [ -z "${1:-}" ]; then
    VERSION="latest"
elif [ -n "${1:-}" ]; then
    VERSION="$1"
fi

if [ "$VERSION" = "latest" ] || [ -z "$VERSION" ]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | cut -d'"' -f4 | head -1 || echo "latest")
fi

if [ "$VERSION" = "latest" ] || [ -z "$VERSION" ]; then
    VERSION=$(curl -fsSL "https://raw.githubusercontent.com/${GITHUB_REPO}/main/.version" 2>/dev/null | tr -d '[:space:]' || echo "")
fi

if [ -n "$VERSION" ] && [ "$VERSION" != "latest" ]; then
    if [[ "$VERSION" != v* ]]; then VERSION="v${VERSION}"; fi
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/tasklynx-${OS}-${ARCH}"
else
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/tasklynx-${OS}-${ARCH}"
fi

echo -e "  Downloading from: ${DOWNLOAD_URL}"
mkdir -p "$BIN_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -fL --progress-bar -o "$BINARY_PATH" "$DOWNLOAD_URL" || {
        echo -e "${RED}Download failed${NC}"
        echo "Try manual download from https://github.com/${GITHUB_REPO}/releases"
        exit 1
    }
elif command -v wget >/dev/null 2>&1; then
    wget --show-progress -O "$BINARY_PATH" "$DOWNLOAD_URL" || exit 1
else
    echo -e "${RED}curl or wget required${NC}"
    exit 1
fi

chmod +x "$BINARY_PATH"

# Create tx alias
ln -sf "$BINARY_PATH" "${BIN_DIR}/tx" 2>/dev/null || cp "$BINARY_PATH" "${BIN_DIR}/tx" && chmod +x "${BIN_DIR}/tx"

# Add to PATH
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        [ -f "$rc" ] && grep -q ".local/bin" "$rc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rc" 2>/dev/null || true
    done
    export PATH="$HOME/.local/bin:$PATH"
fi

echo -e "\n${GREEN}✓ tasklynx installed to ${BINARY_PATH}${NC}"
echo -e "${GREEN}✓ tx alias created${NC}"
echo -e "${YELLOW}Run 'source ~/.bashrc' or restart terminal${NC}\n"
