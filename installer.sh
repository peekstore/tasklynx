#!/bin/sh
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
curl -fsSL -o "$BIN_DIR/tasklynx" "https://github.com/peekstore/tasklynx/releases/download/v0.0.1-beta-1/tasklynx-linux-amd64" || exit 1
chmod +x "$BIN_DIR/tasklynx"
ln -sf "$BIN_DIR/tasklynx" "$BIN_DIR/tx" 2>/dev/null
echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc" 2>/dev/null
echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null
export PATH="$BIN_DIR:$PATH"
"$BIN_DIR/tx" --version 2>/dev/null || "$BIN_DIR/tasklynx" --version
