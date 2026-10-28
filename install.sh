#!/usr/bin/env bash
set -euo pipefail
REPO="peekstore/tasklynx"
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

# Install desktop entry and icon
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
SCALABLE_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
PIXMAP_DIR="$HOME/.local/share/pixmaps"
DESK_DIR="$HOME/.local/share/applications"
BIN_DIR="/usr/local/bin"
if [ -w "$BIN_DIR" ] && [ "$(id -u)" != "0" ]; then
  BIN_DIR="$HOME/.local/bin"
  mkdir -p "$BIN_DIR"
  export PATH="$BIN_DIR:$PATH"
fi
mkdir -p "$ICON_DIR" "$SCALABLE_DIR" "$PIXMAP_DIR" "$DESK_DIR"
curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/assets/tasklynx.png" -o "$PIXMAP_DIR/tasklynx.png" 2>/dev/null || cp "$TMP/../assets/tasklynx.png" "$PIXMAP_DIR/tasklynx.png" 2>/dev/null || true
cp "$PIXMAP_DIR/tasklynx.png" "$ICON_DIR/tasklynx.png" 2>/dev/null || true
curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/assets/tasklynx.svg" -o "$SCALABLE_DIR/tasklynx.svg" 2>/dev/null || cp "$TMP/../assets/tasklynx.svg" "$SCALABLE_DIR/tasklynx.svg" 2>/dev/null || true
cat > "$DESK_DIR/tasklynx.desktop" << DESK
[Desktop Entry]
Version=1.0
Type=Application
Name=Tasklynx
GenericName=Task Manager
Comment=Developer-first local task manager with terminal UI
Exec=tasklynx
Icon=tasklynx
Terminal=true
Categories=Development;Utility;Office;
Keywords=tasks;todo;kanban;cli;
StartupNotify=true
DESK
chmod +x "$DESK_DIR/tasklynx.desktop" 2>/dev/null || true
update-desktop-database "$DESK_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
command -v xdg-icon-resource >/dev/null 2>&1 && xdg-icon-resource install --novendor --size 256 "$ICON_DIR/tasklynx.png" tasklynx 2>/dev/null || true
if [ -d "$BIN_DIR" ] && [ "$(basename "$BIN_DIR")" = ".local" ]; then
  if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    echo "Add $HOME/.local/bin to your PATH if not already present"
  fi
fi

# Install desktop entry and icon
if command -v xdg-desktop-menu >/dev/null 2>&1 || [ "$(uname -s)" = "Linux" ]; then
  ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
  DESK_DIR="$HOME/.local/share/applications"
  mkdir -p "$ICON_DIR" "$DESK_DIR"
  curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/assets/tasklynx.png" -o "$ICON_DIR/tasklynx.png" 2>/dev/null || cp "$(dirname "$0")/../assets/tasklynx.png" "$ICON_DIR/tasklynx.png" 2>/dev/null || true
  cat > "$DESK_DIR/tasklynx.desktop" << DESK
[Desktop Entry]
Name=Tasklynx
Comment=Developer-first local task manager
Exec=tasklynx
Icon=tasklynx
Type=Application
Categories=Development;Utility;
Terminal=true
DESK
  update-desktop-database "$DESK_DIR" 2>/dev/null || true
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
