# Changelog

## 0.0.1-beta-4

- Added dynamic release notes generation in GitHub Actions with changelog, installation, downloads, and AI reader links.
- Expanded installer test matrix covering all 12 distribution methods: Linux x64/arm64, macOS, Windows PowerShell, Android Termux ABIs, Nix Flake, pipx, npm, Debian .deb, Homebrew, mise, Chocolatey, Winget, Scoop, and Pacman.
- Added AI reader (`llms.txt` and `llms-full.txt`) links and badges to README and documentation.
- Dynamic auto-version synchronization across all 30 manifests and package repositories.

## 0.0.1-beta-3

- Complete support for multi-package managers (mise, pacman, apt, homebrew, pipx, winget, scoop, chocolatey, npm).
- Automated package checksum syncing via GitHub Actions.
- Real high-DPI screenshots generated and updated via headless Chromium automation in CI.
- Dual dark/light mode banner support for Void MkDocs theme and GitHub README.
- Built-in Void theme dark/light palette switcher and AI reader (`llms.txt`, `llms-full.txt`).

## 0.0.1-beta-2

- Android builds now use real Android ABI artifacts: `arm64-v8a`, `armeabi-v7a`, `x86_64`.
- Installer detects Termux and selects the matching `tasklynx-android-<abi>` binary.
- Reliable POSIX installer with working `--selfuninstall` (removes binary and `tx` alias).
- Windows installer validates the downloaded PE binary and supports ARM64 detection.
- `tx self update` uses `installer.sh`; `tx self uninstall` removes the `tx` alias.
- Release workflow is idempotent (removes stale drafts/releases before publishing).
- CI installers tested on Linux x64/arm64, macOS, Windows, Termux ABIs and Nix.

## 0.0.1-beta-1

- Rust rewrite.
- Ratatui terminal interface.
- Responsive embedded web dashboard.
- Branches, projects, tags, priorities and deadlines.
- JSON/Markdown export.
- Windows icon embedding.
- Cross-platform release workflow.
- Termux Android targets.
- MkDocs Void documentation.
