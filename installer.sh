#!/bin/sh
# Tasklynx Installer for Linux/macOS/Android (Termux)
set -e

PROJECT_NAME="tasklynx"
VERSION_URL="https://raw.githubusercontent.com/peekstore/${PROJECT_NAME}/refs/heads/main/.version"
RELEASE_URL="https://github.com/peekstore/${PROJECT_NAME}/releases/download"

IS_UNINSTALL=false
IS_PURGE=false
for arg in "$@"; do
    case "$arg" in
        --selfuninstall|--uninstall|-u) IS_UNINSTALL=true ;;
        --purge) IS_PURGE=true ;;
    esac
done

if [ "$IS_UNINSTALL" = true ]; then
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        BASE_DIR="$XDG_CONFIG_HOME/peekstore/${PROJECT_NAME}"
    else
        BASE_DIR="$HOME/.config/peekstore/${PROJECT_NAME}"
    fi
    BIN_DIR="$BASE_DIR/bin"
    echo ""
    echo "Tasklynx Uninstaller"
    echo ""
    if [ -f "${BIN_DIR}/${PROJECT_NAME}" ]; then
        rm -f "${BIN_DIR}/${PROJECT_NAME}" && echo "  Removed ${BIN_DIR}/${PROJECT_NAME}"
    fi
    if [ -e "${BIN_DIR}/tx" ] || [ -L "${BIN_DIR}/tx" ]; then
        rm -f "${BIN_DIR}/tx" && echo "  Removed ${BIN_DIR}/tx"
    fi
    [ -d "$BIN_DIR" ] && rmdir "$BIN_DIR" 2>/dev/null || true
    if [ "$IS_PURGE" = true ]; then
        rm -rf "$BASE_DIR" && echo "  Purged config directory: $BASE_DIR"
        DATA_DIR="${TASKLYNX_HOME:-$HOME/.local/share/tasklynx}"
        rm -rf "$DATA_DIR" && echo "  Purged data directory: $DATA_DIR"
    else
        echo "  Note: config and data directories preserved. Use --purge to remove."
    fi
    echo ""
    echo "Tasklynx uninstalled. Restart your terminal to apply PATH changes."
    exit 0
fi

VERSION="${TASKLYNX_VERSION:-}"
if [ -z "$VERSION" ] && command -v curl >/dev/null 2>&1; then
    VERSION=$(curl -fsSL "$VERSION_URL" 2>/dev/null | tr -d ' \n\r' || true)
fi
if [ -z "$VERSION" ]; then
    VERSION="v0.0.1-beta-3"
fi
case "$VERSION" in
    v*) ;;
    *) VERSION="v${VERSION}" ;;
esac

echo ">>> Installing ${PROJECT_NAME} ${VERSION}..."
echo ""

OS_NAME="$(uname -s)"
OS_LOWER="$(echo "$OS_NAME" | tr '[:upper:]' '[:lower:]')"
ARCH_NAME="$(uname -m)"

ARCH="amd64"
case "$ARCH_NAME" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7l|armv8l) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH_NAME"; exit 1 ;;
esac

OS="linux"
if [ -n "${TERMUX_VERSION:-}" ] || [ -d "/data/data/com.termux" ]; then
    OS="android"
fi
case "$OS_LOWER" in
    darwin*) OS="darwin" ;;
    *) 
        if [ "$OS" != "android" ]; then
            OS="linux"
        fi
        ;;
esac

