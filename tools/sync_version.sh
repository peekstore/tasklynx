#!/usr/bin/env bash
set -euo pipefail

# tools/sync_version.sh
# Reads the root .version file (or accepts an argument)
# and synchronizes the version across every manifest, source, and documentation file.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RAW_VERSION="${1:-}"
if [ -z "$RAW_VERSION" ]; then
  if [ -f "${ROOT_DIR}/.version" ]; then
    RAW_VERSION="$(tr -d '[:space:]' < "${ROOT_DIR}/.version")"
  else
    echo "Error: No version provided and .version file not found" >&2
    exit 1
  fi
fi

# Clean versions
FULL_VERSION="${RAW_VERSION#v}"
TAG="v${FULL_VERSION}"

# PEP 440 version (e.g. 0.0.1-beta-3 -> 0.0.1b3)
PEP440_VERSION=$(echo "$FULL_VERSION" | sed -E 's/[-.]beta[-.]?/b/' | sed -E 's/[-.]alpha[-.]?/a/' | sed -E 's/[-.]rc[-.]?/rc/')

# Arch Linux pkgver (e.g. 0.0.1-beta-3 -> 0.0.1.beta3)
ARCH_PKGVER=$(echo "$FULL_VERSION" | sed -E 's/[-.]beta[-.]?/.beta/' | tr '-' '.')

echo "Synchronizing version across project:"
echo "  Raw version    : ${RAW_VERSION}"
echo "  Clean version  : ${FULL_VERSION}"
echo "  Git tag        : ${TAG}"
echo "  PEP440 (pipx)  : ${PEP440_VERSION}"
echo "  Pacman (arch)  : ${ARCH_PKGVER}"

# 1. Update .version
echo -n "${FULL_VERSION}" > "${ROOT_DIR}/.version"
echo "" >> "${ROOT_DIR}/.version"

# 2. Cargo.toml & Cargo.lock
if [ -f "${ROOT_DIR}/Cargo.toml" ]; then
  sed -i -E 's/^version = "[^"]+"/version = "'"${FULL_VERSION}"'"/' "${ROOT_DIR}/Cargo.toml"
  if command -v cargo >/dev/null 2>&1; then
    (cd "${ROOT_DIR}" && cargo check --quiet 2>/dev/null || true)
  fi
  echo "✓ Cargo.toml and Cargo.lock"
fi

# 3. CITATION.cff
if [ -f "${ROOT_DIR}/CITATION.cff" ]; then
  sed -i -E 's/^version: .*/version: '"${FULL_VERSION}"'/' "${ROOT_DIR}/CITATION.cff"
  echo "✓ CITATION.cff"
fi

# 4. Installers
if [ -f "${ROOT_DIR}/installer.sh" ]; then
  sed -i -E 's/VERSION="v[0-9a-zA-Z.-]+"/VERSION="'"${TAG}"'"/' "${ROOT_DIR}/installer.sh"
  echo "✓ installer.sh"
fi
if [ -f "${ROOT_DIR}/installer.ps1" ]; then
  sed -i -E 's/if \(-not \$version\) \{ \$version = "v[0-9a-zA-Z.-]+" \}/if (-not $version) { $version = "'"${TAG}"'" }/' "${ROOT_DIR}/installer.ps1"
  echo "✓ installer.ps1"
fi

# 5. Chocolatey
if [ -f "${ROOT_DIR}/packaging/chocolatey/tasklynx.nuspec" ]; then
  sed -i -E 's/<version>[^<]+<\/version>/<version>'"${FULL_VERSION}"'<\/version>/' "${ROOT_DIR}/packaging/chocolatey/tasklynx.nuspec"
  sed -i -E 's|<releaseNotes>https://github\.com/peekstore/tasklynx/releases/tag/v[^<]+</releaseNotes>|<releaseNotes>https://github.com/peekstore/tasklynx/releases/tag/'"${TAG}"'</releaseNotes>|' "${ROOT_DIR}/packaging/chocolatey/tasklynx.nuspec"
  echo "✓ Chocolatey nuspec"
fi
if [ -f "${ROOT_DIR}/packaging/chocolatey/tools/chocolateyinstall.ps1" ]; then
  sed -i -E "s/\\\$version[[:space:]]*=[[:space:]]*'[^']+'/\$version      = '${FULL_VERSION}'/" "${ROOT_DIR}/packaging/chocolatey/tools/chocolateyinstall.ps1"
  echo "✓ Chocolatey install.ps1"
