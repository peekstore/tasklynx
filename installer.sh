#!/usr/bin/env bash
# installer.sh - Installer and Uninstaller for tasklynx on Linux / macOS

set -euo pipefail

PROJECT_NAME="tasklynx"
GITHUB_REPO="peekstore/tasklynx"

CONFIG_DIR="${HOME}/.config/tasklynx"
BIN_DIR="${HOME}/.local/bin"
BINARY_PATH="${BIN_DIR}/${PROJECT_NAME}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

clean_profile() {
    local profile="$1"
    if [[ -f "$profile" ]]; then
        grep -v "${PROJECT_NAME}" "$profile" > "${profile}.tmp" 2>/dev/null || true
        mv "${profile}.tmp" "$profile"
    fi
}

add_to_profile() {
    local profile="$1"
    mkdir -p "$(dirname "$BIN_DIR")"
    if [[ -f "$profile" ]] && ! grep -q "\.local/bin" "$profile"; then
        echo "" >> "$profile"
        echo "# tasklynx PATH" >> "$profile"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$profile"
    elif [[ ! -f "$profile" ]]; then
        echo "# tasklynx PATH" > "$profile"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$profile"
    fi
}

IS_UNINSTALL=false
for arg in "$@"; do
    if [[ "$arg" == "--selfuninstall" || "$arg" == "-u" || "$arg" == "--uninstall" ]]; then
        IS_UNINSTALL=true
    fi
done

if [ "$IS_UNINSTALL" = true ]; then
    echo -e "\n${YELLOW}tasklynx Uninstaller${NC}\n"
    if [[ -f "$BINARY_PATH" ]]; then
        printf "  Removing binary: %s ... " "$BINARY_PATH"
        rm -f "$BINARY_PATH"
        echo -e "${GREEN}OK${NC}"
    fi
    for p in "${HOME}/.bashrc" "${HOME}/.zshrc" "${HOME}/.profile" "${HOME}/.bash_profile"; do
        if [[ -f "$p" ]]; then
            printf "  Updating %s ... " "$(basename "$p")"
            clean_profile "$p"
            echo -e "${GREEN}OK${NC}"
        fi
    done
    echo -e "\n${GREEN}tasklynx uninstalled.${NC}\n"
    exit 0
fi

echo -e "\n${CYAN}tasklynx Installer${NC}\n"

printf "  Checking latest version ... "
VERSION_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main/.version"
if command -v curl >/dev/null 2>&1; then
    VERSION=$(curl -fsSL "$VERSION_URL" 2>/dev/null | tr -d '[:space:]' || echo "latest")
elif command -v wget >/dev/null 2>&1; then
    VERSION=$(wget -qO- "$VERSION_URL" 2>/dev/null | tr -d '[:space:]' || echo "latest")
else
    VERSION="latest"
fi
if [ -z "$VERSION" ]; then VERSION="latest"; fi
echo -e "${GREEN}${VERSION}${NC}"

printf "  Detecting platform ... "
OS_NAME=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS_NAME" in
    linux*) OS="linux" ;;
    darwin*) OS="darwin" ;;
    *) echo -e "${RED}FAILED${NC}"; echo "Unsupported OS: $OS_NAME"; exit 1 ;;
esac
ARCH_NAME=$(uname -m)
case "$ARCH_NAME" in
    x86_64|amd64) ARCH="amd64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo -e "${RED}FAILED${NC}"; echo "Unsupported arch: $ARCH_NAME"; exit 1 ;;
esac
echo -e "${GREEN}${OS}-${ARCH}${NC}"

if [ "$VERSION" = "latest" ]; then
    LATEST=$(curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" 2>/dev/null | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -1)
    [ -n "$LATEST" ] && VERSION="$LATEST"
fi

DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/tasklynx-${OS}-${ARCH}"
echo -e "  Downloading from ${DOWNLOAD_URL}\n"
mkdir -p "$BIN_DIR"
if command -v curl >/dev/null 2>&1; then
    curl -fL --progress-bar -o "$BINARY_PATH" "$DOWNLOAD_URL"
elif command -v wget >/dev/null 2>&1; then
    wget --show-progress -O "$BINARY_PATH" "$DOWNLOAD_URL"
else
    echo -e "${RED}curl/wget required${NC}"; exit 1
fi
chmod +x "$BINARY_PATH"

ACTIVE_SHELL=$(basename "${SHELL:-bash}")
if [ "$ACTIVE_SHELL" = "zsh" ]; then
    touch "${HOME}/.zshrc"; add_to_profile "${HOME}/.zshrc"; SRC=".zshrc"
else
    touch "${HOME}/.bashrc"; add_to_profile "${HOME}/.bashrc"; SRC=".bashrc"
fi

echo -e "\n${GREEN}tasklynx ${VERSION} installed to ${BINARY_PATH}${NC}"
echo -e "${YELLOW}Restart shell or run: source ~/${SRC}${NC}\n"
