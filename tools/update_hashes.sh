#!/usr/bin/env bash
set -euo pipefail

# tools/update_hashes.sh
# Updates package manager manifests (Scoop, Winget, Chocolatey, Homebrew)
# with the latest SHA256 checksums from a build directory, SHA256SUMS.txt, or release URL.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECKSUM_SRC="${1:-}"
VERSION="${2:-}"

if [ -z "$VERSION" ]; then
  if [ -f "${ROOT_DIR}/.version" ]; then
    VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/.version")"
  else
    VERSION="0.0.1-beta-4"
  fi
fi
VERSION="${VERSION#v}"

SUMS_FILE=""
CLEANUP_TEMP=false

if [ -z "$CHECKSUM_SRC" ]; then
  if [ -f "${ROOT_DIR}/final/SHA256SUMS.txt" ]; then
    SUMS_FILE="${ROOT_DIR}/final/SHA256SUMS.txt"
  elif [ -f "${ROOT_DIR}/SHA256SUMS.txt" ]; then
    SUMS_FILE="${ROOT_DIR}/SHA256SUMS.txt"
  else
    echo "Fetching SHA256SUMS.txt for v${VERSION} from GitHub releases..."
    TEMP_SUMS=$(mktemp)
    if curl -fsSL "https://github.com/peekstore/tasklynx/releases/download/v${VERSION}/SHA256SUMS.txt" -o "$TEMP_SUMS"; then
      SUMS_FILE="$TEMP_SUMS"
      CLEANUP_TEMP=true
    else
      echo "Error: Could not find or download SHA256SUMS.txt" >&2
      exit 1
    fi
  fi
elif [ -d "$CHECKSUM_SRC" ]; then
  # Calculate checksums from directory
  TEMP_SUMS=$(mktemp)
  (cd "$CHECKSUM_SRC" && find . -maxdepth 1 -type f ! -name "*.sha256" ! -name "SHA256SUMS.txt" -exec sha256sum {} +) > "$TEMP_SUMS"
  SUMS_FILE="$TEMP_SUMS"
  CLEANUP_TEMP=true
elif [ -f "$CHECKSUM_SRC" ]; then
  SUMS_FILE="$CHECKSUM_SRC"
fi

get_hash() {
  local pattern="$1"
  local match
  match=$(grep -E "${pattern}" "$SUMS_FILE" | head -n 1 | awk '{print $1}')
  echo "$match"
}

WIN_AMD64_HASH=$(get_hash "tasklynx-windows-amd64\.exe$")
WIN_ARM64_HASH=$(get_hash "tasklynx-windows-arm64\.exe$")
DARWIN_AMD64_TAR_HASH=$(get_hash "tasklynx-darwin-amd64\.tar\.gz$")
DARWIN_ARM64_TAR_HASH=$(get_hash "tasklynx-darwin-arm64\.tar\.gz$")
LINUX_AMD64_TAR_HASH=$(get_hash "tasklynx-linux-amd64\.tar\.gz$")
LINUX_ARM64_TAR_HASH=$(get_hash "tasklynx-linux-arm64\.tar\.gz$")

echo "Resolved Hashes for v${VERSION}:"
echo "  Windows AMD64 (.exe)    : ${WIN_AMD64_HASH:-<not found>}"
echo "  Windows ARM64 (.exe)    : ${WIN_ARM64_HASH:-<not found>}"
echo "  Darwin AMD64  (.tar.gz) : ${DARWIN_AMD64_TAR_HASH:-<not found>}"
echo "  Darwin ARM64  (.tar.gz) : ${DARWIN_ARM64_TAR_HASH:-<not found>}"
echo "  Linux AMD64   (.tar.gz) : ${LINUX_AMD64_TAR_HASH:-<not found>}"
echo "  Linux ARM64   (.tar.gz) : ${LINUX_ARM64_TAR_HASH:-<not found>}"

# 1. Update Chocolatey install script
CHOCO_SCRIPT="${ROOT_DIR}/packaging/chocolatey/tools/chocolateyinstall.ps1"
if [ -f "$CHOCO_SCRIPT" ] && [ -n "$WIN_AMD64_HASH" ]; then
  sed -i -E "s/checksum64[[:space:]]*=[[:space:]]*'[0-9a-fA-F]{64}'/checksum64    = '${WIN_AMD64_HASH}'/" "$CHOCO_SCRIPT"
  echo "✓ Updated Chocolatey install script"
fi

# 2. Update Scoop manifests
for SCOOP_PATH in "${ROOT_DIR}/packaging/scoop/tasklynx.json" "${ROOT_DIR}/bucket/tasklynx.json"; do
  if [ -f "$SCOOP_PATH" ]; then
    if [ -n "$WIN_AMD64_HASH" ]; then
      sed -i -E "/\"64bit\": \{/,/}/ s/\"hash\": \"[0-9a-fA-F]{64}\"/\"hash\": \"${WIN_AMD64_HASH}\"/" "$SCOOP_PATH"
    fi
    if [ -n "$WIN_ARM64_HASH" ]; then
      sed -i -E "/\"arm64\": \{/,/}/ s/\"hash\": \"[0-9a-fA-F]{64}\"/\"hash\": \"${WIN_ARM64_HASH}\"/" "$SCOOP_PATH"
    fi
    echo "✓ Updated Scoop manifest: ${SCOOP_PATH}"
  fi
done

# 3. Update Winget manifests
WINGET_INSTALLER="${ROOT_DIR}/packaging/winget/manifests/p/peekstore/tasklynx/${VERSION}/peekstore.tasklynx.installer.yaml"
if [ -f "$WINGET_INSTALLER" ]; then
  if [ -n "$WIN_AMD64_HASH" ]; then
    sed -i -E "/Architecture: x64/,/Commands:/ s/InstallerSha256: [0-9a-fA-F]{64}/InstallerSha256: ${WIN_AMD64_HASH}/" "$WINGET_INSTALLER"
  fi
  if [ -n "$WIN_ARM64_HASH" ]; then
    sed -i -E "/Architecture: arm64/,/Commands:/ s/InstallerSha256: [0-9a-fA-F]{64}/InstallerSha256: ${WIN_ARM64_HASH}/" "$WINGET_INSTALLER"
  fi
  echo "✓ Updated Winget multi-manifest: ${WINGET_INSTALLER}"
fi

WINGET_SINGLE="${ROOT_DIR}/packaging/winget/peekstore.tasklynx.yaml"
if [ -f "$WINGET_SINGLE" ]; then
  if [ -n "$WIN_AMD64_HASH" ]; then
    sed -i -E "/Architecture: x64/,/Commands:/ s/InstallerSha256: [0-9a-fA-F]{64}/InstallerSha256: ${WIN_AMD64_HASH}/" "$WINGET_SINGLE"
  fi
  if [ -n "$WIN_ARM64_HASH" ]; then
    sed -i -E "/Architecture: arm64/,/Commands:/ s/InstallerSha256: [0-9a-fA-F]{64}/InstallerSha256: ${WIN_ARM64_HASH}/" "$WINGET_SINGLE"
  fi
  echo "✓ Updated Winget singleton: ${WINGET_SINGLE}"
fi

if [ "$CLEANUP_TEMP" = true ] && [ -f "$SUMS_FILE" ]; then
  rm -f "$SUMS_FILE"
fi

echo "Hash updates complete!"