fi

# 6. Debian
if [ -f "${ROOT_DIR}/packaging/debian/build_deb.sh" ]; then
  sed -i -E 's/VERSION="\$\{1:-[^}]+\}"/VERSION="\${1:-'"${FULL_VERSION}"'}"/' "${ROOT_DIR}/packaging/debian/build_deb.sh"
  echo "✓ Debian build_deb.sh"
fi

# 7. Homebrew
for BREW_FILE in "${ROOT_DIR}/Formula/tasklynx.rb" "${ROOT_DIR}/packaging/homebrew/tasklynx.rb"; do
  if [ -f "$BREW_FILE" ]; then
    sed -i -E 's/version "[^"]+"/version "'"${FULL_VERSION}"'"/' "$BREW_FILE"
    echo "✓ Homebrew formula: $(basename "$BREW_FILE")"
  fi
done

# 8. NPM
if [ -f "${ROOT_DIR}/packaging/npm/package.json" ]; then
  sed -i -E 's/"version": "[^"]+"/"version": "'"${FULL_VERSION}"'"/' "${ROOT_DIR}/packaging/npm/package.json"
  echo "✓ NPM package.json"
fi
if [ -f "${ROOT_DIR}/packaging/npm/bin/tasklynx.js" ]; then
  sed -i -E "s/const VERSION = '[^']+';/const VERSION = '${FULL_VERSION}';/" "${ROOT_DIR}/packaging/npm/bin/tasklynx.js"
  echo "✓ NPM launcher binary"
fi

# 9. Pipx
if [ -f "${ROOT_DIR}/packaging/pipx/pyproject.toml" ]; then
  sed -i -E 's/^version = "[^"]+"/version = "'"${PEP440_VERSION}"'"/' "${ROOT_DIR}/packaging/pipx/pyproject.toml"
  echo "✓ Pipx pyproject.toml"
fi
if [ -f "${ROOT_DIR}/packaging/pipx/tasklynx/__init__.py" ]; then
  sed -i -E 's/__version__ = "[^"]+"/__version__ = "'"${PEP440_VERSION}"'"/' "${ROOT_DIR}/packaging/pipx/tasklynx/__init__.py"
  echo "✓ Pipx __init__.py"
fi
if [ -f "${ROOT_DIR}/packaging/pipx/tasklynx/cli.py" ]; then
  sed -i -E 's/VERSION = "[^"]+"/VERSION = "'"${FULL_VERSION}"'"/' "${ROOT_DIR}/packaging/pipx/tasklynx/cli.py"
  echo "✓ Pipx cli.py"
fi

# 10. Arch Linux / Pacman
if [ -f "${ROOT_DIR}/packaging/pacman/PKGBUILD" ]; then
  sed -i -E 's/^pkgver=.*/pkgver='"${ARCH_PKGVER}"'/' "${ROOT_DIR}/packaging/pacman/PKGBUILD"
  sed -i -E 's/^_tag=.*/_tag='"${TAG}"'/' "${ROOT_DIR}/packaging/pacman/PKGBUILD"
  if command -v makepkg >/dev/null 2>&1; then
    (cd "${ROOT_DIR}/packaging/pacman" && makepkg --printsrcinfo > .SRCINFO 2>/dev/null || true)
  fi
  echo "✓ Arch Linux PKGBUILD & .SRCINFO"
fi

# 11. Scoop
for SCOOP_FILE in "${ROOT_DIR}/packaging/scoop/tasklynx.json" "${ROOT_DIR}/bucket/tasklynx.json"; do
  if [ -f "$SCOOP_FILE" ]; then
    sed -i -E 's/"version": "[^"]+"/"version": "'"${FULL_VERSION}"'"/' "$SCOOP_FILE"
    sed -i -E '1,/"autoupdate":/ s/download\/v[0-9a-zA-Z.-]+\/tasklynx-windows/download\/'"${TAG}"'\/tasklynx-windows/g' "$SCOOP_FILE"
    echo "✓ Scoop manifest: $(basename "$SCOOP_FILE")"
  fi
done