case "$OS" in
    android)
        # Android/Termux uses ABI names, not amd64/arm64.
        ABI=""
        if command -v getprop >/dev/null 2>&1; then
            ABI="$(getprop ro.product.cpu.abi 2>/dev/null || true)"
        fi
        case "$ABI" in
            arm64-v8a) ABI="arm64-v8a" ;;
            x86_64) ABI="x86_64" ;;
            armeabi-v7a|armeabi) ABI="armeabi-v7a" ;;
        esac
        if [ -z "$ABI" ]; then
            case "$ARCH_NAME" in
                aarch64|arm64) ABI="arm64-v8a" ;;
                x86_64|amd64) ABI="x86_64" ;;
                armv7l|armv8l) ABI="armeabi-v7a" ;;
                *) echo "Unsupported Android ABI: $ARCH_NAME"; exit 1 ;;
            esac
        fi
        DOWNLOAD_NAME="${PROJECT_NAME}-android-${ABI}"
        ;;
    linux) DOWNLOAD_NAME="${PROJECT_NAME}-linux-${ARCH}" ;;
    darwin) DOWNLOAD_NAME="${PROJECT_NAME}-darwin-${ARCH}" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

DOWNLOAD_URL="${RELEASE_URL}/${VERSION}/${DOWNLOAD_NAME}"

if [ "${DRY_RUN:-}" = "1" ]; then
    echo "$DOWNLOAD_NAME"
    exit 0
fi

if [ -n "${XDG_CONFIG_HOME:-}" ]; then
    BIN_DIR="$XDG_CONFIG_HOME/peekstore/${PROJECT_NAME}/bin"
else
    BIN_DIR="$HOME/.config/peekstore/${PROJECT_NAME}/bin"
fi
mkdir -p "$BIN_DIR"

echo "Downloading ${DOWNLOAD_URL}"
if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "${BIN_DIR}/${PROJECT_NAME}" "$DOWNLOAD_URL" || {
        echo "Error: Failed to download ${PROJECT_NAME}"
        exit 1
    }
elif command -v wget >/dev/null 2>&1; then
    wget -O "${BIN_DIR}/${PROJECT_NAME}" "$DOWNLOAD_URL" || {
        echo "Error: Failed to download ${PROJECT_NAME}"
        exit 1
    }
else
    echo "Error: curl or wget required"
    exit 1
fi

chmod +x "${BIN_DIR}/${PROJECT_NAME}"
if [ ! -f "${BIN_DIR}/${PROJECT_NAME}" ] || [ ! -s "${BIN_DIR}/${PROJECT_NAME}" ]; then
    echo "Error: Download failed or empty file"
    exit 1
fi

# Verify it's a binary, not HTML
if head -c 15 "${BIN_DIR}/${PROJECT_NAME}" | grep -q "<"; then
    echo "Error: Downloaded HTML instead of binary (wrong URL/version)"
    exit 1
fi

ln -sf "${BIN_DIR}/${PROJECT_NAME}" "${BIN_DIR}/tx" 2>/dev/null

SIZE=$(du -k "${BIN_DIR}/${PROJECT_NAME}" 2>/dev/null | cut -f1 || echo "?")
echo "OK   Installed to ${BIN_DIR}/${PROJECT_NAME} (${SIZE} KB)"

SHELL_NAME="$(basename "${SHELL:-sh}")"
SHELL_RC=""
case "$SHELL_NAME" in
    bash) SHELL_RC="$HOME/.bashrc" ;;
    zsh)  SHELL_RC="$HOME/.zshrc" ;;
    fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
    *)    SHELL_RC="$HOME/.profile" ;;
esac

case ":$PATH:" in
    *":$BIN_DIR:"*) echo "Already in PATH" ;;
    *)
        echo "Adding to PATH..."
        if [ "$SHELL_NAME" = "fish" ]; then
            mkdir -p "$(dirname "$SHELL_RC")"
            echo "set -gx PATH $BIN_DIR \$PATH" >> "$SHELL_RC"
        else
            echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$SHELL_RC"
        fi
        echo "PATH updated. Run 'source $SHELL_RC' or restart your terminal."
        ;;
esac

export PATH="$PATH:$BIN_DIR"
echo ""
echo "OK   ${PROJECT_NAME} ${VERSION} installed successfully!"
echo "Run '${PROJECT_NAME} --help' or 'tx --help' to get started"