# 12. Winget
if [ -f "${ROOT_DIR}/packaging/winget/peekstore.tasklynx.yaml" ]; then
  sed -i -E 's/^PackageVersion: .*/PackageVersion: '"${FULL_VERSION}"'/' "${ROOT_DIR}/packaging/winget/peekstore.tasklynx.yaml"
  sed -i -E 's/download\/v[0-9a-zA-Z.-]+\/tasklynx-windows/download\/'"${TAG}"'\/tasklynx-windows/g' "${ROOT_DIR}/packaging/winget/peekstore.tasklynx.yaml"
  echo "✓ Winget singleton manifest"
fi

WINGET_DIR="${ROOT_DIR}/packaging/winget/manifests/p/peekstore/tasklynx"
if [ -d "$WINGET_DIR" ]; then
  TARGET_DIR="${WINGET_DIR}/${FULL_VERSION}"
  if [ ! -d "$TARGET_DIR" ]; then
    LATEST_PREV=$(find "$WINGET_DIR" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)
    if [ -n "$LATEST_PREV" ] && [ "$LATEST_PREV" != "$TARGET_DIR" ]; then
      cp -r "$LATEST_PREV" "$TARGET_DIR"
    else
      mkdir -p "$TARGET_DIR"
    fi
  fi
  if [ -d "$TARGET_DIR" ]; then
    find "$TARGET_DIR" -type f -exec sed -i -E "s/PackageVersion: [0-9a-zA-Z.-]+/PackageVersion: ${FULL_VERSION}/g" {} +
    find "$TARGET_DIR" -type f -exec sed -i -E "s/download\/v[0-9a-zA-Z.-]+\/tasklynx-windows/download\/${TAG}\/tasklynx-windows/g" {} +
    echo "✓ Winget multi-manifest directory: ${FULL_VERSION}"
  fi
fi

# 13. Documentation & Markdown files
for MD_FILE in "${ROOT_DIR}/README.md" "${ROOT_DIR}/docs/index.md" "${ROOT_DIR}/docs/installation.md" "${ROOT_DIR}/docs/development.md" "${ROOT_DIR}/packaging/npm/README.md" "${ROOT_DIR}/packaging/pipx/README.md"; do
  if [ -f "$MD_FILE" ]; then
    sed -i -E "s/tasklynx_[0-9a-zA-Z.-]+_amd64\.deb/tasklynx_${FULL_VERSION}_amd64.deb/g" "$MD_FILE"
    sed -i -E "s/TASKLYNX_VERSION=v[0-9a-zA-Z.-]+/TASKLYNX_VERSION=${TAG}/g" "$MD_FILE"
    sed -i -E "s/\\\$env:TASKLYNX_VERSION = \"v[0-9a-zA-Z.-]+\"/\$env:TASKLYNX_VERSION = \"${TAG}\"/g" "$MD_FILE"
  fi
done
if [ -f "${ROOT_DIR}/docs/development.md" ]; then
  sed -i -E "s/git tag -a v[0-9a-zA-Z.-]+ -m \"Tasklynx v[0-9a-zA-Z.-]+\"/git tag -a ${TAG} -m \"Tasklynx ${TAG}\"/" "${ROOT_DIR}/docs/development.md"
  sed -i -E "s/git push origin v[0-9a-zA-Z.-]+/git push origin ${TAG}/" "${ROOT_DIR}/docs/development.md"
fi
if [ -f "${ROOT_DIR}/docs/index.md" ]; then
  sed -i -E "s/- \*\*Version:\*\* .*/- **Version:** ${FULL_VERSION}/" "${ROOT_DIR}/docs/index.md"
fi
if [ -f "${ROOT_DIR}/.github/ISSUE_TEMPLATE/bug_report.md" ]; then
  sed -i -E "s/e\.g\. [0-9a-zA-Z.-]+ \(run/e.g. ${FULL_VERSION} (run/" "${ROOT_DIR}/.github/ISSUE_TEMPLATE/bug_report.md"
fi
if [ -f "${ROOT_DIR}/.github/workflows/package-publish.yml" ]; then
  sed -i -E 's/build_deb\.sh "[0-9a-zA-Z.-]+"/build_deb.sh "'"${FULL_VERSION}"'"/' "${ROOT_DIR}/.github/workflows/package-publish.yml"
fi
if [ -f "${ROOT_DIR}/tools/update_hashes.sh" ]; then
  sed -i -E 's/VERSION="[0-9a-zA-Z.-]+"/VERSION="'"${FULL_VERSION}"'"/' "${ROOT_DIR}/tools/update_hashes.sh"
fi

echo "All files successfully synchronized to version ${FULL_VERSION} (${TAG})!"
